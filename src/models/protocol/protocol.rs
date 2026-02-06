use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::models::protocol::http::http::HttpRequest;

#[derive(Debug, Clone, EnumString, Display, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Protocol {
	#[serde(rename = "http", alias = "http", alias = "HTTP")]
	#[strum(to_string = "HTTP")]
	HttpRequest(HttpRequest),
}

impl Default for Protocol {
	fn default() -> Self {
		Protocol::HttpRequest(HttpRequest::default())
	}
}
