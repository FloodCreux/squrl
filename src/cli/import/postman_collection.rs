use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use clap::ValueEnum;
use parking_lot::RwLock;
use rayon::prelude::*;

use parse_postman_collection::v2_1_0::{
	AuthType, Body, FormParameterSrcUnion, HeaderUnion, Host, Items, Language, Mode, RequestClass,
	RequestUnion, Url,
};
use thiserror::Error;

use crate::cli::args::ARGS;
use crate::models::auth::auth::Auth;
use crate::models::auth::basic::BasicAuth;
use crate::models::auth::bearer_token::BearerToken;
use crate::models::auth::digest::{Digest, DigestAlgorithm, DigestCharset, DigestError, DigestQop};
use crate::models::auth::jwt::{JwtAlgorithm, JwtSecretType, JwtToken};
use crate::models::collection::{Collection, CollectionFileFormat};
use crate::models::protocol::http::body::ContentType;
use crate::models::protocol::http::http::HttpRequest;
use crate::models::protocol::http::method::Method;
use crate::models::protocol::protocol::Protocol;
use crate::models::request::{DEFAULT_HEADERS, KeyValue, Request};
use crate::models::settings::{RequestSettings, Setting};

#[derive(Error, Debug)]
pub enum ImportPostmanError {
	#[error("Could not parse Postman collection \"{0}\"\n\t{1}")]
	CouldNotParseCollection(String, String),
	#[error("Collection \"{0}\" already exists")]
	CollectionAlreadyExists(String),
	#[error("Unknown method \"{0}\"")]
	UnknownMethod(String),
	#[error("{0}")]
	AuthError(String),
}

pub fn recursive_has_requests(
	item: &mut Items,
	collections: &mut Vec<Collection>,
	nesting_prefix: &mut String,
	depth_level: &mut u16,
	max_depth: u16,
	file_format: CollectionFileFormat,
) -> anyhow::Result<Option<Arc<RwLock<Request>>>> {
	if is_folder(item) {
		let mut requests: Vec<Arc<RwLock<Request>>> = vec![];

		let mut folder_name = item
			.clone()
			.name
			.expect("postman folder should have a name");
		folder_name = folder_name.replace("/", "-");
		folder_name = folder_name.replace("\\", "-");
		folder_name = folder_name.trim().to_string();

		let collection_name = format!("{nesting_prefix}{folder_name}");

		*depth_level += 1;

		if *depth_level == max_depth {
			println!("\tMet max depth level");
			requests = recursive_get_requests(item)?;
		} else {
			nesting_prefix.push_str(&format!("{folder_name} "));

			let mut has_sub_folders = false;

			for mut sub_item in item
				.item
				.clone()
				.expect("postman folder should have sub-items")
			{
				if let Some(request) = recursive_has_requests(
					&mut sub_item,
					collections,
					nesting_prefix,
					depth_level,
					max_depth,
					file_format,
				)? {
					requests.push(request);
				} else {
					has_sub_folders = true;
				}
			}

			if has_sub_folders {
				nesting_prefix.clear();
			}
		}

		if !requests.is_empty() {
			println!("\tFound collection \"{}\"", collection_name);

			let collection = Collection {
				name: collection_name.clone(),
				last_position: Some(collections.len() - 1),
				folders: vec![],
				requests,
				path: ARGS
					.directory
					.as_ref()
					.expect("--directory argument is required")
					.join(format!("{}.{}", collection_name, file_format)),
				file_format,
			};

			collections.push(collection);
			*depth_level -= 1;
		}

		Ok(None)
	} else {
		Ok(Some(Arc::new(RwLock::new(parse_request(item.clone())?))))
	}
}

pub fn recursive_get_requests(item: &mut Items) -> anyhow::Result<Vec<Arc<RwLock<Request>>>> {
	if let Some(items) = &mut item.item {
		let mut requests: Vec<Arc<RwLock<Request>>> = vec![];

		for item in items {
			requests.extend(recursive_get_requests(item)?);
		}

		Ok(requests)
	} else {
		Ok(vec![Arc::new(RwLock::new(parse_request(item.clone())?))])
	}
}

pub fn is_folder(folder: &Items) -> bool {
	folder.item.is_some()
}

