use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use image::{ImageFormat, RgbImage};
use mockito;
use parking_lot::RwLock;

use squrl::app::request::http::send::send_http_request;
use squrl::models::request::{KeyValue, Request};
use squrl::models::response::ResponseContent;
use squrl::models::settings::{RequestSettings, Setting};

use squrl::models::environment::Environment;

fn build_request_builder(url: &str) -> reqwest_middleware::RequestBuilder {
	let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build();
	client.get(url)
}

fn build_env() -> Option<Arc<RwLock<Environment>>> {
	None
}

fn build_local_request(timeout_ms: u32) -> Arc<RwLock<Request>> {
	Arc::new(RwLock::new(Request {
		settings: RequestSettings {
			timeout: Setting::U32(timeout_ms),
			pretty_print_response_content: Setting::Bool(true),
			..Default::default()
		},
		..Default::default()
	}))
}

#[tokio::test]
async fn test_successful_text_response() {
	let mut server = mockito::Server::new_async().await;
	let mock = server
		.mock("GET", "/hello")
		.with_status(200)
		.with_body("Hello, world!")
		.create_async()
		.await;

	let url = format!("{}/hello", server.url());
	let request_builder = build_request_builder(&url);
	let local_request = build_local_request(5000);

	let result = send_http_request(request_builder, local_request.clone(), &build_env()).await;
	mock.assert_async().await;

	let response = result.unwrap();
	assert_eq!(response.status_code, Some("200 OK".to_string()));
	assert!(response.duration.is_some());
	assert!(!local_request.read().is_pending);

	match response.content {
		Some(ResponseContent::Body(body)) => assert_eq!(body, "Hello, world!"),
		other => panic!("Expected Body content, got {:?}", other),
	}
}

#[tokio::test]
async fn test_server_error_response() {
	let mut server = mockito::Server::new_async().await;
	let mock = server
		.mock("GET", "/error")
		.with_status(500)
		.with_body("Internal Server Error")
		.create_async()
		.await;

	let url = format!("{}/error", server.url());
	let request_builder = build_request_builder(&url);
	let local_request = build_local_request(5000);

	let result = send_http_request(request_builder, local_request, &build_env()).await;
	mock.assert_async().await;

	let response = result.unwrap();
	assert_eq!(
		response.status_code,
		Some("500 Internal Server Error".to_string())
	);

	match response.content {
		Some(ResponseContent::Body(body)) => assert_eq!(body, "Internal Server Error"),
		other => panic!("Expected Body content, got {:?}", other),
	}
}

#[tokio::test]
async fn test_timeout() {
	// Use a TCP server that accepts connections but never sends a response,
	// so send() blocks waiting for headers until the timeout fires.
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let port = listener.local_addr().unwrap().port();

	tokio::spawn(async move {
		loop {
			if let Ok((socket, _)) = listener.accept().await {
				tokio::spawn(async move {
					let _socket = socket;
					tokio::time::sleep(Duration::from_secs(60)).await;
				});
			}
		}
	});

	let url = format!("http://127.0.0.1:{}/slow", port);
	let request_builder = build_request_builder(&url);
	let local_request = build_local_request(100); // 100ms timeout

	let result = send_http_request(request_builder, local_request, &build_env()).await;

	let response = result.unwrap();
	assert_eq!(response.status_code, Some("TIMEOUT".to_string()));
	assert!(response.content.is_none());
	assert!(response.duration.is_some());
}

#[tokio::test]
async fn test_cancellation() {
	// Use a TCP server that accepts connections but never sends a response,
	// so send() blocks waiting for headers until cancellation fires.
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let port = listener.local_addr().unwrap().port();

	tokio::spawn(async move {
		loop {
			if let Ok((socket, _)) = listener.accept().await {
				tokio::spawn(async move {
					let _socket = socket;
					tokio::time::sleep(Duration::from_secs(60)).await;
				});
			}
		}
	});

	let url = format!("http://127.0.0.1:{}/cancel", port);
	let request_builder = build_request_builder(&url);
	let local_request = build_local_request(30000);

	let cancellation_token = local_request.read().cancellation_token.clone();

	// Cancel after a short delay
	tokio::spawn(async move {
		tokio::time::sleep(Duration::from_millis(50)).await;
		cancellation_token.cancel();
	});

	let result = send_http_request(request_builder, local_request, &build_env()).await;

	let response = result.unwrap();
	assert_eq!(response.status_code, Some("CANCELED".to_string()));
	assert!(response.content.is_none());
	assert!(response.duration.is_some());
}

