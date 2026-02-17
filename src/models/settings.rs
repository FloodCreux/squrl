use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "RawRequestSettings")]
pub struct RequestSettings {
	pub use_config_proxy: Setting,
	pub allow_redirects: Setting,
	pub timeout: Setting,
	pub store_received_cookies: Setting,
	pub pretty_print_response_content: Setting,
	pub accept_invalid_certs: Setting,
	pub accept_invalid_hostnames: Setting,
}

/// Raw deserialization target that mirrors [`RequestSettings`] before
/// normalization.  Used by `#[serde(from)]` so that every deserialized
/// `RequestSettings` is automatically normalized.
#[derive(Deserialize)]
struct RawRequestSettings {
	use_config_proxy: Setting,
	allow_redirects: Setting,
	timeout: Setting,
	store_received_cookies: Setting,
	pretty_print_response_content: Setting,
	accept_invalid_certs: Setting,
	accept_invalid_hostnames: Setting,
}

impl From<RawRequestSettings> for RequestSettings {
	fn from(raw: RawRequestSettings) -> Self {
		let mut settings = RequestSettings {
			use_config_proxy: raw.use_config_proxy,
			allow_redirects: raw.allow_redirects,
			timeout: raw.timeout,
			store_received_cookies: raw.store_received_cookies,
			pretty_print_response_content: raw.pretty_print_response_content,
			accept_invalid_certs: raw.accept_invalid_certs,
			accept_invalid_hostnames: raw.accept_invalid_hostnames,
		};
		settings.normalize();
		settings
	}
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Setting {
	Bool(bool),
	U32(u32),
}

/// Custom [`Deserialize`] for [`Setting`] that handles cross-type coercion.
///
/// When using `#[serde(untagged)]`, a YAML integer `0` or `1` would always
/// deserialize as `U32` even when a boolean was intended (e.g. a user
/// hand-editing a collection file).  This visitor accepts booleans as
/// `Bool` and integers as `U32`, making the per-field normalization in
/// [`RequestSettings::normalize`] the single place that resolves any
/// remaining ambiguity.
impl<'de> Deserialize<'de> for Setting {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct SettingVisitor;

		impl<'de> Visitor<'de> for SettingVisitor {
			type Value = Setting;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a boolean or a positive integer")
			}

			fn visit_bool<E: de::Error>(self, v: bool) -> Result<Setting, E> {
				Ok(Setting::Bool(v))
			}

			fn visit_i64<E: de::Error>(self, v: i64) -> Result<Setting, E> {
				u32::try_from(v)
					.map(Setting::U32)
					.map_err(|_| de::Error::custom(format!("integer {v} out of u32 range")))
			}

			fn visit_u64<E: de::Error>(self, v: u64) -> Result<Setting, E> {
				u32::try_from(v)
					.map(Setting::U32)
					.map_err(|_| de::Error::custom(format!("integer {v} out of u32 range")))
			}

			fn visit_f64<E: de::Error>(self, v: f64) -> Result<Setting, E> {
				if v.fract() == 0.0 && v >= 0.0 && v <= u32::MAX as f64 {
					Ok(Setting::U32(v as u32))
				} else {
					Err(de::Error::custom(format!(
						"expected a boolean or positive integer, got {v}"
					)))
				}
			}
		}

		deserializer.deserialize_any(SettingVisitor)
	}
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
	/// Coerce each field to its expected variant type.
	///
	/// Boolean fields that were deserialized as `U32` (e.g. a user wrote `1`
	/// instead of `true` in a YAML collection file) are converted:
	///   - `U32(0)` → `Bool(false)`
	///   - `U32(1)` → `Bool(true)`
	///   - any other `U32` → default value for that field
	///
	/// The `timeout` field, conversely, coerces `Bool` to its default (`30000`).
	pub fn normalize(&mut self) {
		let defaults = RequestSettings::default();

		self.use_config_proxy = coerce_to_bool(&self.use_config_proxy, &defaults.use_config_proxy);
		self.allow_redirects = coerce_to_bool(&self.allow_redirects, &defaults.allow_redirects);
		self.timeout = coerce_to_u32(&self.timeout, &defaults.timeout);
		self.store_received_cookies = coerce_to_bool(
			&self.store_received_cookies,
			&defaults.store_received_cookies,
		);
		self.pretty_print_response_content = coerce_to_bool(
			&self.pretty_print_response_content,
			&defaults.pretty_print_response_content,
		);
		self.accept_invalid_certs =
			coerce_to_bool(&self.accept_invalid_certs, &defaults.accept_invalid_certs);
		self.accept_invalid_hostnames = coerce_to_bool(
			&self.accept_invalid_hostnames,
			&defaults.accept_invalid_hostnames,
		);
	}

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

