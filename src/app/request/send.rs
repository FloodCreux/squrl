use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use reqwest::header::HeaderMap;
use reqwest::multipart::Part;
use reqwest::redirect::Policy;
use reqwest::{ClientBuilder, Proxy, Url};
use reqwest_middleware::Extension;
use reqwest_tracing::{DisableOtelPropagation, OtelName, TracingMiddleware};
use thiserror::Error;
use tracing::trace;

use crate::app::App;
use crate::app::constants::FILE_VALUE_PREFIX;
use crate::app::files::environment::save_environment_to_file;
use crate::app::request::scripts::{execute_post_request_script, execute_pre_request_script};
use crate::app::request::send::RequestResponseError::PostRequestScript;
use crate::models::auth::auth::Auth;
use crate::models::auth::basic::BasicAuth;
use crate::models::auth::bearer_token::BearerToken;
use crate::models::auth::digest::{Digest, digest_to_authorization_header};
use crate::models::auth::jwt::{JwtError, JwtToken, jwt_do_jaat};
use crate::models::environment::Environment;
use crate::models::protocol::http::body::ContentType::{
	File, Form, Html, Javascript, Json, Multipart, NoBody, Raw, Xml,
};
use crate::models::protocol::protocol::Protocol;
use crate::models::request::Request;
use crate::models::response::RequestResponse;
use anyhow::Context;

