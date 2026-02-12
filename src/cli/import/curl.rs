use crate::cli::import::curl::ImportCurlError::{
	CouldNotParseCurl, CouldNotParseUrl, CouldNotReadFile, UnknownMethod,
};
use crate::models::auth::auth::Auth;
use crate::models::auth::basic::BasicAuth;
use crate::models::auth::bearer_token::BearerToken;
use crate::models::auth::digest::{
	Digest, DigestAlgorithm, DigestCharset, DigestQop, extract_www_authenticate_digest_data,
};
use crate::models::protocol::http::body::ContentType;
use crate::models::protocol::http::body::ContentType::NoBody;
use crate::models::protocol::http::http::HttpRequest;
use crate::models::protocol::http::method::Method;
use crate::models::protocol::protocol::Protocol;
use crate::models::request::{KeyValue, Request};
use anyhow::anyhow;
use parking_lot::RwLock;
use rayon::prelude::*;
use regex::Regex;
use reqwest::Url;
use reqwest::header::CONTENT_TYPE;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum ImportCurlError {
	#[error("Could not read cURL file\n\t{0}")]
	CouldNotReadFile(String),
	#[error("Could not parse cURL\n\t{0}")]
	CouldNotParseCurl(String),
	#[error("Could not parse URL\n\t{0}")]
	CouldNotParseUrl(String),
	#[error("Unknown method\n\t{0}")]
	UnknownMethod(String),
}

pub fn parse_requests_recursively(
	path: &PathBuf,
	recursive: bool,
	max_depth: u16,
) -> anyhow::Result<Vec<Arc<RwLock<Request>>>> {
	let max_depth: usize = match recursive {
		true => max_depth as usize,
		false => 1,
	};

	let mut requests: Vec<Arc<RwLock<Request>>> = vec![];
	let walker = WalkDir::new(path)
		.max_depth(max_depth)
		.into_iter()
		.filter_map(|e| e.ok());

	for entry in walker {
		if !entry.file_type().is_file() {
			continue;
		}

		// Will use the file name as the request name
		let file_name = entry
			.file_name()
			.to_str()
			.expect("file name should be valid UTF-8")
			.to_string();
		let request = parse_request(&entry.path().to_path_buf(), file_name)?;

		requests.push(request);
	}

	Ok(requests)
}

/// TODO: parse everything with regexes in order to handle everything
pub fn parse_request(path: &PathBuf, request_name: String) -> anyhow::Result<Arc<RwLock<Request>>> {
	let curl_stringed = match fs::read_to_string(path) {
		Ok(original_curl) => original_curl,
		Err(e) => return Err(anyhow!(CouldNotReadFile(e.to_string()))),
	};

	println!("\tRequest name: {}", request_name);

	let parsed_curl = match curl_parser::ParsedRequest::load(&curl_stringed, None::<String>) {
		Ok(parsed_curl) => parsed_curl,
		Err(e) => return Err(anyhow!(CouldNotParseCurl(e.to_string()))),
	};

	/* URL */

	// Parse the URL so we can transform it
	let mut curl_url = match Url::parse(&parsed_curl.url.to_string()) {
		Ok(url) => url,
		Err(e) => return Err(anyhow!(CouldNotParseUrl(e.to_string()))),
	};

	curl_url.set_query(None);
	let url = curl_url.to_string();

	/* QUERY PARAMS */

	let params = curl_url
		.query_pairs()
		.par_bridge()
		.map(|(k, v)| KeyValue {
			enabled: true,
			data: (k.to_string(), v.to_string()),
		})
		.collect();

	/* METHOD */

	let method = match Method::from_str(parsed_curl.method.as_str()) {
		Ok(method) => method,
		Err(e) => return Err(anyhow!(UnknownMethod(e.to_string()))),
	};

	/* HEADERS */

	let headers: Vec<KeyValue> = parsed_curl
		.headers
		.iter()
		.par_bridge()
		.filter(|(header_name, _)| header_name.as_str() != "authorization") // Exclude Authorization header, as that will be handled by the auth field
		.map(|(k, v)| KeyValue {
			enabled: true,
			data: (
				k.to_string(),
				v.to_str()
					.expect("header value should be valid UTF-8")
					.to_string(),
			),
		})
		.collect();

	/* AUTH */

	let basic_auth_regex = Regex::new(r#"(-u|--user) ["'](?<username>.*):(?<password>.*)["']"#)?;
	let digest_auth_regex =
		Regex::new(r#"--digest (-u|--user) ["'](?<username>.*):(?<password>.*)["']"#)?;

	let auth = match basic_auth_regex.captures(&curl_stringed) {
		None => {
			let authorization_header_value =
				parsed_curl
					.headers
					.iter()
					.par_bridge()
					.find_map_any(|(header_name, value)| {
						match header_name.as_str() == "authorization" {
							true => {
								Some(value.to_str().expect("header value should be valid UTF-8"))
							}
							false => None,
						}
					});

			let digest_credentials = match digest_auth_regex.captures(&curl_stringed) {
				None => None,
				Some(capture) => {
					let username = capture["username"].to_string();
					let password = capture["password"].to_string();

					Some((username, password))
				}
			};

			match authorization_header_value {
				None => match digest_credentials {
					None => Auth::NoAuth,
					Some((username, password)) => Auth::Digest(Digest {
						username,
						password,
						domains: String::new(),
						realm: String::new(),
						nonce: String::new(),
						opaque: String::new(),
						stale: false,
						algorithm: DigestAlgorithm::default(),
						qop: DigestQop::default(),
						user_hash: false,
						charset: DigestCharset::default(),
						nc: 0,
					}),
				},
				Some(authorization_header_value) => {
					if let Some(bearer_token) = authorization_header_value.strip_prefix("Bearer ") {
						Auth::BearerToken(BearerToken {
							token: bearer_token.to_string(),
						})
					} else if authorization_header_value.starts_with("Digest ") {
						let (username, password) = match digest_credentials {
							None => (String::new(), String::new()),
							Some((username, password)) => (username, password),
						};

						let (
							domains,
							realm,
							nonce,
							opaque,
							stale,
							algorithm,
							qop,
							user_hash,
							charset,
						) = extract_www_authenticate_digest_data(authorization_header_value)?;

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
							nc: 0,
						})
					} else {
						Auth::NoAuth
					}
				}
			}
		}
		Some(capture) => {
			let username = capture["username"].to_string();
			let password = capture["password"].to_string();

			Auth::BasicAuth(BasicAuth { username, password })
		}
	};

	/* BODY */

	let body;

	// TODO: does not support forms yet
	if !parsed_curl.body.is_empty() {
		let content_type_header = headers
			.par_iter()
			.find_any(|header| header.data.0 == CONTENT_TYPE.as_str());
		let body_stringed = parsed_curl.body.join("\n");

		if let Some(content_type) = content_type_header {
			body = ContentType::from_content_type(&content_type.data.1, body_stringed);
		} else {
			body = NoBody;
		}
	} else {
		body = NoBody;
	}

	let request = Request {
		name: request_name,
		url,
		params,
		headers,
		auth,
		protocol: Protocol::HttpRequest(HttpRequest { method, body }),
		..Default::default()
	};

	Ok(Arc::new(RwLock::new(request)))
}