/// Coerce a `Setting` to `Bool`. If it is already `Bool`, return as-is.
/// If it is `U32(0)` → `Bool(false)`, `U32(1)` → `Bool(true)`.
/// Any other `U32` value falls back to the provided default.
fn coerce_to_bool(setting: &Setting, default: &Setting) -> Setting {
	match setting {
		Setting::Bool(_) => setting.clone(),
		Setting::U32(0) => Setting::Bool(false),
		Setting::U32(1) => Setting::Bool(true),
		Setting::U32(_) => default.clone(),
	}
}

/// Coerce a `Setting` to `U32`. If it is already `U32`, return as-is.
/// If it is `Bool`, fall back to the provided default.
fn coerce_to_u32(setting: &Setting, default: &Setting) -> Setting {
	match setting {
		Setting::U32(_) => setting.clone(),
		Setting::Bool(_) => default.clone(),
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

	// ── Normalization ───────────────────────────────────────────

	#[test]
	fn normalize_coerces_u32_0_to_bool_false_for_bool_fields() {
		let mut settings = RequestSettings {
			use_config_proxy: Setting::U32(0),
			allow_redirects: Setting::U32(0),
			timeout: Setting::U32(5000),
			store_received_cookies: Setting::U32(0),
			pretty_print_response_content: Setting::U32(0),
			accept_invalid_certs: Setting::U32(0),
			accept_invalid_hostnames: Setting::U32(0),
		};
		settings.normalize();

		assert_eq!(settings.use_config_proxy.as_bool(), Some(false));
		assert_eq!(settings.allow_redirects.as_bool(), Some(false));
		assert_eq!(settings.store_received_cookies.as_bool(), Some(false));
		assert_eq!(
			settings.pretty_print_response_content.as_bool(),
			Some(false)
		);
		assert_eq!(settings.accept_invalid_certs.as_bool(), Some(false));
		assert_eq!(settings.accept_invalid_hostnames.as_bool(), Some(false));
	}

	#[test]
	fn normalize_coerces_u32_1_to_bool_true_for_bool_fields() {
		let mut settings = RequestSettings {
			use_config_proxy: Setting::U32(1),
			allow_redirects: Setting::U32(1),
			timeout: Setting::U32(5000),
			store_received_cookies: Setting::U32(1),
			pretty_print_response_content: Setting::U32(1),
			accept_invalid_certs: Setting::U32(1),
			accept_invalid_hostnames: Setting::U32(1),
		};
		settings.normalize();

		assert_eq!(settings.use_config_proxy.as_bool(), Some(true));
		assert_eq!(settings.allow_redirects.as_bool(), Some(true));
		assert_eq!(settings.store_received_cookies.as_bool(), Some(true));
		assert_eq!(settings.pretty_print_response_content.as_bool(), Some(true));
		assert_eq!(settings.accept_invalid_certs.as_bool(), Some(true));
		assert_eq!(settings.accept_invalid_hostnames.as_bool(), Some(true));
	}

	#[test]
	fn normalize_falls_back_to_default_for_nonsensical_u32_on_bool_field() {
		let mut settings = RequestSettings {
			use_config_proxy: Setting::U32(42),
			allow_redirects: Setting::U32(999),
			timeout: Setting::U32(5000),
			store_received_cookies: Setting::U32(100),
			pretty_print_response_content: Setting::U32(200),
			accept_invalid_certs: Setting::U32(300),
			accept_invalid_hostnames: Setting::U32(400),
		};
		settings.normalize();

		let defaults = RequestSettings::default();
		assert_eq!(
			settings.use_config_proxy.as_bool(),
			defaults.use_config_proxy.as_bool()
		);
		assert_eq!(
			settings.allow_redirects.as_bool(),
			defaults.allow_redirects.as_bool()
		);
		assert_eq!(
			settings.accept_invalid_certs.as_bool(),
			defaults.accept_invalid_certs.as_bool()
		);
	}

	#[test]
	fn normalize_coerces_bool_on_timeout_to_default() {
		let mut settings = RequestSettings::default();
		settings.timeout = Setting::Bool(true);
		settings.normalize();

		assert_eq!(settings.timeout.as_u32(), Some(30000));
	}

	#[test]
	fn normalize_leaves_correct_types_unchanged() {
		let mut settings = RequestSettings {
			use_config_proxy: Setting::Bool(false),
			allow_redirects: Setting::Bool(true),
			timeout: Setting::U32(10000),
			store_received_cookies: Setting::Bool(false),
			pretty_print_response_content: Setting::Bool(true),
			accept_invalid_certs: Setting::Bool(true),
			accept_invalid_hostnames: Setting::Bool(false),
		};
		settings.normalize();

		assert_eq!(settings.use_config_proxy.as_bool(), Some(false));
		assert_eq!(settings.allow_redirects.as_bool(), Some(true));
		assert_eq!(settings.timeout.as_u32(), Some(10000));
		assert_eq!(settings.store_received_cookies.as_bool(), Some(false));
		assert_eq!(settings.pretty_print_response_content.as_bool(), Some(true));
		assert_eq!(settings.accept_invalid_certs.as_bool(), Some(true));
		assert_eq!(settings.accept_invalid_hostnames.as_bool(), Some(false));
	}

	// ── JSON roundtrip ──────────────────────────────────────────

	#[test]
	fn json_roundtrip_preserves_types() {
		let original = RequestSettings {
			use_config_proxy: Setting::Bool(false),
			allow_redirects: Setting::Bool(true),
			timeout: Setting::U32(5000),
			store_received_cookies: Setting::Bool(false),
			pretty_print_response_content: Setting::Bool(true),
			accept_invalid_certs: Setting::Bool(true),
			accept_invalid_hostnames: Setting::Bool(false),
		};

		let json = serde_json::to_string(&original).unwrap();
		let restored: RequestSettings = serde_json::from_str(&json).unwrap();

		assert_eq!(restored.use_config_proxy.as_bool(), Some(false));
		assert_eq!(restored.allow_redirects.as_bool(), Some(true));
		assert_eq!(restored.timeout.as_u32(), Some(5000));
		assert_eq!(restored.store_received_cookies.as_bool(), Some(false));
		assert_eq!(restored.pretty_print_response_content.as_bool(), Some(true));
		assert_eq!(restored.accept_invalid_certs.as_bool(), Some(true));
		assert_eq!(restored.accept_invalid_hostnames.as_bool(), Some(false));
	}

	#[test]
	fn json_with_integer_for_bool_field_is_normalized() {
		// Simulate a hand-edited JSON where someone wrote 1 instead of true
		let json = r#"{
			"use_config_proxy": 1,
			"allow_redirects": 0,
			"timeout": 5000,
			"store_received_cookies": true,
			"pretty_print_response_content": true,
			"accept_invalid_certs": false,
			"accept_invalid_hostnames": false
		}"#;

		let settings: RequestSettings = serde_json::from_str(json).unwrap();

		// 1 → true, 0 → false via normalization
		assert_eq!(settings.use_config_proxy.as_bool(), Some(true));
		assert_eq!(settings.allow_redirects.as_bool(), Some(false));
		assert_eq!(settings.timeout.as_u32(), Some(5000));
	}

	#[test]
	fn json_with_bool_for_timeout_is_normalized_to_default() {
		let json = r#"{
			"use_config_proxy": true,
			"allow_redirects": true,
			"timeout": true,
			"store_received_cookies": true,
			"pretty_print_response_content": true,
			"accept_invalid_certs": false,
			"accept_invalid_hostnames": false
		}"#;

		let settings: RequestSettings = serde_json::from_str(json).unwrap();

		// Bool on timeout → default 30000
		assert_eq!(settings.timeout.as_u32(), Some(30000));
	}

	// ── YAML roundtrip ──────────────────────────────────────────

	#[test]
	fn yaml_roundtrip_preserves_types() {
		let original = RequestSettings {
			use_config_proxy: Setting::Bool(false),
			allow_redirects: Setting::Bool(true),
			timeout: Setting::U32(5000),
			store_received_cookies: Setting::Bool(false),
			pretty_print_response_content: Setting::Bool(true),
			accept_invalid_certs: Setting::Bool(true),
			accept_invalid_hostnames: Setting::Bool(false),
		};

		let yaml = serde_yaml_ng::to_string(&original).unwrap();
		let restored: RequestSettings = serde_yaml_ng::from_str(&yaml).unwrap();

		assert_eq!(restored.use_config_proxy.as_bool(), Some(false));
		assert_eq!(restored.allow_redirects.as_bool(), Some(true));
		assert_eq!(restored.timeout.as_u32(), Some(5000));
		assert_eq!(restored.store_received_cookies.as_bool(), Some(false));
		assert_eq!(restored.pretty_print_response_content.as_bool(), Some(true));
		assert_eq!(restored.accept_invalid_certs.as_bool(), Some(true));
		assert_eq!(restored.accept_invalid_hostnames.as_bool(), Some(false));
	}

	#[test]
	fn yaml_with_integer_for_bool_field_is_normalized() {
		// Simulate a hand-edited YAML where someone wrote 1/0 instead of true/false
		let yaml = "
use_config_proxy: 1
allow_redirects: 0
timeout: 5000
store_received_cookies: true
pretty_print_response_content: true
accept_invalid_certs: false
accept_invalid_hostnames: false
";

		let settings: RequestSettings = serde_yaml_ng::from_str(yaml).unwrap();

		assert_eq!(settings.use_config_proxy.as_bool(), Some(true));
		assert_eq!(settings.allow_redirects.as_bool(), Some(false));
		assert_eq!(settings.timeout.as_u32(), Some(5000));
	}

	// ── coerce helpers ──────────────────────────────────────────

	#[test]
	fn coerce_to_bool_preserves_bool() {
		let default = Setting::Bool(true);
		assert!(matches!(
			coerce_to_bool(&Setting::Bool(false), &default),
			Setting::Bool(false)
		));
		assert!(matches!(
			coerce_to_bool(&Setting::Bool(true), &default),
			Setting::Bool(true)
		));
	}

	#[test]
	fn coerce_to_bool_converts_u32_0_and_1() {
		let default = Setting::Bool(true);
		assert!(matches!(
			coerce_to_bool(&Setting::U32(0), &default),
			Setting::Bool(false)
		));
		assert!(matches!(
			coerce_to_bool(&Setting::U32(1), &default),
			Setting::Bool(true)
		));
	}

	#[test]
	fn coerce_to_bool_falls_back_for_other_u32() {
		let default = Setting::Bool(false);
		assert!(matches!(
			coerce_to_bool(&Setting::U32(42), &default),
			Setting::Bool(false)
		));
	}

	#[test]
	fn coerce_to_u32_preserves_u32() {
		let default = Setting::U32(30000);
		assert!(matches!(
			coerce_to_u32(&Setting::U32(5000), &default),
			Setting::U32(5000)
		));
	}

	#[test]
	fn coerce_to_u32_falls_back_for_bool() {
		let default = Setting::U32(30000);
		assert!(matches!(
			coerce_to_u32(&Setting::Bool(true), &default),
			Setting::U32(30000)
		));
	}
}
