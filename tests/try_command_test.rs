mod helpers;

use helpers::squrl;

// The `try` command sends one-shot requests without needing a directory.
// These tests use real HTTP endpoints (httpbin) or mock unavailable scenarios.

#[test]
fn test_try_with_url() {
	// Use a URL that will fail to connect quickly (unroutable address)
	// to test the command structure without depending on external services.
	squrl()
		.args([
			"try",
			"--url",
			"http://127.0.0.1:1/test",
			"--timeout",
			"500",
		])
		.timeout(std::time::Duration::from_secs(10))
		.assert()
		.success(); // try still succeeds (prints error response)
}

#[test]
fn test_try_with_method() {
	squrl()
		.args([
			"try",
			"--url",
			"http://127.0.0.1:1/test",
			"--method",
			"POST",
			"--timeout",
			"500",
		])
		.timeout(std::time::Duration::from_secs(10))
		.assert()
		.success();
}

#[test]
fn test_try_with_status_code_flag_on_connection_error_panics() {
	// Known issue: using --status-code when the request fails (e.g., connection refused)
	// causes a panic because the response has no status code.
	// This test documents the current behavior.
	squrl()
		.args([
			"try",
			"--url",
			"http://127.0.0.1:1/test",
			"--timeout",
			"500",
			"--status-code",
		])
		.timeout(std::time::Duration::from_secs(10))
		.assert()
		.failure();
}

#[test]
fn test_try_with_hide_content() {
	squrl()
		.args([
			"try",
			"--url",
			"http://127.0.0.1:1/test",
			"--timeout",
			"500",
			"--hide-content",
		])
		.timeout(std::time::Duration::from_secs(10))
		.assert()
		.success();
}

#[test]
fn test_try_with_headers_flag() {
	squrl()
		.args([
			"try",
			"--url",
			"http://127.0.0.1:1/test",
			"--timeout",
			"500",
			"--headers",
			"--hide-content",
		])
		.timeout(std::time::Duration::from_secs(10))
		.assert()
		.success();
}

#[test]
fn test_try_with_duration_flag() {
	squrl()
		.args([
			"try",
			"--url",
			"http://127.0.0.1:1/test",
			"--timeout",
			"500",
			"--duration",
			"--hide-content",
		])
		.timeout(std::time::Duration::from_secs(10))
		.assert()
		.success();
}

#[test]
fn test_try_with_json_body() {
	squrl()
		.args([
			"try",
			"--url",
			"http://127.0.0.1:1/test",
			"--method",
			"POST",
			"--body-json",
			r#"{"key": "value"}"#,
			"--timeout",
			"500",
		])
		.timeout(std::time::Duration::from_secs(10))
		.assert()
		.success();
}

#[test]
fn test_try_with_custom_header() {
	squrl()
		.args([
			"try",
			"--url",
			"http://127.0.0.1:1/test",
			"--add-header",
			"x-custom",
			"my-value",
			"--timeout",
			"500",
		])
		.timeout(std::time::Duration::from_secs(10))
		.assert()
		.success();
}

#[test]
fn test_try_with_no_base_headers() {
	squrl()
		.args([
			"try",
			"--url",
			"http://127.0.0.1:1/test",
			"--no-base-headers",
			"--timeout",
			"500",
		])
		.timeout(std::time::Duration::from_secs(10))
		.assert()
		.success();
}

#[test]
fn test_try_with_query_params() {
	squrl()
		.args([
			"try",
			"--url",
			"http://127.0.0.1:1/test",
			"--add-param",
			"key",
			"value",
			"--timeout",
			"500",
		])
		.timeout(std::time::Duration::from_secs(10))
		.assert()
		.success();
}
