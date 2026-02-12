use crate::cli::import::http_file::ImportHttpFileError::{
	CouldNotParseMethod, CouldNotParseUrl, CouldNotReadFile, NoRequestsFound,
};
use crate::models::auth::auth::Auth;
use crate::models::auth::basic::BasicAuth;
use crate::models::auth::bearer_token::BearerToken;
use crate::models::protocol::http::body::ContentType;
use crate::models::protocol::http::body::ContentType::NoBody;
use crate::models::protocol::http::http::HttpRequest;
use crate::models::protocol::http::method::Method;
use crate::models::protocol::protocol::Protocol;
use crate::models::protocol::ws::ws::WsRequest;
use crate::models::request::{KeyValue, Request};
use anyhow::anyhow;
use parking_lot::RwLock;
use reqwest::Url;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum ImportHttpFileError {
	#[error("Could not read .http file\n\t{0}")]
	CouldNotReadFile(String),
	#[error("Could not parse HTTP method\n\t{0}")]
	CouldNotParseMethod(String),
	#[error("Could not parse URL\n\t{0}")]
	CouldNotParseUrl(String),
	#[error("No requests found in .http file")]
	NoRequestsFound,
}

pub fn parse_http_files_recursively(
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

		let file_path = entry.path().to_path_buf();

		if file_path.extension().is_some_and(|ext| ext == "http") {
			let parsed = parse_http_file(&file_path)?;
			requests.extend(parsed);
		}
	}

	Ok(requests)
}

pub fn parse_http_file(path: &PathBuf) -> anyhow::Result<Vec<Arc<RwLock<Request>>>> {
	let content = match fs::read_to_string(path) {
		Ok(content) => content,
		Err(e) => return Err(anyhow!(CouldNotReadFile(e.to_string()))),
	};

	parse_http_content(&content)
}