pub fn parse_request(item: Items) -> anyhow::Result<Request> {
	let item_name = item.name.clone().expect("postman item should have a name");

	println!("\t\tFound request \"{}\"", item_name);

	let mut request = Request {
		name: item_name,
		protocol: Protocol::HttpRequest(HttpRequest::default()),
		..Default::default()
	};

	request.scripts.pre_request_script = retrieve_request_scripts(&item);

	/* SETTINGS */

	// TODO: update parse_postman_collection to handle "protocolProfileBehavior"
	match retrieve_settings(&item) {
		None => {}
		Some(request_settings) => request.settings = request_settings,
	}

	/* REQUEST */

	let item_request = item.request.expect("postman item should have a request");

	match &item_request {
		RequestUnion::RequestClass(request_class) => {
			/* URL */

			if let Some(url) = &request_class.url {
				match url {
					Url::String(url) => request.url = url.to_string(),
					Url::UrlClass(url_class) => {
						request.url = url_class
							.raw
							.clone()
							.expect("postman URL should have a raw value")
					}
				}
			}

			/* QUERY PARAMS */

			match retrieve_query_params(request_class) {
				None => {}
				Some(query_params) => request.params = query_params,
			}

			/* METHOD */

			if let Some(method) = &request_class.method {
				let http_request = request.get_http_request_mut()?;
				http_request.method = match Method::from_str(method) {
					Ok(method) => method,
					Err(_) => {
						return Err(anyhow!(ImportPostmanError::UnknownMethod(method.clone())));
					}
				};
			}

			/* AUTH */

			match retrieve_auth(request_class) {
				None => {}
				Some(auth) => match auth {
					Ok(auth) => request.auth = auth,
					Err(error) => {
						return Err(anyhow!(ImportPostmanError::AuthError(error.to_string())));
					}
				},
			}

			/* HEADERS */

			match retrieve_headers(request_class) {
				None => {}
				Some(headers) => request.headers = headers,
			}

			/* BODY */

			match retrieve_body(request_class) {
				None => {}
				Some(body) => {
					match &body {
						ContentType::Multipart(_) => {} // TODO: Not handled yet
						body_type => {
							let content_type = body_type.to_content_type().clone();
							request.modify_or_create_header("content-type", &content_type);
						}
					}

					let http_request = request.get_http_request_mut()?;
					http_request.body = body;
				}
			}
		}
		RequestUnion::String(_) => {}
	}

	Ok(request)
}

pub fn retrieve_query_params(request_class: &RequestClass) -> Option<Vec<KeyValue>> {
	let url = request_class.url.clone()?;

	match url {
		Url::String(_) => None,
		Url::UrlClass(url_class) => {
			let mut query_params: Vec<KeyValue> = vec![];

			for query_param in url_class.query? {
				query_params.push(KeyValue {
					enabled: !query_param.disabled.unwrap_or(false), // Set default to enabled
					data: (query_param.key?, query_param.value?),
				})
			}

			Some(query_params)
		}
	}
}

pub fn retrieve_body(request_class: &RequestClass) -> Option<ContentType> {
	let body = request_class.body.clone()?;

	match body {
		Body::String(body_as_raw) => Some(ContentType::Raw(body_as_raw)),
		Body::BodyClass(body) => {
			let body_mode = body.mode?;

			match body_mode {
				Mode::Raw => {
					let body_as_raw = body.raw?;

					if let Some(options) = body.options {
						let language = options.raw?.language?;

						let request_body = match language {
							Language::Html => ContentType::Html(body_as_raw),
							Language::Json => ContentType::Json(body_as_raw),
							Language::Text => ContentType::Raw(body_as_raw),
							Language::Xml => ContentType::Xml(body_as_raw),
							Language::Javascript => ContentType::Javascript(body_as_raw),
						};

						Some(request_body)
					} else {
						Some(ContentType::Raw(body_as_raw))
					}
				}
				Mode::File => {
					let file = body.file?;
					let file_path = file.src?;

					Some(ContentType::File(file_path))
				}
				Mode::Formdata => {
					let form_data = body.formdata?;

					let mut multipart: Vec<KeyValue> = vec![];

					for param in form_data {
						let param_type = param.form_parameter_type?;

						let key_value = match param_type.as_str() {
							"text" => KeyValue {
								enabled: true,
								data: (param.key, param.value.unwrap_or(String::new())),
							},
							"file" => {
								let file = match param.src? {
									FormParameterSrcUnion::File(file) => file,
									// If there are many files, tries to get the first one
									FormParameterSrcUnion::Files(files) => {
										files.first()?.to_string()
									}
								};

								KeyValue {
									enabled: true,
									data: (param.key, format!("!!{file}")),
								}
							}
							param_type => {
								println!("\t\t\tUnknown Multipart form type \"{param_type}\"");
								return None;
							}
						};

						multipart.push(key_value);
					}

					Some(ContentType::Multipart(multipart))
				}
				Mode::Urlencoded => {
					let form_data = body.urlencoded?;

					let mut url_encoded: Vec<KeyValue> = vec![];

					for param in form_data {
						let value = param.value.unwrap_or(String::new());
						let is_disabled = param.disabled.unwrap_or(false);

						let key_value = KeyValue {
							enabled: !is_disabled,
							data: (param.key, value),
						};

						url_encoded.push(key_value);
					}

					Some(ContentType::Form(url_encoded))
				}
			}
		}
	}
}

