use squrl::models::protocol::protocol::Protocol;
use squrl::models::request::{KeyValue, Request};

#[test]
fn default_request_has_empty_url() {
	let req = Request::default();
	assert_eq!(req.url, "");
}

#[test]
fn default_request_has_empty_headers() {
	let req = Request::default();
	assert!(req.headers.is_empty());
}

#[test]
fn default_request_has_empty_params() {
	let req = Request::default();
	assert!(req.params.is_empty());
}

#[test]
fn default_request_has_http_protocol() {
	let req = Request::default();
	assert!(matches!(req.protocol, Protocol::HttpRequest(_)));
}

#[test]
fn default_request_is_not_pending() {
	let req = Request::default();
	assert!(!req.is_pending);
}

#[test]
fn default_key_value_is_disabled() {
	let kv = KeyValue::default();
	assert!(!kv.enabled);
}

#[test]
fn default_key_value_has_empty_data() {
	let kv = KeyValue::default();
	assert_eq!(kv.data, ("".to_string(), "".to_string()));
}

#[test]
fn request_serialization_excludes_is_pending() {
	let req = Request {
		is_pending: true,
		..Default::default()
	};
	let json = serde_json::to_string(&req).unwrap();
	assert!(!json.contains("is_pending"));
}

#[test]
fn request_serialization_excludes_cancellation_token() {
	let req = Request::default();
	let json = serde_json::to_string(&req).unwrap();
	assert!(!json.contains("cancellation_token"));
}

#[test]
fn request_serialization_includes_response() {
	let req = Request::default();
	let json = serde_json::to_string(&req).unwrap();
	assert!(json.contains("response"));
}

#[test]
fn request_roundtrip_preserves_url() {
	let req = Request {
		url: "https://example.com".to_string(),
		..Default::default()
	};
	let json = serde_json::to_string(&req).unwrap();
	let deserialized: Request = serde_json::from_str(&json).unwrap();
	assert_eq!(deserialized.url, "https://example.com");
}

#[test]
fn request_roundtrip_preserves_headers() {
	let req = Request {
		headers: vec![KeyValue {
			enabled: true,
			data: ("Content-Type".to_string(), "application/json".to_string()),
		}],
		..Default::default()
	};
	let json = serde_json::to_string(&req).unwrap();
	let deserialized: Request = serde_json::from_str(&json).unwrap();
	assert_eq!(deserialized.headers.len(), 1);
	assert!(deserialized.headers[0].enabled);
	assert_eq!(deserialized.headers[0].data.0, "Content-Type");
}

#[test]
fn deserialized_request_defaults_is_pending_to_false() {
	let req = Request {
		url: "http://test.com".to_string(),
		..Default::default()
	};
	let json = serde_json::to_string(&req).unwrap();
	let req: Request = serde_json::from_str(&json).unwrap();
	assert!(!req.is_pending);
}

#[test]
fn request_roundtrip_preserves_protocol() {
	let req = Request::default();
	let json = serde_json::to_string(&req).unwrap();
	let deserialized: Request = serde_json::from_str(&json).unwrap();
	assert!(matches!(deserialized.protocol, Protocol::HttpRequest(_)));
}

#[test]
fn request_serialization_includes_protocol() {
	let req = Request::default();
	let json = serde_json::to_string(&req).unwrap();
	assert!(json.contains("protocol"));
}