#[derive(Error, Debug)]
pub enum PrepareRequestError {
	#[error("(CONSOLE) PRE-REQUEST SCRIPT ERROR")]
	PreRequestScript,
	#[error("INVALID URL")]
	InvalidUrl,
	#[error("COULD NOT OPEN FILE")]
	CouldNotOpenFile,
	#[error("{0}")]
	JwtError(#[from] JwtError),
	#[error("{0}")]
	Other(#[from] anyhow::Error),
}

/// Result of the synchronous `prepare_request` phase.
/// When the request body is a file, the builder cannot be finalized synchronously
/// because opening the file requires an async call. In that case, `pending_file`
/// contains the path that the caller must open asynchronously and attach to the
/// builder via `.body()`.
pub struct PreparedRequest {
	pub builder: reqwest_middleware::RequestBuilder,
	pub pending_file: Option<PathBuf>,
}

#[derive(Error, Debug)]
pub enum RequestResponseError {
	#[error("(CONSOLE) POST-SCRIPT ERROR")]
	PostRequestScript,
	#[error("COULD NOT DECODE RESPONSE TEXT OR BYTES")]
	CouldNotDecodeResponse,
	#[error(transparent)]
	WebsocketError(#[from] reqwest_websocket::Error),
}

impl App<'_> {
	/// Prepare an HTTP/WS request synchronously.
	///
	/// When the request body is a `File`, the returned [`PreparedRequest`] will
	/// contain `pending_file = Some(path)`. The caller is responsible for
	/// opening the file asynchronously and attaching it to the builder via
	/// `builder.body(file)` before sending.
	pub fn prepare_request(
		&self,
		request: &mut Request,
	) -> Result<PreparedRequest, PrepareRequestError> {
		trace!("Preparing request");

		let env = self.get_selected_env_as_local();

		let mut client_builder = ClientBuilder::new()
			.default_headers(HeaderMap::new())
			.referer(false);

		/* REDIRECTS */

		if !request.settings.allow_redirects.as_bool().unwrap_or(true) {
			client_builder = client_builder.redirect(Policy::none());
		}

		/* STORE COOKIES */

		let should_store_cookies = request
			.settings
			.store_received_cookies
			.as_bool()
			.unwrap_or(true);

		client_builder = client_builder.cookie_store(should_store_cookies);

		/* PROXY */

		if request.settings.use_config_proxy.as_bool().unwrap_or(true)
			&& let Some(proxy) = &self.core.config.get_proxy()
		{
			if let Some(http_proxy_str) = &proxy.http_proxy {
				let proxy = Proxy::http(http_proxy_str).context("Could not parse HTTP proxy")?;
				client_builder = client_builder.proxy(proxy);
			}

			if let Some(https_proxy_str) = &proxy.https_proxy {
				let proxy = Proxy::https(https_proxy_str).context("Could not parse HTTPS proxy")?;
				client_builder = client_builder.proxy(proxy);
			}
		}

		/* COOKIES */

		let local_cookie_store = Arc::clone(&self.core.cookies_popup.cookie_store);
		client_builder = client_builder.cookie_provider(local_cookie_store);

		/* PRE-REQUEST SCRIPT */

		let modified_request = self.handle_pre_request_script(request, env)?;

		/* INVALID CERTS */

		if request
			.settings
			.accept_invalid_certs
			.as_bool()
			.unwrap_or(false)
		{
			client_builder = client_builder.danger_accept_invalid_certs(true);
		}

		/* INVALID HOSTNAMES */

		if request
			.settings
			.accept_invalid_hostnames
			.as_bool()
			.unwrap_or(false)
		{
			client_builder = client_builder.danger_accept_invalid_hostnames(true);
		}

		/* CLIENT */

		let untraced_client = client_builder
			.build()
			.context("Could not build HTTP client")?;
		let client = reqwest_middleware::ClientBuilder::new(untraced_client)
			.with(TracingMiddleware::default())
			.with_init(Extension(OtelName(modified_request.name.into())))
			.with_init(Extension(DisableOtelPropagation))
			.build();

		/* PARAMS */

		let params = self.key_value_vec_to_tuple_vec(&modified_request.params);
		let query_params = params
			.iter()
			.filter(|(key, _)| !(key.starts_with("{") && key.ends_with("}")));
		let path_params = params
			.iter()
			.filter(|(key, _)| key.starts_with("{") && key.ends_with("}"));

		/* URL */

		let mut url = self.replace_env_keys_by_value(&modified_request.url);

		for (key, value) in path_params {
			url = url.replace(key, value);
		}

		let url = if params.is_empty() {
			Url::parse(&url)
		} else {
			// this adds extra "?" when params is empty
			Url::parse_with_params(&url, query_params)
		};

		let url = match url {
			Ok(url) => url,
			Err(_) => return Err(PrepareRequestError::InvalidUrl),
		};

		let url_path = url.path().to_owned();

		/* REQUEST */

		let method = match &modified_request.protocol {
			Protocol::HttpRequest(http_request) => http_request.method.to_reqwest(),
			Protocol::WsRequest(_) => reqwest::Method::GET,
		};

		let mut request_builder = client.request(method, url);

		/* AUTH */

		match &modified_request.auth {
			Auth::NoAuth => {}
			Auth::BasicAuth(BasicAuth { username, password }) => {
				let username = self.replace_env_keys_by_value(username);
				let password = self.replace_env_keys_by_value(password);

				request_builder = request_builder.basic_auth(username, Some(password));
			}
			Auth::BearerToken(BearerToken {
				token: bearer_token,
			}) => {
				let bearer_token = self.replace_env_keys_by_value(bearer_token);

				request_builder = request_builder.bearer_auth(bearer_token);
			}
			Auth::JwtToken(JwtToken {
				algorithm,
				secret_type,
				secret,
				payload,
			}) => {
				let secret = self.replace_env_keys_by_value(secret);
				let payload = self.replace_env_keys_by_value(payload);

				let bearer_token = jwt_do_jaat(algorithm, secret_type, secret, payload)?;
				request_builder = request_builder.bearer_auth(bearer_token);
			}
			Auth::Digest(Digest {
				username,
				password,
				domains,
				realm,
				nonce,
				opaque,
				stale,
				algorithm,
				qop,
				user_hash,
				charset,
				..
			}) => {
				let digest = request.auth.get_digest_mut();
				digest.nc += 1;

				let digest_header = digest_to_authorization_header(
					username,
					password,
					&url_path,
					domains.clone(),
					realm.clone(),
					nonce.clone(),
					opaque.clone(),
					*stale,
					algorithm,
					qop,
					*user_hash,
					charset,
					digest.nc,
				);

				request_builder = request_builder.header("Authorization", &digest_header);
			}
		}

		/* BODY */

		let mut pending_file: Option<PathBuf> = None;

		if let Protocol::HttpRequest(http_request) = &modified_request.protocol {
			match &http_request.body {
				NoBody => {}
				Multipart(form_data) => {
					let mut multipart = reqwest::multipart::Form::new();

					for form_data in form_data {
						let key = self.replace_env_keys_by_value(&form_data.data.0);
						let value = self.replace_env_keys_by_value(&form_data.data.1);

						// If the value starts with the file prefix, then it is supposed to be a file
						if let Some(file_path) = value.strip_prefix(FILE_VALUE_PREFIX) {
							let path = PathBuf::from(file_path);

							match get_file_content_with_name(path) {
								Ok((file_content, file_name)) => {
									let part = Part::bytes(file_content).file_name(file_name);
									multipart = multipart.part(key, part);
								}
								Err(_) => {
									return Err(PrepareRequestError::CouldNotOpenFile);
								}
							}
						} else {
							multipart = multipart.text(key, value);
						}
					}

					request_builder = request_builder.multipart(multipart);
				}
				Form(form_data) => {
					let form = self.key_value_vec_to_tuple_vec(form_data);

					request_builder = request_builder.form(&form);
				}
				File(file_path) => {
					let file_path_with_env_values = self.replace_env_keys_by_value(file_path);
					pending_file = Some(PathBuf::from(file_path_with_env_values));
				}
				Raw(body) | Json(body) | Xml(body) | Html(body) | Javascript(body) => {
					let body_with_env_values = self.replace_env_keys_by_value(body);
					request_builder = request_builder.body(body_with_env_values);
				}
			};
		}

		/* HEADERS */

		for header in &modified_request.headers {
			if !header.enabled {
				continue;
			}

			let header_name = self.replace_env_keys_by_value(&header.data.0);
			let header_value = self.replace_env_keys_by_value(&header.data.1);

			request_builder = request_builder.header(header_name, header_value);
		}

		trace!("Request prepared");

		Ok(PreparedRequest {
			builder: request_builder,
			pending_file,
		})
	}

	/// Finalize a [`PreparedRequest`] by opening any pending file body
	/// asynchronously and attaching it to the request builder.
	pub async fn finalize_prepared_request(
		prepared: PreparedRequest,
	) -> Result<reqwest_middleware::RequestBuilder, PrepareRequestError> {
		match prepared.pending_file {
			None => Ok(prepared.builder),
			Some(path) => match tokio::fs::File::open(path).await {
				Ok(file) => Ok(prepared.builder.body(file)),
				Err(_) => Err(PrepareRequestError::CouldNotOpenFile),
			},
		}
	}

	pub fn handle_pre_request_script(
		&self,
		request: &mut Request,
		env: Option<Arc<RwLock<Environment>>>,
	) -> anyhow::Result<Request, PrepareRequestError> {
		match &request.scripts.pre_request_script {
			None => {
				request.console_output.pre_request_output = None;
				Ok(request.clone())
			}
			Some(pre_request_script) => {
				let env_values = match &env {
					None => None,
					Some(local_env) => {
						let env = local_env.read();
						Some(env.values.clone())
					}
				};

				let (result_request, env_variables, console_output) =
					execute_pre_request_script(pre_request_script, request, env_values);

				match &env {
					None => {}
					Some(local_env) => match env_variables {
						None => {}
						Some(env_variables) => {
							let mut env = local_env.write();
							env.values = env_variables;
							save_environment_to_file(&env);
						}
					},
				}

				request.console_output.pre_request_output = Some(console_output);

				match result_request {
					None => Err(PrepareRequestError::PreRequestScript),
					Some(request) => Ok(request),
				}
			}
		}
	}

	pub fn handle_post_request_script(
		request: &Request,
		response: RequestResponse,
		env: &Option<Arc<RwLock<Environment>>>,
	) -> anyhow::Result<(RequestResponse, Option<String>), RequestResponseError> {
		match &request.scripts.post_request_script {
			None => Ok((response, None)),
			Some(post_request_script) => {
				let env_values = match &env {
					None => None,
					Some(env) => {
						let env = env.read();
						Some(env.values.clone())
					}
				};

				let (result_response, env_variables, result_console_output) =
					execute_post_request_script(post_request_script, &response, env_values);

				match env {
					None => {}
					Some(env) => match env_variables {
						None => {}
						Some(env_variables) => {
							let mut env = env.write();
							env.values = env_variables;
							save_environment_to_file(&env);
						}
					},
				}

				match result_response {
					None => Err(PostRequestScript),
					Some(result_response) => Ok((result_response, Some(result_console_output))),
				}
			}
		}
	}
}

pub fn get_file_content_with_name(path: PathBuf) -> std::io::Result<(Vec<u8>, String)> {
	let mut buffer: Vec<u8> = vec![];
	let mut file = std::fs::File::open(path.clone())?;

	file.read_to_end(&mut buffer)?;

	let file_name = path
		.file_name()
		.and_then(|n| n.to_str())
		.unwrap_or("file")
		.to_string();

	Ok((buffer, file_name))
}
