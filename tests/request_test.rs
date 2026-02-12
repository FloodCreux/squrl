mod helpers;

use helpers::{
	minimal_collection_json, multi_request_collection_json, seed_collection, squrl, temp_dir,
};
use predicates::prelude::*;
use std::fs;

// ── Request Info ──────────────────────────────────────────────

#[test]
fn test_request_info() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&multi_request_collection_json("my-api"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"my-api/first-request",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("name: first-request"))
		.stdout(predicate::str::contains("protocol: HTTP"))
		.stdout(predicate::str::contains("method: GET"))
		.stdout(predicate::str::contains("https://example.com/first"));
}

#[test]
fn test_request_info_with_body() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&multi_request_collection_json("my-api"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"my-api/second-request",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("name: second-request"))
		.stdout(predicate::str::contains("method: POST"))
		.stdout(predicate::str::contains("body: JSON"));
}

#[test]
fn test_request_info_not_found() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"my-api/nonexistent",
		])
		.assert()
		.failure();
}

#[test]
fn test_request_info_collection_not_found() {
	let dir = temp_dir();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"nonexistent/req",
		])
		.assert()
		.failure();
}

// ── Request New ───────────────────────────────────────────────

#[test]
fn test_request_new() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "existing-req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"new",
			"my-api/new-request",
			"--url",
			"https://httpbin.org/get",
			"--method",
			"POST",
		])
		.assert()
		.success();

	// Verify the new request exists
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"my-api/new-request",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("name: new-request"))
		.stdout(predicate::str::contains("method: POST"))
		.stdout(predicate::str::contains("https://httpbin.org/get"));
}

#[test]
fn test_request_new_with_headers() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "existing", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"new",
			"my-api/header-req",
			"--url",
			"https://httpbin.org/headers",
			"--add-header",
			"x-custom",
			"my-value",
			"--no-base-headers",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"my-api/header-req",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("x-custom: my-value"));
}

// ── Request Delete ────────────────────────────────────────────

#[test]
fn test_request_delete() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&multi_request_collection_json("my-api"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"delete",
			"my-api/first-request",
		])
		.assert()
		.success();

	// Verify it's gone
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"my-api/first-request",
		])
		.assert()
		.failure();

	// But second-request still exists
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"my-api/second-request",
		])
		.assert()
		.success();
}

// ── Request Rename ────────────────────────────────────────────

#[test]
fn test_request_rename() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "old-name", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"rename",
			"my-api/old-name",
			"new-name",
		])
		.assert()
		.success();

	// Old name should fail
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"my-api/old-name",
		])
		.assert()
		.failure();

	// New name should work
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"my-api/new-name",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("name: new-name"));
}

// ── URL Get/Set ───────────────────────────────────────────────

#[test]
fn test_request_url_get() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://httpbin.org/get"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"url",
			"my-api/req",
			"get",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("https://httpbin.org/get"));
}

#[test]
fn test_request_url_set() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://old-url.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"url",
			"my-api/req",
			"set",
			"https://new-url.com/api",
		])
		.assert()
		.success();

	// Verify the change
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"url",
			"my-api/req",
			"get",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("https://new-url.com/api"));
}

// ── Method Get/Set ────────────────────────────────────────────

#[test]
fn test_request_method_get() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"method",
			"my-api/req",
			"get",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("GET"));
}

#[test]
fn test_request_method_set() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"method",
			"my-api/req",
			"set",
			"PUT",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"method",
			"my-api/req",
			"get",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("PUT"));
}

// ── Headers ───────────────────────────────────────────────────

#[test]
fn test_request_header_all() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&multi_request_collection_json("my-api"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"header",
			"my-api/first-request",
			"all",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("accept"))
		.stdout(predicate::str::contains("x-custom"));
}

#[test]
fn test_request_header_get() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&multi_request_collection_json("my-api"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"header",
			"my-api/first-request",
			"get",
			"x-custom",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("hello"));
}

#[test]
fn test_request_header_add_and_get() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"header",
			"my-api/req",
			"add",
			"x-new-header",
			"new-value",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"header",
			"my-api/req",
			"get",
			"x-new-header",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("new-value"));
}

#[test]
fn test_request_header_set() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&multi_request_collection_json("my-api"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"header",
			"my-api/first-request",
			"set",
			"x-custom",
			"updated-value",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"header",
			"my-api/first-request",
			"get",
			"x-custom",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("updated-value"));
}

