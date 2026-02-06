use squrl::models::settings::{RequestSettings, Setting};

#[test]
fn as_bool_returns_true() {
	assert!(Setting::Bool(true).as_bool());
}

#[test]
fn as_bool_returns_false() {
	assert!(!Setting::Bool(false).as_bool());
}

#[test]
fn as_u32_returns_value() {
	assert_eq!(Setting::U32(1000).as_u32(), 1000);
}

#[test]
fn as_u32_returns_zero() {
	assert_eq!(Setting::U32(0).as_u32(), 0);
}

#[test]
#[should_panic]
fn as_bool_panics_on_u32_variant() {
	Setting::U32(42).as_bool();
}

#[test]
#[should_panic]
fn as_u32_panics_on_bool_variant() {
	Setting::Bool(true).as_u32();
}

#[test]
fn default_request_settings_has_30s_timeout() {
	let settings = RequestSettings::default();
	assert_eq!(settings.timeout.as_u32(), 30000);
}

#[test]
fn default_request_settings_has_pretty_print_enabled() {
	let settings = RequestSettings::default();
	assert!(settings.pretty_print_response_content.as_bool());
}