pub fn parse_http_content(content: &str) -> anyhow::Result<Vec<Arc<RwLock<Request>>>> {
	let lines: Vec<&str> = content.lines().collect();
	let mut requests: Vec<Arc<RwLock<Request>>> = vec![];

	let mut i = 0;
	while i < lines.len() {
		// Skip leading blank lines and comments
		let line = lines[i].trim();
		if line.is_empty() || is_comment(line) && !line.starts_with("###") {
			i += 1;
			continue;
		}

		// Check for request separator with optional name
		let request_name = if line.starts_with("###") {
			let name = line.trim_start_matches('#').trim().to_string();
			i += 1;
			if name.is_empty() { None } else { Some(name) }
		} else {
			None
		};

		// Skip blank lines and comments after ###
		while i < lines.len() {
			let l = lines[i].trim();
			if l.is_empty() || (is_comment(l) && !l.starts_with("###")) {
				i += 1;
			} else {
				break;
			}
		}

		if i >= lines.len() {
			break;
		}

		// Parse the request line: METHOD URL [HTTP/version]
		let request_line = lines[i].trim();

		// If we hit another separator, skip it
		if request_line.starts_with("###") {
			continue;
		}

		let parts: Vec<&str> = request_line.splitn(3, ' ').collect();
		if parts.len() < 2 {
			i += 1;
			continue;
		}

		let method_str = parts[0];
		let url_str = parts[1];

		let is_websocket = method_str == "WEBSOCKET";

		let method = if is_websocket {
			None
		} else {
			match Method::from_str(method_str) {
				Ok(method) => Some(method),
				Err(e) => return Err(anyhow!(CouldNotParseMethod(e.to_string()))),
			}
		};

		i += 1;

		// Parse headers (non-blank lines before the first blank line)
		let mut raw_headers: Vec<(String, String)> = vec![];
		while i < lines.len() {
			let l = lines[i].trim();

			// Stop at blank line, next request separator, or EOF
			if l.is_empty() || l.starts_with("###") {
				break;
			}

			// Skip comments within header section
			if is_comment(l) {
				i += 1;
				continue;
			}

			// Parse header: Name: Value
			if let Some(colon_pos) = l.find(':') {
				let header_name = l[..colon_pos].trim().to_string();
				let header_value = l[colon_pos + 1..].trim().to_string();
				raw_headers.push((header_name, header_value));
			}

			i += 1;
		}

		// Skip the blank line separating headers from body
		if i < lines.len() && lines[i].trim().is_empty() {
			i += 1;
		}

		// Parse body (everything until next ### or EOF)
		let mut body_lines: Vec<&str> = vec![];
		while i < lines.len() {
			let l = lines[i].trim();
			if l.starts_with("###") {
				break;
			}
			body_lines.push(lines[i]);
			i += 1;
		}

		// Trim trailing blank lines from body
		while body_lines.last().is_some_and(|l| l.trim().is_empty()) {
			body_lines.pop();
		}

		let body_string = body_lines.join("\n");

		// Process URL - extract query params
		let mut parsed_url = match Url::parse(url_str) {
			Ok(url) => url,
			Err(e) => return Err(anyhow!(CouldNotParseUrl(e.to_string()))),
		};

		let params: Vec<KeyValue> = parsed_url
			.query_pairs()
			.map(|(k, v)| KeyValue {
				enabled: true,
				data: (k.to_string(), v.to_string()),
			})
			.collect();

		parsed_url.set_query(None);
		let url = parsed_url.to_string();

		// Derive request name if not provided
		let name = match request_name {
			Some(n) => n,
			None => {
				let path = parsed_url.path();
				format!("{} {}", method_str, path)
			}
		};

		// Extract auth from Authorization header
		let auth = extract_auth_from_headers(&raw_headers);

		// Build headers (exclude Authorization since it's handled by auth)
		let headers: Vec<KeyValue> = raw_headers
			.iter()
			.filter(|(name, _)| name.to_lowercase() != "authorization")
			.map(|(k, v)| KeyValue {
				enabled: true,
				data: (k.clone(), v.clone()),
			})
			.collect();

		// Build protocol-specific request
		let protocol = if is_websocket {
			Protocol::WsRequest(WsRequest::default())
		} else {
			let method = method.unwrap();

			let body = if body_string.is_empty() {
				NoBody
			} else {
				let content_type_value = raw_headers
					.iter()
					.find(|(name, _)| name.to_lowercase() == "content-type")
					.map(|(_, v)| v.as_str());

				match content_type_value {
					Some(ct) => ContentType::from_content_type(ct, body_string),
					None => ContentType::Raw(body_string),
				}
			};

			Protocol::HttpRequest(HttpRequest { method, body })
		};

		let request = Request {
			name,
			url,
			params,
			headers,
			auth,
			protocol,
			..Default::default()
		};

		requests.push(Arc::new(RwLock::new(request)));
	}

	if requests.is_empty() {
		return Err(anyhow!(NoRequestsFound));
	}

	Ok(requests)
}

fn is_comment(line: &str) -> bool {
	// ### is a separator, not a comment
	if line.starts_with("###") {
		return false;
	}
	line.starts_with('#') || line.starts_with("//")
}

fn extract_auth_from_headers(headers: &[(String, String)]) -> Auth {
	let auth_header = headers
		.iter()
		.find(|(name, _)| name.to_lowercase() == "authorization");

	match auth_header {
		None => Auth::NoAuth,
		Some((_, value)) => {
			if let Some(token) = value.strip_prefix("Bearer ") {
				Auth::BearerToken(BearerToken {
					token: token.to_string(),
				})
			} else if let Some(encoded) = value.strip_prefix("Basic ") {
				// Decode base64 basic auth
				match base64_decode_basic_auth(encoded) {
					Some((username, password)) => Auth::BasicAuth(BasicAuth { username, password }),
					None => Auth::NoAuth,
				}
			} else {
				Auth::NoAuth
			}
		}
	}
}

