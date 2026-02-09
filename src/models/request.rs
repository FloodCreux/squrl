use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::models::{
	auth::auth::Auth, protocol::protocol::Protocol, response::RequestResponse,
	settings::RequestSettings,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Request {
	pub name: String,
	pub url: String,
	pub headers: Vec<KeyValue>,
	pub params: Vec<KeyValue>,
	pub settings: RequestSettings,
	pub auth: Auth,

	pub protocol: Protocol,

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

impl Request {
	pub fn update_url_and_params(&mut self, url: String) {
		let url_parts = url.trim().split_once("?");

		let final_url: String;
		let query_params: &str;

		if let Some((url, found_query_params)) = url_parts {
			final_url = url.to_string();
			query_params = found_query_params;
		} else {
			final_url = url;
			query_params = "";
		}

		let mut found_params = vec![];

		let path_params_pattern = Regex::new(r"(\{+[\w-]+}+)").unwrap();
		for (_, [path_param]) in path_params_pattern
			.captures_iter(&final_url)
			.map(|c| c.extract())
		{
			if path_param.starts_with("{{") || path_param.ends_with("}}") {
				continue;
			}

			found_params.push((path_param.to_string(), None));
		}

		let query_params_pattern = Regex::new(r"(&?([^=]+)=([^&]+))").unwrap();
		for (_, [_, param_name, value]) in query_params_pattern
			.captures_iter(query_params)
			.map(|c| c.extract())
		{
			found_params.push((param_name.to_string(), Some(value.to_string())));
		}

		self.params
			.retain(|param| found_params.iter().any(|found| found.0 == param.data.0));

		for found_param in found_params {
			let param = self
				.params
				.iter_mut()
				.find(|param| param.data.0 == found_param.0);

			if let Some(param) = param {
				if let Some(value) = found_param.1 {
					param.data.1 = value;
				}
			} else {
				let value = found_param.1.unwrap_or_else(|| String::from("value"));
				self.params.push(KeyValue {
					enabled: true,
					data: (found_param.0, value),
				});
			}
		}

		self.url = final_url;
	}
}

lazy_static! {
	pub static ref DEFAULT_HEADERS: Vec<KeyValue> = vec![
		KeyValue {
			enabled: true,
			data: (String::from("cache-control"), String::from("no-cache")),
		},
		KeyValue {
			enabled: true,
			data: (
				String::from("user-agent"),
				format!("squrl/v{}", env!("CARGO_PKG_VERSION"))
			),
		},
		KeyValue {
			enabled: true,
			data: (String::from("accept"), String::from("*/*")),
		},
		KeyValue {
			enabled: true,
			data: (
				String::from("accept-encoding"),
				String::from("gzip, deflate, br")
			),
		},
		KeyValue {
			enabled: true,
			data: (String::from("connection"), String::from("keep-alive")),
		},
	];
}
