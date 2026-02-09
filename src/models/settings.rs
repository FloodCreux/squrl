use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSettings {
	pub use_config_proxy: Setting,
	pub allow_redirects: Setting,
	pub timeout: Setting,
	pub store_received_cookies: Setting,
	pub pretty_print_response_content: Setting,
	pub accept_invalid_certs: Setting,
	pub accept_invalid_hostnames: Setting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Setting {
	Bool(bool),
	U32(u32),
}

impl Setting {
	pub fn as_bool(&self) -> bool {
		match self {
			Setting::Bool(bool) => *bool,
			Setting::U32(_) => unreachable!(),
		}
	}

	pub fn as_u32(&self) -> u32 {
		match self {
			Setting::Bool(_) => unreachable!(),
			Setting::U32(u32) => *u32,
		}
	}
}

impl FromStr for Setting {
	type Err = String;

	fn from_str(input: &str) -> Result<Self, String> {
		match bool::from_str(&input.to_lowercase()) {
			Ok(bool) => Ok(Setting::Bool(bool)),
			Err(_) => match u32::from_str(input) {
				Ok(u32) => Ok(Setting::U32(u32)),
				Err(_) => Err(String::from(
					"Value should either be a boolean or a positive int",
				)),
			},
		}
	}
}

impl Default for RequestSettings {
	fn default() -> Self {
		RequestSettings {
			use_config_proxy: Setting::Bool(true),
			allow_redirects: Setting::Bool(true),
			timeout: Setting::U32(30000),
			store_received_cookies: Setting::Bool(true),
			pretty_print_response_content: Setting::Bool(true),
			accept_invalid_certs: Setting::Bool(false),
			accept_invalid_hostnames: Setting::Bool(false),
		}
	}
}