#[test]
fn test_request_header_delete() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&multi_request_collection_json("my-api"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"header",
			"my-api/first-request",
			"delete",
			"x-custom",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"header",
			"my-api/first-request",
			"get",
			"x-custom",
		])
		.assert()
		.failure();
}

// ── Query Params ──────────────────────────────────────────────

#[test]
fn test_request_params_all() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&multi_request_collection_json("my-api"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"params",
			"my-api/first-request",
			"all",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("page"));
}

#[test]
fn test_request_params_add_and_get() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"params",
			"my-api/req",
			"add",
			"limit",
			"50",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"params",
			"my-api/req",
			"get",
			"limit",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("50"));
}

// ── Body ──────────────────────────────────────────────────────

#[test]
fn test_request_body_get_no_body() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"body",
			"my-api/req",
			"get",
		])
		.assert()
		.success();
}

#[test]
fn test_request_body_set_json() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"body",
			"my-api/req",
			"set",
			"json",
			r#"{"key": "value"}"#,
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"body",
			"my-api/req",
			"get",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("JSON"))
		.stdout(predicate::str::contains("key"));
}

#[test]
fn test_request_body_set_raw() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"body",
			"my-api/req",
			"set",
			"raw",
			"hello world",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"body",
			"my-api/req",
			"get",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("hello world"));
}

// ── Settings ──────────────────────────────────────────────────

#[test]
fn test_request_settings_all() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"settings",
			"my-api/req",
			"all",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("true").or(predicate::str::contains("false")))
		.stdout(predicate::str::contains("30000"));
}

#[test]
fn test_request_settings_get_timeout() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"settings",
			"my-api/req",
			"get",
			"timeout",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("30000"));
}

#[test]
fn test_request_settings_set_timeout() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"settings",
			"my-api/req",
			"set",
			"timeout",
			"5000",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"settings",
			"my-api/req",
			"get",
			"timeout",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("5000"));
}

// ── Export ─────────────────────────────────────────────────────

#[test]
fn test_request_export_curl() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://httpbin.org/get"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"export",
			"my-api/req",
			"curl",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("curl"))
		.stdout(predicate::str::contains("https://httpbin.org/get"));
}

#[test]
fn test_request_export_http() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://httpbin.org/get"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"export",
			"my-api/req",
			"http",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("GET"))
		.stdout(predicate::str::contains("HTTP/1.1"))
		.stdout(predicate::str::contains("Host: httpbin.org"));
}

// ── Auth ──────────────────────────────────────────────────────

#[test]
fn test_request_auth_set_basic() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"auth",
			"my-api/req",
			"set",
			"basic-auth",
			"myuser",
			"mypass",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"auth",
			"my-api/req",
			"get",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("Basic"))
		.stdout(predicate::str::contains("myuser"))
		.stdout(predicate::str::contains("mypass"));
}

#[test]
fn test_request_auth_set_bearer() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"auth",
			"my-api/req",
			"set",
			"bearer-token",
			"my-secret-token",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"auth",
			"my-api/req",
			"get",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("Bearer"))
		.stdout(predicate::str::contains("my-secret-token"));
}

#[test]
fn test_request_auth_set_no_auth() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	// First set an auth
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"auth",
			"my-api/req",
			"set",
			"bearer-token",
			"my-token",
		])
		.assert()
		.success();

	// Then remove it
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"auth",
			"my-api/req",
			"set",
			"no-auth",
		])
		.assert()
		.success();

	// Verify auth is gone (info should not show auth details)
	let output = squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"my-api/req",
		])
		.assert()
		.success();

	let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
	assert!(!stdout.contains("Bearer"));
	assert!(!stdout.contains("my-token"));
}

// ── Scripts ───────────────────────────────────────────────────

#[test]
fn test_request_scripts_set_and_get() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"scripts",
			"my-api/req",
			"set",
			"pre",
			"console.log('hello');",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"scripts",
			"my-api/req",
			"get",
			"pre",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("console.log"));
}

// ── Invalid collection/request format ─────────────────────────

#[test]
fn test_request_invalid_format_no_slash() {
	let dir = temp_dir();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"noslash",
		])
		.assert()
		.failure();
}

// ── Persistence verification ──────────────────────────────────

#[test]
fn test_changes_persist_to_disk() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "req", "https://example.com"),
	);

	// Modify the URL
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"url",
			"my-api/req",
			"set",
			"https://updated.example.com",
		])
		.assert()
		.success();

	// Read the JSON file directly and verify
	let content = fs::read_to_string(dir.path().join("my-api.json")).unwrap();
	assert!(content.contains("https://updated.example.com"));
}
