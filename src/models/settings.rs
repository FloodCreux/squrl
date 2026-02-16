use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

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
#[serde(untagged)]
pub enum Setting {
	Bool(bool),
	U32(u32),
}

impl Setting {
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Setting::Bool(b) => Some(*b),
			_ => None,
		}
	}

	pub fn as_u32(&self) -> Option<u32> {
		match self {
			Setting::U32(n) => Some(*n),
			_ => None,
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

impl Display for Setting {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let str = match self {
			Setting::Bool(bool) => bool.to_string(),
			Setting::U32(uint) => uint.to_string(),
		};
		write!(f, "{}", str)
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

/// Display names for request settings, used by both [`RequestSettings::to_vec`]
/// and [`RequestSettings::update_from_vec`] to avoid duplicated string literals.
const SETTING_USE_CONFIG_PROXY: &str = "Use config proxy";
const SETTING_ALLOW_REDIRECTS: &str = "Allow redirects";
const SETTING_TIMEOUT: &str = "Timeout (ms)";
const SETTING_STORE_RECEIVED_COOKIES: &str = "Store received cookies";
const SETTING_PRETTY_PRINT_RESPONSE_CONTENT: &str = "Pretty print response content";
const SETTING_ACCEPT_INVALID_CERTS: &str = "Accept invalid certs";
const SETTING_ACCEPT_INVALID_HOSTNAMES: &str = "Accept invalid hostnames";

impl RequestSettings {
	pub fn to_vec(&self) -> Vec<(String, Setting)> {
		vec![
			(
				String::from(SETTING_USE_CONFIG_PROXY),
				self.use_config_proxy.clone(),
			),
			(
				String::from(SETTING_ALLOW_REDIRECTS),
				self.allow_redirects.clone(),
			),
			(String::from(SETTING_TIMEOUT), self.timeout.clone()),
			(
				String::from(SETTING_STORE_RECEIVED_COOKIES),
				self.store_received_cookies.clone(),
			),
			(
				String::from(SETTING_PRETTY_PRINT_RESPONSE_CONTENT),
				self.pretty_print_response_content.clone(),
			),
			(
				String::from(SETTING_ACCEPT_INVALID_CERTS),
				self.accept_invalid_certs.clone(),
			),
			(
				String::from(SETTING_ACCEPT_INVALID_HOSTNAMES),
				self.accept_invalid_hostnames.clone(),
			),
		]
	}

	pub fn update_from_vec(&mut self, vec: &[(String, Setting)]) {
		for (setting_name, setting_value) in vec {
			match setting_name.as_str() {
				SETTING_USE_CONFIG_PROXY => self.use_config_proxy = setting_value.clone(),
				SETTING_ALLOW_REDIRECTS => self.allow_redirects = setting_value.clone(),
				SETTING_TIMEOUT => self.timeout = setting_value.clone(),
				SETTING_STORE_RECEIVED_COOKIES => {
					self.store_received_cookies = setting_value.clone()
				}
				SETTING_PRETTY_PRINT_RESPONSE_CONTENT => {
					self.pretty_print_response_content = setting_value.clone()
				}
				SETTING_ACCEPT_INVALID_CERTS => self.accept_invalid_certs = setting_value.clone(),
				SETTING_ACCEPT_INVALID_HOSTNAMES => {
					self.accept_invalid_hostnames = setting_value.clone()
				}
				_ => {}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn as_bool_returns_some_true() {
		assert_eq!(Setting::Bool(true).as_bool(), Some(true));
	}

	#[test]
	fn as_bool_returns_some_false() {
		assert_eq!(Setting::Bool(false).as_bool(), Some(false));
	}

	#[test]
	fn as_bool_returns_none_for_u32_variant() {
		assert_eq!(Setting::U32(42).as_bool(), None);
	}

	#[test]
	fn as_u32_returns_some_value() {
		assert_eq!(Setting::U32(1000).as_u32(), Some(1000));
	}

	#[test]
	fn as_u32_returns_some_zero() {
		assert_eq!(Setting::U32(0).as_u32(), Some(0));
	}

	#[test]
	fn as_u32_returns_none_for_bool_variant() {
		assert_eq!(Setting::Bool(true).as_u32(), None);
	}

	#[test]
	fn default_request_settings_has_30s_timeout() {
		let settings = RequestSettings::default();
		assert_eq!(settings.timeout.as_u32(), Some(30000));
	}

	#[test]
	fn default_request_settings_has_pretty_print_enabled() {
		let settings = RequestSettings::default();
		assert_eq!(settings.pretty_print_response_content.as_bool(), Some(true));
	}

	#[test]
	fn to_vec_and_update_from_vec_roundtrip() {
		let original = RequestSettings {
			use_config_proxy: Setting::Bool(false),
			allow_redirects: Setting::Bool(false),
			timeout: Setting::U32(5000),
			store_received_cookies: Setting::Bool(false),
			pretty_print_response_content: Setting::Bool(false),
			accept_invalid_certs: Setting::Bool(true),
			accept_invalid_hostnames: Setting::Bool(true),
		};

		let vec = original.to_vec();
		let mut restored = RequestSettings::default();
		restored.update_from_vec(&vec);

		assert_eq!(restored.use_config_proxy.as_bool(), Some(false));
		assert_eq!(restored.allow_redirects.as_bool(), Some(false));
		assert_eq!(restored.timeout.as_u32(), Some(5000));
		assert_eq!(restored.store_received_cookies.as_bool(), Some(false));
		assert_eq!(
			restored.pretty_print_response_content.as_bool(),
			Some(false)
		);
		assert_eq!(restored.accept_invalid_certs.as_bool(), Some(true));
		assert_eq!(restored.accept_invalid_hostnames.as_bool(), Some(true));
	}

	#[test]
	fn update_from_vec_ignores_unknown_setting_names() {
		let mut settings = RequestSettings::default();
		let vec = vec![("Unknown setting".to_string(), Setting::Bool(true))];
		settings.update_from_vec(&vec);
		// All defaults should remain unchanged
		assert_eq!(settings.use_config_proxy.as_bool(), Some(true));
		assert_eq!(settings.timeout.as_u32(), Some(30000));
	}

	#[test]
	fn to_vec_produces_correct_display_names() {
		let settings = RequestSettings::default();
		let vec = settings.to_vec();
		let names: Vec<&str> = vec.iter().map(|(n, _)| n.as_str()).collect();
		assert_eq!(
			names,
			vec![
				"Use config proxy",
				"Allow redirects",
				"Timeout (ms)",
				"Store received cookies",
				"Pretty print response content",
				"Accept invalid certs",
				"Accept invalid hostnames",
			]
		);
	}
}