pub fn retrieve_auth(request_class: &RequestClass) -> Option<anyhow::Result<Auth>> {
	let auth = request_class.auth.clone()?;

	match auth.auth_type {
		AuthType::Basic => {
			let basic_attributes = auth.basic?;

			let mut username = String::new();
			let mut password = String::new();

			for basic_attribute in basic_attributes {
				match basic_attribute.key.as_str() {
					"username" => {
						username = basic_attribute
							.value
							.expect("basic auth attribute should have a value")
							.as_str()?
							.to_string()
					}
					"password" => {
						password = basic_attribute
							.value
							.expect("basic auth attribute should have a value")
							.as_str()?
							.to_string()
					}
					_ => {}
				}
			}

			Some(Ok(Auth::BasicAuth(BasicAuth { username, password })))
		}
		AuthType::Bearer => {
			let bearer_token_attributes = auth.bearer?;

			let mut bearer_token = String::new();

			for bearer_token_attribute in bearer_token_attributes {
				if bearer_token_attribute.key.as_str() == "token" {
					bearer_token = bearer_token_attribute
						.value
						.expect("bearer token attribute should have a value")
						.as_str()?
						.to_string()
				}
			}

			Some(Ok(Auth::BearerToken(BearerToken {
				token: bearer_token,
			})))
		}
		AuthType::Jwt => {
			let jwt_attributes = auth.jwt?;

			let mut algorithm = String::new();
			let mut secret = String::new();
			let mut payload = String::new();

			let mut is_secret_base64 = false;

			for jwt_attribute in jwt_attributes {
				match jwt_attribute.key.as_str() {
					"algorithm" => {
						algorithm = jwt_attribute
							.value
							.expect("JWT attribute should have a value")
							.as_str()?
							.to_string()
					}
					"secret" => {
						secret = jwt_attribute
							.value
							.expect("JWT attribute should have a value")
							.as_str()?
							.to_string()
					}
					"payload" => {
						payload = jwt_attribute
							.value
							.expect("JWT attribute should have a value")
							.as_str()?
							.to_string()
					}
					"isSecretBase64Encoded" => {
						is_secret_base64 = jwt_attribute
							.value
							.expect("JWT attribute should have a value")
							.as_bool()?
					}
					_ => {}
				}
			}

			let algorithm =
				JwtAlgorithm::from_str(&algorithm, true).expect("JWT algorithm should be valid");
			let mut secret_type = algorithm.default_secret_type();

			match algorithm {
				JwtAlgorithm::HS256 | JwtAlgorithm::HS384 | JwtAlgorithm::HS512
					if is_secret_base64 =>
				{
					secret_type = JwtSecretType::Base64
				}
				_ => {}
			}

			Some(Ok(Auth::JwtToken(JwtToken {
				algorithm,
				secret_type,
				secret,
				payload,
			})))
		}
		AuthType::Digest => {
			let digest_attributes = auth.digest?;

			let mut username = String::new();
			let mut password = String::new();
			let mut realm = String::new();
			let mut nonce = String::new();
			let mut opaque = String::new();
			let mut algorithm = DigestAlgorithm::default();
			let mut qop = DigestQop::default();

			for digest_attribute in digest_attributes {
				let value = digest_attribute
					.value
					.expect("digest attribute should have a value")
					.as_str()?
					.to_string();

				match digest_attribute.key.as_str() {
					"username" => username = value,
					"password" => password = value,
					"realm" => realm = value,
					"nonce" => nonce = value,
					"opaque" => opaque = value,
					"algorithm" => match digest_auth::Algorithm::from_str(&value) {
						Ok(new_algorithm) => {
							algorithm = DigestAlgorithm::from_digest_auth_algorithm(new_algorithm)
						}
						Err(_) => return Some(Err(anyhow!(DigestError::InvalidAlgorithm(value)))),
					},
					"qop" => match digest_auth::Qop::from_str(&value) {
						Ok(new_qop) => qop = DigestQop::from_digest_auth_qop(new_qop),
						Err(_) => return Some(Err(anyhow!(DigestError::InvalidAlgorithm(value)))),
					},
					_ => {}
				}
			}

			Some(Ok(Auth::Digest(Digest {
				username,
				password,
				domains: String::new(),
				realm,
				nonce,
				opaque,
				stale: false,
				algorithm,
				qop,
				user_hash: false,
				charset: DigestCharset::default(),
				nc: 0,
			})))
		}
		AuthType::Awsv4 => Some(Ok(Auth::NoAuth)),
		AuthType::Hawk => Some(Ok(Auth::NoAuth)),
		AuthType::Noauth => Some(Ok(Auth::NoAuth)),
		AuthType::Ntlm => Some(Ok(Auth::NoAuth)),
		AuthType::Oauth1 => Some(Ok(Auth::NoAuth)),
		AuthType::Oauth2 => Some(Ok(Auth::NoAuth)),
	}
}

pub fn retrieve_headers(request_class: &RequestClass) -> Option<Vec<KeyValue>> {
	let headers = request_class.header.clone()?;

	let mut headers_to_return: Vec<KeyValue> = DEFAULT_HEADERS.clone();

	match headers {
		HeaderUnion::HeaderArray(headers) => {
			for header in headers {
				headers_to_return.push(KeyValue {
					enabled: !header.disabled.unwrap_or(false),
					data: (header.key, header.value),
				})
			}

			Some(headers_to_return)
		}
		HeaderUnion::String(_) => None,
	}
}

pub fn retrieve_request_scripts(item: &Items) -> Option<String> {
	let events = item.event.clone()?;

	for event in events {
		if event.listen == "prerequest" {
			let script = event.script?;
			match script.exec? {
				Host::String(_) => {}
				Host::StringArray(exec) => {
					let script: String = exec
						.par_iter()
						.map(|line| line.replace("pm.", "") + "\n")
						.collect();

					return Some(script);
				}
			}
		}
	}

	None
}

pub fn retrieve_settings(item: &Items) -> Option<RequestSettings> {
	let protocol_profile_behavior = item.protocol_profile_behavior.clone()?;

	let mut settings = RequestSettings::default();

	if let Some(follow_redirects) = protocol_profile_behavior.follow_redirects {
		settings.allow_redirects = Setting::Bool(follow_redirects);
	}

	if let Some(disable_cookies) = protocol_profile_behavior.disable_cookies {
		settings.store_received_cookies = Setting::Bool(!disable_cookies);
	}

	Some(settings)
}