fn base64_decode_basic_auth(encoded: &str) -> Option<(String, String)> {
	use base64::Engine;
	let decoded = base64::engine::general_purpose::STANDARD
		.decode(encoded)
		.ok()?;
	let decoded_str = String::from_utf8(decoded).ok()?;
	let mut parts = decoded_str.splitn(2, ':');
	let username = parts.next()?.to_string();
	let password = parts.next().unwrap_or("").to_string();
	Some((username, password))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::protocol::http::body::ContentType;

	#[test]
	fn parse_single_get_request() {
		let content = r#"### Get Users
GET https://api.example.com/users HTTP/1.1
Accept: application/json
"#;

		let requests = parse_http_content(content).unwrap();
		assert_eq!(requests.len(), 1);

		let req = requests[0].read();
		assert_eq!(req.name, "Get Users");
		assert_eq!(req.url, "https://api.example.com/users");
		assert_eq!(req.headers.len(), 1);
		assert_eq!(req.headers[0].data.0, "Accept");
		assert_eq!(req.headers[0].data.1, "application/json");

		match &req.protocol {
			Protocol::HttpRequest(http) => assert!(matches!(http.method, Method::GET)),
			_ => panic!("Expected HttpRequest"),
		}
	}

	#[test]
	fn parse_post_with_body() {
		let content = r#"### Create User
POST https://api.example.com/users
Content-Type: application/json

{"name": "John", "email": "john@example.com"}
"#;

		let requests = parse_http_content(content).unwrap();
		assert_eq!(requests.len(), 1);

		let req = requests[0].read();
		assert_eq!(req.name, "Create User");

		match &req.protocol {
			Protocol::HttpRequest(http) => {
				assert!(matches!(http.method, Method::POST));
				match &http.body {
					ContentType::Json(body) => {
						assert!(body.contains("John"));
					}
					other => panic!("Expected Json body, got {:?}", other),
				}
			}
			_ => panic!("Expected HttpRequest"),
		}
	}

	#[test]
	fn parse_multiple_requests() {
		let content = r#"### First
GET https://api.example.com/first

### Second
POST https://api.example.com/second
Content-Type: application/json

{"key": "value"}

### Third
DELETE https://api.example.com/third
"#;

		let requests = parse_http_content(content).unwrap();
		assert_eq!(requests.len(), 3);

		assert_eq!(requests[0].read().name, "First");
		assert_eq!(requests[1].read().name, "Second");
		assert_eq!(requests[2].read().name, "Third");
	}

	#[test]
	fn handle_comments() {
		let content = r#"# This is a comment
// This is also a comment
### My Request
# Another comment
GET https://api.example.com/test
// Comment in headers
Accept: application/json
"#;

		let requests = parse_http_content(content).unwrap();
		assert_eq!(requests.len(), 1);
		assert_eq!(requests[0].read().name, "My Request");
	}

	#[test]
	fn derive_name_from_method_and_path() {
		let content = "GET https://api.example.com/api/users\n";

		let requests = parse_http_content(content).unwrap();
		assert_eq!(requests.len(), 1);
		assert_eq!(requests[0].read().name, "GET /api/users");
	}

	#[test]
	fn handle_query_parameters() {
		let content = r#"### Search
GET https://api.example.com/search?q=rust&page=1
"#;

		let requests = parse_http_content(content).unwrap();
		assert_eq!(requests.len(), 1);

		let req = requests[0].read();
		assert_eq!(req.url, "https://api.example.com/search");
		assert_eq!(req.params.len(), 2);
		assert_eq!(req.params[0].data, ("q".to_string(), "rust".to_string()));
		assert_eq!(req.params[1].data, ("page".to_string(), "1".to_string()));
	}

	#[test]
	fn handle_bearer_auth() {
		let content = r#"### Authenticated Request
GET https://api.example.com/me
Authorization: Bearer my-secret-token
"#;

		let requests = parse_http_content(content).unwrap();
		let req = requests[0].read();

		match &req.auth {
			Auth::BearerToken(bearer) => assert_eq!(bearer.token, "my-secret-token"),
			other => panic!("Expected BearerToken, got {:?}", other),
		}

		// Authorization header should not be in the headers list
		assert!(req.headers.is_empty());
	}

	#[test]
	fn handle_no_body_request() {
		let content = r#"### Health Check
GET https://api.example.com/health
"#;

		let requests = parse_http_content(content).unwrap();
		let req = requests[0].read();

		match &req.protocol {
			Protocol::HttpRequest(http) => {
				assert!(matches!(http.body, ContentType::NoBody));
			}
			_ => panic!("Expected HttpRequest"),
		}
	}

	#[test]
	fn no_requests_returns_error() {
		let content = "# Just a comment\n// Another comment\n";
		let result = parse_http_content(content);
		assert!(result.is_err());
	}

	#[test]
	fn request_without_separator() {
		let content = "GET https://api.example.com/test\nAccept: application/json\n";

		let requests = parse_http_content(content).unwrap();
		assert_eq!(requests.len(), 1);
		assert_eq!(requests[0].read().name, "GET /test");
	}

	#[test]
	fn multiple_requests_with_bodies() {
		let content = r#"### Create
POST https://api.example.com/items
Content-Type: application/json

{"name": "item1"}

### Update
PUT https://api.example.com/items/1
Content-Type: application/json

{"name": "updated"}
"#;

		let requests = parse_http_content(content).unwrap();
		assert_eq!(requests.len(), 2);

		let req1 = requests[0].read();
		let req2 = requests[1].read();

		match &req1.protocol {
			Protocol::HttpRequest(http) => assert!(matches!(http.method, Method::POST)),
			_ => panic!("Expected HttpRequest"),
		}

		match &req2.protocol {
			Protocol::HttpRequest(http) => assert!(matches!(http.method, Method::PUT)),
			_ => panic!("Expected HttpRequest"),
		}
	}

	#[test]
	fn parse_websocket_request() {
		let content = r#"### Echo WebSocket
WEBSOCKET wss://echo.websocket.org
"#;

		let requests = parse_http_content(content).unwrap();
		assert_eq!(requests.len(), 1);

		let req = requests[0].read();
		assert_eq!(req.name, "Echo WebSocket");
		assert_eq!(req.url, "wss://echo.websocket.org/");

		match &req.protocol {
			Protocol::WsRequest(_) => {}
			_ => panic!("Expected WsRequest"),
		}
	}

	#[test]
	fn parse_websocket_with_headers() {
		let content = r#"### Authenticated WS
WEBSOCKET wss://api.example.com/ws
Authorization: Bearer my-token
X-Custom: header-value
"#;

		let requests = parse_http_content(content).unwrap();
		assert_eq!(requests.len(), 1);

		let req = requests[0].read();
		assert_eq!(req.name, "Authenticated WS");

		match &req.auth {
			Auth::BearerToken(bearer) => assert_eq!(bearer.token, "my-token"),
			other => panic!("Expected BearerToken, got {:?}", other),
		}

		// Authorization header should be extracted into auth, not kept in headers
		assert_eq!(req.headers.len(), 1);
		assert_eq!(req.headers[0].data.0, "X-Custom");
		assert_eq!(req.headers[0].data.1, "header-value");

		match &req.protocol {
			Protocol::WsRequest(_) => {}
			_ => panic!("Expected WsRequest"),
		}
	}

	#[test]
	fn parse_mixed_http_and_websocket() {
		let content = r#"### Get Users
GET https://api.example.com/users

### Live Feed
WEBSOCKET wss://api.example.com/feed

### Create User
POST https://api.example.com/users
Content-Type: application/json

{"name": "Jane"}
"#;

		let requests = parse_http_content(content).unwrap();
		assert_eq!(requests.len(), 3);

		assert_eq!(requests[0].read().name, "Get Users");
		match &requests[0].read().protocol {
			Protocol::HttpRequest(http) => assert!(matches!(http.method, Method::GET)),
			_ => panic!("Expected HttpRequest"),
		}

		assert_eq!(requests[1].read().name, "Live Feed");
		match &requests[1].read().protocol {
			Protocol::WsRequest(_) => {}
			_ => panic!("Expected WsRequest"),
		}

		assert_eq!(requests[2].read().name, "Create User");
		match &requests[2].read().protocol {
			Protocol::HttpRequest(http) => assert!(matches!(http.method, Method::POST)),
			_ => panic!("Expected HttpRequest"),
		}
	}
}