#[tokio::test]
async fn test_response_headers_are_captured() {
	let mut server = mockito::Server::new_async().await;
	let mock = server
		.mock("GET", "/headers")
		.with_status(200)
		.with_header("x-custom-header", "custom-value")
		.with_body("ok")
		.create_async()
		.await;

	let url = format!("{}/headers", server.url());
	let request_builder = build_request_builder(&url);
	let local_request = build_local_request(5000);

	let result = send_http_request(request_builder, local_request, &build_env()).await;
	mock.assert_async().await;

	let response = result.unwrap();
	let custom_header = response
		.headers
		.iter()
		.find(|(name, _)| name == "x-custom-header");
	assert_eq!(
		custom_header,
		Some(&("x-custom-header".to_string(), "custom-value".to_string()))
	);
}

#[tokio::test]
async fn test_image_response() {
	// Generate a valid 1x1 PNG using the image crate
	let img = RgbImage::new(1, 1);
	let mut png_bytes: Vec<u8> = Vec::new();
	img.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
		.unwrap();

	let mut server = mockito::Server::new_async().await;
	let mock = server
		.mock("GET", "/image")
		.with_status(200)
		.with_header("content-type", "image/png")
		.with_body(png_bytes.clone())
		.create_async()
		.await;

	let url = format!("{}/image", server.url());
	let request_builder = build_request_builder(&url);
	let local_request = build_local_request(5000);

	let result = send_http_request(request_builder, local_request, &build_env()).await;
	mock.assert_async().await;

	let response = result.unwrap();
	assert_eq!(response.status_code, Some("200 OK".to_string()));

	match response.content {
		Some(ResponseContent::Image(img)) => {
			assert_eq!(img.data, png_bytes);
			assert!(img.image.is_some());
		}
		other => panic!("Expected Image content, got {:?}", other),
	}
}

#[tokio::test]
async fn test_sets_is_pending() {
	let mut server = mockito::Server::new_async().await;
	let _mock = server
		.mock("GET", "/pending")
		.with_status(200)
		.with_body("ok")
		.create_async()
		.await;

	let url = format!("{}/pending", server.url());
	let request_builder = build_request_builder(&url);
	let local_request = build_local_request(5000);

	assert!(!local_request.read().is_pending);

	let _ = send_http_request(request_builder, local_request.clone(), &build_env()).await;

	// is_pending is set to true during the request, then back to false before returning
	assert!(!local_request.read().is_pending);
}

#[tokio::test]
async fn test_connection_error() {
	// Point to a port that's not listening
	let request_builder = build_request_builder("http://127.0.0.1:1");
	let local_request = build_local_request(5000);

	let result = send_http_request(request_builder, local_request, &build_env()).await;

	let response = result.unwrap();
	assert!(response.status_code.is_none());
	assert!(response.duration.is_some());

	match response.content {
		Some(ResponseContent::Body(body)) => {
			assert!(!body.is_empty());
		}
		other => panic!("Expected Body content with error message, got {:?}", other),
	}
}

#[tokio::test]
async fn test_cookies_are_captured() {
	let mut server = mockito::Server::new_async().await;
	let mock = server
		.mock("GET", "/cookies")
		.with_status(200)
		.with_header("set-cookie", "session=abc123")
		.with_body("ok")
		.create_async()
		.await;

	let url = format!("{}/cookies", server.url());
	let request_builder = build_request_builder(&url);
	let local_request = build_local_request(5000);

	let result = send_http_request(request_builder, local_request, &build_env()).await;
	mock.assert_async().await;

	let response = result.unwrap();
	assert!(response.cookies.is_some());
	let cookies = response.cookies.unwrap();
	assert!(
		cookies.contains("session"),
		"Expected 'session' in cookies: {}",
		cookies
	);
	assert!(
		cookies.contains("abc123"),
		"Expected 'abc123' in cookies: {}",
		cookies
	);
}

