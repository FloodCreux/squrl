use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::models::{response::RequestResponse, settings::RequestSettings};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Request {
	pub url: String,
	pub headers: Vec<KeyValue>,
	pub params: Vec<KeyValue>,
	pub settings: RequestSettings,

	#[serde(
		skip_serializing_if = "should_skip_requests_response",
		default = "RequestResponse::default"
	)]
	pub response: RequestResponse,

	#[serde(skip)]
	pub is_pending: bool,

	#[serde(skip)]
	pub cancellation_token: CancellationToken,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
	pub enabled: bool,
	pub data: (String, String),
}

fn should_skip_requests_response(_: &RequestResponse) -> bool {
	// TODO: finish this method
	false
}