#[cfg(test)]
mod tests {
	use super::*;
	use parse_postman_collection::v2_1_0::{
		Auth as PostmanAuth, AuthAttribute, BodyClass, Event, File as PostmanFile, FormParameter,
		Header, Language, Mode, Options, ProtocolProfileBehavior, QueryParam, Raw, RequestClass,
		RequestUnion, Script, Url, UrlClass, UrlEncodedParameter,
	};

	/// Helper to build a minimal Items leaf (request, not folder)
	fn make_item(name: &str, request_class: RequestClass) -> Items {
		Items {
			description: None,
			event: None,
			id: None,
			name: Some(name.to_string()),
			protocol_profile_behavior: None,
			request: Some(RequestUnion::RequestClass(request_class)),
			response: None,
			variable: None,
			auth: None,
			item: None,
		}
	}

	/// Helper to build a minimal RequestClass
	fn make_request_class(method: &str, url: &str) -> RequestClass {
		RequestClass {
			auth: None,
			body: None,
			certificate: None,
			description: None,
			header: None,
			method: Some(method.to_string()),
			proxy: None,
			url: Some(Url::String(url.to_string())),
		}
	}

	// ── is_folder ────────────────────────────────────────────────

	#[test]
	fn is_folder_returns_true_when_item_has_sub_items() {
		let folder = Items {
			description: None,
			event: None,
			id: None,
			name: Some("folder".to_string()),
			protocol_profile_behavior: None,
			request: None,
			response: None,
			variable: None,
			auth: None,
			item: Some(vec![]),
		};
		assert!(is_folder(&folder));
	}

	#[test]
	fn is_folder_returns_false_for_leaf_item() {
		let item = make_item("request", make_request_class("GET", "https://example.com"));
		assert!(!is_folder(&item));
	}

	// ── parse_request ────────────────────────────────────────────

	#[test]
	fn parse_request_basic_get() {
		let item = make_item(
			"Get Users",
			make_request_class("GET", "https://api.test.com/users"),
		);
		let request = parse_request(item).unwrap();

		assert_eq!(request.name, "Get Users");
		assert_eq!(request.url, "https://api.test.com/users");
		let http = request.get_http_request().unwrap();
		assert!(matches!(http.method, Method::GET));
	}

	#[test]
	fn parse_request_post_method() {
		let item = make_item(
			"Create User",
			make_request_class("POST", "https://api.test.com/users"),
		);
		let request = parse_request(item).unwrap();

		let http = request.get_http_request().unwrap();
		assert!(matches!(http.method, Method::POST));
	}

	#[test]
	fn parse_request_with_url_class() {
		let rc = RequestClass {
			auth: None,
			body: None,
			certificate: None,
			description: None,
			header: None,
			method: Some("GET".to_string()),
			proxy: None,
			url: Some(Url::UrlClass(UrlClass {
				hash: None,
				host: None,
				path: None,
				port: None,
				protocol: None,
				query: None,
				raw: Some("https://example.com/api".to_string()),
				variable: None,
			})),
		};
		let item = make_item("URL Class Test", rc);
		let request = parse_request(item).unwrap();

		assert_eq!(request.url, "https://example.com/api");
	}

	#[test]
	fn parse_request_unknown_method_returns_error() {
		let item = make_item(
			"Bad Method",
			make_request_class("FOOBAR", "https://example.com"),
		);
		let result = parse_request(item);
		assert!(result.is_err());
		assert!(result.unwrap_err().to_string().contains("FOOBAR"));
	}

	// ── retrieve_query_params ────────────────────────────────────

	#[test]
	fn retrieve_query_params_from_url_class() {
		let rc = RequestClass {
			auth: None,
			body: None,
			certificate: None,
			description: None,
			header: None,
			method: Some("GET".to_string()),
			proxy: None,
			url: Some(Url::UrlClass(UrlClass {
				hash: None,
				host: None,
				path: None,
				port: None,
				protocol: None,
				query: Some(vec![
					QueryParam {
						description: None,
						disabled: None,
						key: Some("page".to_string()),
						value: Some("1".to_string()),
					},
					QueryParam {
						description: None,
						disabled: Some(true),
						key: Some("limit".to_string()),
						value: Some("10".to_string()),
					},
				]),
				raw: Some("https://example.com".to_string()),
				variable: None,
			})),
		};

		let params = retrieve_query_params(&rc).unwrap();
		assert_eq!(params.len(), 2);
		assert!(params[0].enabled); // disabled=None defaults to enabled
		assert_eq!(params[0].data.0, "page");
		assert_eq!(params[0].data.1, "1");
		assert!(!params[1].enabled); // disabled=true
		assert_eq!(params[1].data.0, "limit");
	}

	#[test]
	fn retrieve_query_params_from_string_url_returns_none() {
		let rc = RequestClass {
			auth: None,
			body: None,
			certificate: None,
			description: None,
			header: None,
			method: Some("GET".to_string()),
			proxy: None,
			url: Some(Url::String("https://example.com?page=1".to_string())),
		};

		assert!(retrieve_query_params(&rc).is_none());
	}

	#[test]
	fn retrieve_query_params_no_url_returns_none() {
		let rc = RequestClass {
			auth: None,
			body: None,
			certificate: None,
			description: None,
			header: None,
			method: Some("GET".to_string()),
			proxy: None,
			url: None,
		};

		assert!(retrieve_query_params(&rc).is_none());
	}

	// ── retrieve_body ────────────────────────────────────────────

