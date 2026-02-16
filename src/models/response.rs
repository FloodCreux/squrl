use image::DynamicImage;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RequestResponse {
	pub duration: Option<String>,
	pub status_code: Option<String>,
	pub content: Option<ResponseContent>,
	pub cookies: Option<String>,
	pub headers: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseContent {
	Body(String),
	Image(ImageResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResponse {
	pub data: Vec<u8>,

	#[serde(skip)]
	pub image: Option<DynamicImage>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn default_response_has_no_duration() {
		let resp = RequestResponse::default();
		assert!(resp.duration.is_none());
	}

	#[test]
	fn default_response_has_no_status_code() {
		let resp = RequestResponse::default();
		assert!(resp.status_code.is_none());
	}

	#[test]
	fn default_response_has_no_content() {
		let resp = RequestResponse::default();
		assert!(resp.content.is_none());
	}

	#[test]
	fn default_response_has_no_cookies() {
		let resp = RequestResponse::default();
		assert!(resp.cookies.is_none());
	}

	#[test]
	fn default_response_has_empty_headers() {
		let resp = RequestResponse::default();
		assert!(resp.headers.is_empty());
	}

	#[test]
	fn response_content_body_holds_string() {
		let content = ResponseContent::Body("hello".to_string());
		match content {
			ResponseContent::Body(s) => assert_eq!(s, "hello"),
			_ => panic!("Expected Body variant"),
		}
	}

	#[test]
	fn response_content_image_holds_data() {
		let data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes
		let content = ResponseContent::Image(ImageResponse {
			data: data.clone(),
			image: None,
		});
		match content {
			ResponseContent::Image(img) => assert_eq!(img.data, data),
			_ => panic!("Expected Image variant"),
		}
	}

	#[test]
	fn response_serializes_to_json() {
		let resp = RequestResponse {
			duration: Some("100ms".to_string()),
			status_code: Some("200 OK".to_string()),
			content: Some(ResponseContent::Body("test".to_string())),
			cookies: None,
			headers: vec![("content-type".to_string(), "text/plain".to_string())],
		};
		let json = serde_json::to_string(&resp).unwrap();
		assert!(json.contains("200 OK"));
		assert!(json.contains("100ms"));
		assert!(json.contains("test"));
	}

	#[test]
	fn response_deserializes_from_json() {
		let json = r#"{"duration":"50ms","status_code":"404 Not Found","content":"not found","cookies":null,"headers":[]}"#;
		let resp: RequestResponse = serde_json::from_str(json).unwrap();
		assert_eq!(resp.status_code, Some("404 Not Found".to_string()));
		assert_eq!(resp.duration, Some("50ms".to_string()));
	}

	#[test]
	fn response_clone_is_independent() {
		let resp = RequestResponse {
			duration: Some("100ms".to_string()),
			status_code: Some("200 OK".to_string()),
			content: Some(ResponseContent::Body("original".to_string())),
			cookies: None,
			headers: vec![],
		};
		let cloned = resp.clone();
		assert_eq!(resp.status_code, cloned.status_code);
		assert_eq!(resp.duration, cloned.duration);
	}
}