#[tokio::test]
async fn test_json_pretty_print_enabled() {
	let mut server = mockito::Server::new_async().await;
	let mock = server
		.mock("GET", "/json")
		.with_status(200)
		.with_header("content-type", "application/json")
		.with_body(r#"{"key":"value","nested":{"a":1}}"#)
		.create_async()
		.await;

	let url = format!("{}/json", server.url());
	let request_builder = build_request_builder(&url);
	let local_request = build_local_request(5000);

	let result = send_http_request(request_builder, local_request, &build_env()).await;
	mock.assert_async().await;

	let response = result.unwrap();
	match response.content {
		Some(ResponseContent::Body(body)) => {
			assert!(
				body.contains('\n'),
				"Expected pretty-printed JSON with newlines"
			);
			assert!(
				body.contains("  "),
				"Expected pretty-printed JSON with indentation"
			);
		}
		other => panic!("Expected Body content, got {:?}", other),
	}
}

#[tokio::test]
async fn test_json_pretty_print_disabled() {
	let mut server = mockito::Server::new_async().await;
	let mock = server
		.mock("GET", "/json")
		.with_status(200)
		.with_header("content-type", "application/json")
		.with_body(r#"{"key":"value"}"#)
		.create_async()
		.await;

	let url = format!("{}/json", server.url());
	let request_builder = build_request_builder(&url);
	let local_request = Arc::new(RwLock::new(Request {
		settings: RequestSettings {
			timeout: Setting::U32(5000),
			pretty_print_response_content: Setting::Bool(false),
			..Default::default()
		},
		..Default::default()
	}));

	let result = send_http_request(request_builder, local_request, &build_env()).await;
	mock.assert_async().await;

	let response = result.unwrap();
	match response.content {
		Some(ResponseContent::Body(body)) => {
			assert_eq!(body, r#"{"key":"value"}"#);
		}
		other => panic!("Expected Body content, got {:?}", other),
	}
}

#[tokio::test]
async fn test_empty_response_body() {
	let mut server = mockito::Server::new_async().await;
	let mock = server
		.mock("GET", "/empty")
		.with_status(204)
		.with_body("")
		.create_async()
		.await;

	let url = format!("{}/empty", server.url());
	let request_builder = build_request_builder(&url);
	let local_request = build_local_request(5000);

	let result = send_http_request(request_builder, local_request, &build_env()).await;
	mock.assert_async().await;

	let response = result.unwrap();
	assert_eq!(response.status_code, Some("204 No Content".to_string()));
	match response.content {
		Some(ResponseContent::Body(body)) => assert_eq!(body, ""),
		other => panic!("Expected empty Body content, got {:?}", other),
	}
}

#[tokio::test]
async fn test_multiple_response_headers() {
	let mut server = mockito::Server::new_async().await;
	let mock = server
		.mock("GET", "/multi-headers")
		.with_status(200)
		.with_header("x-first", "one")
		.with_header("x-second", "two")
		.with_header("x-third", "three")
		.with_body("ok")
		.create_async()
		.await;

	let url = format!("{}/multi-headers", server.url());
	let request_builder = build_request_builder(&url);
	let local_request = build_local_request(5000);

	let result = send_http_request(request_builder, local_request, &build_env()).await;
	mock.assert_async().await;

	let response = result.unwrap();
	let header_names: Vec<&str> = response.headers.iter().map(|(k, _)| k.as_str()).collect();
	assert!(header_names.contains(&"x-first"));
	assert!(header_names.contains(&"x-second"));
	assert!(header_names.contains(&"x-third"));
}

#[tokio::test]
async fn test_request_with_custom_headers() {
	let mut server = mockito::Server::new_async().await;
	let mock = server
		.mock("GET", "/auth")
		.with_status(200)
		.with_body("authorized")
		.create_async()
		.await;

	let url = format!("{}/auth", server.url());
	let local_request = Arc::new(RwLock::new(Request {
		url: url.clone(),
		headers: vec![KeyValue {
			enabled: true,
			data: ("Authorization".to_string(), "Bearer test-token".to_string()),
		}],
		settings: RequestSettings {
			timeout: Setting::U32(5000),
			pretty_print_response_content: Setting::Bool(true),
			..Default::default()
		},
		..Default::default()
	}));

	let request_builder = build_request_builder(&url);
	let result = send_http_request(request_builder, local_request, &build_env()).await;
	mock.assert_async().await;

	let response = result.unwrap();
	assert_eq!(response.status_code, Some("200 OK".to_string()));
}

#[tokio::test]
async fn test_duration_format() {
	let mut server = mockito::Server::new_async().await;
	let _mock = server
		.mock("GET", "/timing")
		.with_status(200)
		.with_body("ok")
		.create_async()
		.await;

	let url = format!("{}/timing", server.url());
	let request_builder = build_request_builder(&url);
	let local_request = build_local_request(5000);

	let result = send_http_request(request_builder, local_request, &build_env()).await;
	let response = result.unwrap();

	let duration = response.duration.unwrap();
	// Duration format should contain time unit indicators (ns, µs, ms, or s)
	assert!(
		duration.contains("ns")
			|| duration.contains("µs")
			|| duration.contains("ms")
			|| duration.contains('s'),
		"Duration '{}' should contain a time unit",
		duration
	);
}