	#[test]
	fn retrieve_body_raw_json() {
		let rc = RequestClass {
			auth: None,
			body: Some(Body::BodyClass(BodyClass {
				disabled: None,
				file: None,
				formdata: None,
				options: Some(Options {
					raw: Some(Raw {
						language: Some(Language::Json),
					}),
				}),
				mode: Some(Mode::Raw),
				raw: Some(r#"{"key": "value"}"#.to_string()),
				urlencoded: None,
			})),
			certificate: None,
			description: None,
			header: None,
			method: Some("POST".to_string()),
			proxy: None,
			url: None,
		};

		let body = retrieve_body(&rc).unwrap();
		assert!(matches!(body, ContentType::Json(ref s) if s == r#"{"key": "value"}"#));
	}

	#[test]
	fn retrieve_body_raw_xml() {
		let rc = RequestClass {
			auth: None,
			body: Some(Body::BodyClass(BodyClass {
				disabled: None,
				file: None,
				formdata: None,
				options: Some(Options {
					raw: Some(Raw {
						language: Some(Language::Xml),
					}),
				}),
				mode: Some(Mode::Raw),
				raw: Some("<root/>".to_string()),
				urlencoded: None,
			})),
			certificate: None,
			description: None,
			header: None,
			method: Some("POST".to_string()),
			proxy: None,
			url: None,
		};

		let body = retrieve_body(&rc).unwrap();
		assert!(matches!(body, ContentType::Xml(ref s) if s == "<root/>"));
	}

	#[test]
	fn retrieve_body_raw_html() {
		let rc = RequestClass {
			auth: None,
			body: Some(Body::BodyClass(BodyClass {
				disabled: None,
				file: None,
				formdata: None,
				options: Some(Options {
					raw: Some(Raw {
						language: Some(Language::Html),
					}),
				}),
				mode: Some(Mode::Raw),
				raw: Some("<html></html>".to_string()),
				urlencoded: None,
			})),
			certificate: None,
			description: None,
			header: None,
			method: Some("POST".to_string()),
			proxy: None,
			url: None,
		};

		let body = retrieve_body(&rc).unwrap();
		assert!(matches!(body, ContentType::Html(ref s) if s == "<html></html>"));
	}

	#[test]
	fn retrieve_body_raw_text() {
		let rc = RequestClass {
			auth: None,
			body: Some(Body::BodyClass(BodyClass {
				disabled: None,
				file: None,
				formdata: None,
				options: Some(Options {
					raw: Some(Raw {
						language: Some(Language::Text),
					}),
				}),
				mode: Some(Mode::Raw),
				raw: Some("plain text".to_string()),
				urlencoded: None,
			})),
			certificate: None,
			description: None,
			header: None,
			method: Some("POST".to_string()),
			proxy: None,
			url: None,
		};

		let body = retrieve_body(&rc).unwrap();
		assert!(matches!(body, ContentType::Raw(ref s) if s == "plain text"));
	}

	#[test]
	fn retrieve_body_raw_javascript() {
		let rc = RequestClass {
			auth: None,
			body: Some(Body::BodyClass(BodyClass {
				disabled: None,
				file: None,
				formdata: None,
				options: Some(Options {
					raw: Some(Raw {
						language: Some(Language::Javascript),
					}),
				}),
				mode: Some(Mode::Raw),
				raw: Some("console.log('hi');".to_string()),
				urlencoded: None,
			})),
			certificate: None,
			description: None,
			header: None,
			method: Some("POST".to_string()),
			proxy: None,
			url: None,
		};

		let body = retrieve_body(&rc).unwrap();
		assert!(matches!(body, ContentType::Javascript(_)));
	}

	#[test]
	fn retrieve_body_raw_without_options_defaults_to_raw() {
		let rc = RequestClass {
			auth: None,
			body: Some(Body::BodyClass(BodyClass {
				disabled: None,
				file: None,
				formdata: None,
				options: None,
				mode: Some(Mode::Raw),
				raw: Some("raw content".to_string()),
				urlencoded: None,
			})),
			certificate: None,
			description: None,
			header: None,
			method: Some("POST".to_string()),
			proxy: None,
			url: None,
		};

		let body = retrieve_body(&rc).unwrap();
		assert!(matches!(body, ContentType::Raw(ref s) if s == "raw content"));
	}

	#[test]
	fn retrieve_body_file_mode() {
		let rc = RequestClass {
			auth: None,
			body: Some(Body::BodyClass(BodyClass {
				disabled: None,
				file: Some(PostmanFile {
					content: None,
					src: Some("/path/to/file.txt".to_string()),
				}),
				formdata: None,
				options: None,
				mode: Some(Mode::File),
				raw: None,
				urlencoded: None,
			})),
			certificate: None,
			description: None,
			header: None,
			method: Some("POST".to_string()),
			proxy: None,
			url: None,
		};

		let body = retrieve_body(&rc).unwrap();
		assert!(matches!(body, ContentType::File(ref s) if s == "/path/to/file.txt"));
	}

	#[test]
	fn retrieve_body_urlencoded() {
		let rc = RequestClass {
			auth: None,
			body: Some(Body::BodyClass(BodyClass {
				disabled: None,
				file: None,
				formdata: None,
				options: None,
				mode: Some(Mode::Urlencoded),
				raw: None,
				urlencoded: Some(vec![
					UrlEncodedParameter {
						description: None,
						disabled: None,
						key: "username".to_string(),
						value: Some("admin".to_string()),
					},
					UrlEncodedParameter {
						description: None,
						disabled: Some(true),
						key: "debug".to_string(),
						value: Some("true".to_string()),
					},
				]),
			})),
			certificate: None,
			description: None,
			header: None,
			method: Some("POST".to_string()),
			proxy: None,
			url: None,
		};

		let body = retrieve_body(&rc).unwrap();
		match body {
			ContentType::Form(params) => {
				assert_eq!(params.len(), 2);
				assert!(params[0].enabled);
				assert_eq!(params[0].data.0, "username");
				assert_eq!(params[0].data.1, "admin");
				assert!(!params[1].enabled);
			}
			_ => panic!("expected Form content type"),
		}
	}

	#[test]
	fn retrieve_body_formdata_text() {
		let rc = RequestClass {
			auth: None,
			body: Some(Body::BodyClass(BodyClass {
				disabled: None,
				file: None,
				formdata: Some(vec![FormParameter {
					content_type: None,
					description: None,
					disabled: None,
					key: "field".to_string(),
					form_parameter_type: Some("text".to_string()),
					value: Some("value".to_string()),
					src: None,
				}]),
				options: None,
				mode: Some(Mode::Formdata),
				raw: None,
				urlencoded: None,
			})),
			certificate: None,
			description: None,
			header: None,
			method: Some("POST".to_string()),
			proxy: None,
			url: None,
		};

		let body = retrieve_body(&rc).unwrap();
		match body {
			ContentType::Multipart(params) => {
				assert_eq!(params.len(), 1);
				assert_eq!(params[0].data.0, "field");
				assert_eq!(params[0].data.1, "value");
			}
			_ => panic!("expected Multipart content type"),
		}
	}

	#[test]
	fn retrieve_body_formdata_file() {
		let rc = RequestClass {
			auth: None,
			body: Some(Body::BodyClass(BodyClass {
				disabled: None,
				file: None,
				formdata: Some(vec![FormParameter {
					content_type: None,
					description: None,
					disabled: None,
					key: "upload".to_string(),
					form_parameter_type: Some("file".to_string()),
					value: None,
					src: Some(FormParameterSrcUnion::File("/path/to/file.png".to_string())),
				}]),
				options: None,
				mode: Some(Mode::Formdata),
				raw: None,
				urlencoded: None,
			})),
			certificate: None,
			description: None,
			header: None,
			method: Some("POST".to_string()),
			proxy: None,
			url: None,
		};

		let body = retrieve_body(&rc).unwrap();
		match body {
			ContentType::Multipart(params) => {
				assert_eq!(params.len(), 1);
				assert_eq!(params[0].data.0, "upload");
				assert!(params[0].data.1.contains("file.png"));
			}
			_ => panic!("expected Multipart content type"),
		}
	}

	#[test]
	fn retrieve_body_string_variant() {
		let rc = RequestClass {
			auth: None,
			body: Some(Body::String("raw body string".to_string())),
			certificate: None,
			description: None,
			header: None,
			method: Some("POST".to_string()),
			proxy: None,
			url: None,
		};

		let body = retrieve_body(&rc).unwrap();
		assert!(matches!(body, ContentType::Raw(ref s) if s == "raw body string"));
	}

	#[test]
	fn retrieve_body_none_returns_none() {
		let rc = RequestClass {
			auth: None,
			body: None,
			certificate: None,
			description: None,
			header: None,
			method: Some("GET".to_string()),
			proxy: None,
			url: None,
		};

		assert!(retrieve_body(&rc).is_none());
	}

	// ── retrieve_auth ────────────────────────────────────────────

	fn make_auth(auth_type: AuthType) -> PostmanAuth {
		PostmanAuth {
			awsv4: None,
			basic: None,
			bearer: None,
			jwt: None,
			digest: None,
			hawk: None,
			noauth: None,
			ntlm: None,
			oauth1: None,
			oauth2: None,
			auth_type,
		}
	}

	#[test]
	fn retrieve_auth_basic() {
		let mut postman_auth = make_auth(AuthType::Basic);
		postman_auth.basic = Some(vec![
			AuthAttribute {
				key: "username".to_string(),
				auth_type: None,
				value: Some(serde_json::Value::String("myuser".to_string())),
			},
			AuthAttribute {
				key: "password".to_string(),
				auth_type: None,
				value: Some(serde_json::Value::String("mypass".to_string())),
			},
		]);

		let rc = RequestClass {
			auth: Some(postman_auth),
			body: None,
			certificate: None,
			description: None,
			header: None,
			method: None,
			proxy: None,
			url: None,
		};

		let auth = retrieve_auth(&rc).unwrap().unwrap();
		match auth {
			Auth::BasicAuth(basic) => {
				assert_eq!(basic.username, "myuser");
				assert_eq!(basic.password, "mypass");
			}
			_ => panic!("expected BasicAuth"),
		}
	}

	#[test]
	fn retrieve_auth_bearer() {
		let mut postman_auth = make_auth(AuthType::Bearer);
		postman_auth.bearer = Some(vec![AuthAttribute {
			key: "token".to_string(),
			auth_type: None,
			value: Some(serde_json::Value::String("my-token-123".to_string())),
		}]);

		let rc = RequestClass {
			auth: Some(postman_auth),
			body: None,
			certificate: None,
			description: None,
			header: None,
			method: None,
			proxy: None,
			url: None,
		};

		let auth = retrieve_auth(&rc).unwrap().unwrap();
		match auth {
			Auth::BearerToken(bt) => assert_eq!(bt.token, "my-token-123"),
			_ => panic!("expected BearerToken"),
		}
	}

	#[test]
	fn retrieve_auth_noauth() {
		let postman_auth = make_auth(AuthType::Noauth);
		let rc = RequestClass {
			auth: Some(postman_auth),
			body: None,
			certificate: None,
			description: None,
			header: None,
			method: None,
			proxy: None,
			url: None,
		};

		let auth = retrieve_auth(&rc).unwrap().unwrap();
		assert!(matches!(auth, Auth::NoAuth));
	}

	#[test]
	fn retrieve_auth_unsupported_types_return_noauth() {
		for auth_type in [
			AuthType::Awsv4,
			AuthType::Hawk,
			AuthType::Ntlm,
			AuthType::Oauth1,
			AuthType::Oauth2,
		] {
			let postman_auth = make_auth(auth_type);
			let rc = RequestClass {
				auth: Some(postman_auth),
				body: None,
				certificate: None,
				description: None,
				header: None,
				method: None,
				proxy: None,
				url: None,
			};

			let auth = retrieve_auth(&rc).unwrap().unwrap();
			assert!(matches!(auth, Auth::NoAuth));
		}
	}

	#[test]
	fn retrieve_auth_none_returns_none() {
		let rc = RequestClass {
			auth: None,
			body: None,
			certificate: None,
			description: None,
			header: None,
			method: None,
			proxy: None,
			url: None,
		};

		assert!(retrieve_auth(&rc).is_none());
	}

	// ── retrieve_headers ─────────────────────────────────────────

	#[test]
	fn retrieve_headers_array() {
		let rc = RequestClass {
			auth: None,
			body: None,
			certificate: None,
			description: None,
			header: Some(HeaderUnion::HeaderArray(vec![
				Header {
					description: None,
					disabled: None,
					key: "X-Custom".to_string(),
					value: "custom-value".to_string(),
				},
				Header {
					description: None,
					disabled: Some(true),
					key: "X-Disabled".to_string(),
					value: "disabled-value".to_string(),
				},
			])),
			method: None,
			proxy: None,
			url: None,
		};

		let headers = retrieve_headers(&rc).unwrap();
		// Should include DEFAULT_HEADERS + 2 custom headers
		assert!(headers.len() >= 7); // 5 default + 2 custom
		let custom = headers.iter().find(|h| h.data.0 == "X-Custom").unwrap();
		assert!(custom.enabled);
		assert_eq!(custom.data.1, "custom-value");
		let disabled = headers.iter().find(|h| h.data.0 == "X-Disabled").unwrap();
		assert!(!disabled.enabled);
	}

	#[test]
	fn retrieve_headers_string_variant_returns_none() {
		let rc = RequestClass {
			auth: None,
			body: None,
			certificate: None,
			description: None,
			header: Some(HeaderUnion::String("Content-Type: text/plain".to_string())),
			method: None,
			proxy: None,
			url: None,
		};

		assert!(retrieve_headers(&rc).is_none());
	}

	#[test]
	fn retrieve_headers_none_returns_none() {
		let rc = RequestClass {
			auth: None,
			body: None,
			certificate: None,
			description: None,
			header: None,
			method: None,
			proxy: None,
			url: None,
		};

		assert!(retrieve_headers(&rc).is_none());
	}

	// ── retrieve_request_scripts ─────────────────────────────────

	#[test]
	fn retrieve_request_scripts_prerequest() {
		let item = Items {
			description: None,
			event: Some(vec![Event {
				disabled: None,
				id: None,
				listen: "prerequest".to_string(),
				script: Some(Script {
					exec: Some(Host::StringArray(vec![
						"pm.environment.set('key', 'value');".to_string(),
						"pm.console.log('hello');".to_string(),
					])),
					id: None,
					name: None,
					src: None,
					script_type: None,
				}),
			}]),
			id: None,
			name: Some("test".to_string()),
			protocol_profile_behavior: None,
			request: None,
			response: None,
			variable: None,
			auth: None,
			item: None,
		};

		let script = retrieve_request_scripts(&item).unwrap();
		// pm. prefix should be stripped
		assert!(script.contains("environment.set('key', 'value');"));
		assert!(script.contains("console.log('hello');"));
		assert!(!script.contains("pm."));
	}

	#[test]
	fn retrieve_request_scripts_no_events_returns_none() {
		let item = Items {
			description: None,
			event: None,
			id: None,
			name: Some("test".to_string()),
			protocol_profile_behavior: None,
			request: None,
			response: None,
			variable: None,
			auth: None,
			item: None,
		};

		assert!(retrieve_request_scripts(&item).is_none());
	}

	#[test]
	fn retrieve_request_scripts_ignores_test_events() {
		let item = Items {
			description: None,
			event: Some(vec![Event {
				disabled: None,
				id: None,
				listen: "test".to_string(), // not "prerequest"
				script: Some(Script {
					exec: Some(Host::StringArray(vec!["pm.test('check');".to_string()])),
					id: None,
					name: None,
					src: None,
					script_type: None,
				}),
			}]),
			id: None,
			name: Some("test".to_string()),
			protocol_profile_behavior: None,
			request: None,
			response: None,
			variable: None,
			auth: None,
			item: None,
		};

		assert!(retrieve_request_scripts(&item).is_none());
	}

	// ── retrieve_settings ────────────────────────────────────────

	#[test]
	fn retrieve_settings_follow_redirects_disabled() {
		let item = Items {
			description: None,
			event: None,
			id: None,
			name: Some("test".to_string()),
			protocol_profile_behavior: Some(ProtocolProfileBehavior {
				disable_body_pruning: None,
				follow_redirects: Some(false),
				disable_cookies: None,
			}),
			request: None,
			response: None,
			variable: None,
			auth: None,
			item: None,
		};

		let settings = retrieve_settings(&item).unwrap();
		assert_eq!(settings.allow_redirects.as_bool(), Some(false));
		// cookies should remain default (true)
		assert_eq!(settings.store_received_cookies.as_bool(), Some(true));
	}

	#[test]
	fn retrieve_settings_cookies_disabled() {
		let item = Items {
			description: None,
			event: None,
			id: None,
			name: Some("test".to_string()),
			protocol_profile_behavior: Some(ProtocolProfileBehavior {
				disable_body_pruning: None,
				follow_redirects: None,
				disable_cookies: Some(true),
			}),
			request: None,
			response: None,
			variable: None,
			auth: None,
			item: None,
		};

		let settings = retrieve_settings(&item).unwrap();
		// disable_cookies=true => store_received_cookies=false
		assert_eq!(settings.store_received_cookies.as_bool(), Some(false));
		// redirects should remain default (true)
		assert_eq!(settings.allow_redirects.as_bool(), Some(true));
	}

	#[test]
	fn retrieve_settings_both_set() {
		let item = Items {
			description: None,
			event: None,
			id: None,
			name: Some("test".to_string()),
			protocol_profile_behavior: Some(ProtocolProfileBehavior {
				disable_body_pruning: None,
				follow_redirects: Some(true),
				disable_cookies: Some(false),
			}),
			request: None,
			response: None,
			variable: None,
			auth: None,
			item: None,
		};

		let settings = retrieve_settings(&item).unwrap();
		assert_eq!(settings.allow_redirects.as_bool(), Some(true));
		assert_eq!(settings.store_received_cookies.as_bool(), Some(true));
	}

	#[test]
	fn retrieve_settings_none_returns_none() {
		let item = Items {
			description: None,
			event: None,
			id: None,
			name: Some("test".to_string()),
			protocol_profile_behavior: None,
			request: None,
			response: None,
			variable: None,
			auth: None,
			item: None,
		};

		assert!(retrieve_settings(&item).is_none());
	}

	// ── recursive_get_requests ───────────────────────────────────

	#[test]
	fn recursive_get_requests_single_leaf() {
		let mut item = make_item("Leaf", make_request_class("GET", "https://leaf.com"));
		let requests = recursive_get_requests(&mut item).unwrap();

		assert_eq!(requests.len(), 1);
		assert_eq!(requests[0].read().name, "Leaf");
	}

	#[test]
	fn recursive_get_requests_nested_folder() {
		let child1 = make_item("Child1", make_request_class("GET", "https://c1.com"));
		let child2 = make_item("Child2", make_request_class("POST", "https://c2.com"));

		let mut folder = Items {
			description: None,
			event: None,
			id: None,
			name: Some("Folder".to_string()),
			protocol_profile_behavior: None,
			request: None,
			response: None,
			variable: None,
			auth: None,
			item: Some(vec![child1, child2]),
		};

		let requests = recursive_get_requests(&mut folder).unwrap();
		assert_eq!(requests.len(), 2);
		assert_eq!(requests[0].read().name, "Child1");
		assert_eq!(requests[1].read().name, "Child2");
	}

	// ── parse_request with body sets content-type header ─────────

	#[test]
	fn parse_request_with_json_body_adds_content_type_header() {
		let rc = RequestClass {
			auth: None,
			body: Some(Body::BodyClass(BodyClass {
				disabled: None,
				file: None,
				formdata: None,
				options: Some(Options {
					raw: Some(Raw {
						language: Some(Language::Json),
					}),
				}),
				mode: Some(Mode::Raw),
				raw: Some(r#"{"test": true}"#.to_string()),
				urlencoded: None,
			})),
			certificate: None,
			description: None,
			header: None,
			method: Some("POST".to_string()),
			proxy: None,
			url: Some(Url::String("https://example.com".to_string())),
		};

		let item = make_item("JSON Request", rc);
		let request = parse_request(item).unwrap();

		let ct_header = request
			.headers
			.iter()
			.find(|h| h.data.0 == "content-type")
			.expect("should have content-type header");
		assert!(ct_header.data.1.contains("json"));
	}

	// ── parse_request with settings ──────────────────────────────

	#[test]
	fn parse_request_with_settings_from_protocol_profile() {
		let mut item = make_item(
			"Settings Test",
			make_request_class("GET", "https://example.com"),
		);
		item.protocol_profile_behavior = Some(ProtocolProfileBehavior {
			disable_body_pruning: None,
			follow_redirects: Some(false),
			disable_cookies: Some(true),
		});

		let request = parse_request(item).unwrap();
		assert_eq!(request.settings.allow_redirects.as_bool(), Some(false));
		assert_eq!(
			request.settings.store_received_cookies.as_bool(),
			Some(false)
		);
	}
}
