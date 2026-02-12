#![allow(dead_code)]

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Create a new `squrl` Command pointing at the cargo-built binary.
#[allow(deprecated)]
pub fn squrl() -> Command {
	Command::cargo_bin("squrl").expect("binary should exist")
}

/// Create a fresh temporary directory to use as the --directory arg.
pub fn temp_dir() -> TempDir {
	tempfile::tempdir().expect("should create temp dir")
}

/// Seed a temp directory with a JSON collection file.
/// Returns the path to the created file.
pub fn seed_collection(dir: &Path, name: &str, json: &str) -> std::path::PathBuf {
	let path = dir.join(format!("{name}.json"));
	fs::write(&path, json).expect("should write collection file");
	path
}

/// Seed a temp directory with an environment file (.env.<name>).
pub fn seed_environment(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
	let path = dir.join(format!(".env.{name}"));
	fs::write(&path, content).expect("should write environment file");
	path
}

/// A minimal valid collection JSON with one request.
pub fn minimal_collection_json(collection_name: &str, request_name: &str, url: &str) -> String {
	serde_json::json!({
		"name": collection_name,
		"last_position": 0,
		"requests": [
			{
				"name": request_name,
				"url": url,
				"params": [],
				"headers": [
					{"enabled": true, "data": ["user-agent", "SQURL/test"]}
				],
				"auth": {"no_auth": null},
				"scripts": {
					"pre_request_script": null,
					"post_request_script": null
				},
				"settings": {
					"use_config_proxy": true,
					"allow_redirects": true,
					"timeout": 30000,
					"store_received_cookies": true,
					"pretty_print_response_content": true,
					"accept_invalid_certs": false,
					"accept_invalid_hostnames": false
				},
				"protocol": {
					"type": "http",
					"method": "GET",
					"body": "no_body"
				}
			}
		]
	})
	.to_string()
}

/// A collection JSON with multiple requests.
pub fn multi_request_collection_json(collection_name: &str) -> String {
	serde_json::json!({
		"name": collection_name,
		"last_position": 0,
		"requests": [
			{
				"name": "first-request",
				"url": "https://example.com/first",
				"params": [
					{"enabled": true, "data": ["page", "1"]}
				],
				"headers": [
					{"enabled": true, "data": ["accept", "application/json"]},
					{"enabled": true, "data": ["x-custom", "hello"]}
				],
				"auth": {"no_auth": null},
				"scripts": {
					"pre_request_script": null,
					"post_request_script": null
				},
				"settings": {
					"use_config_proxy": true,
					"allow_redirects": true,
					"timeout": 30000,
					"store_received_cookies": true,
					"pretty_print_response_content": true,
					"accept_invalid_certs": false,
					"accept_invalid_hostnames": false
				},
				"protocol": {
					"type": "http",
					"method": "GET",
					"body": "no_body"
				}
			},
			{
				"name": "second-request",
				"url": "https://example.com/second",
				"params": [],
				"headers": [],
				"auth": {"no_auth": null},
				"scripts": {
					"pre_request_script": null,
					"post_request_script": null
				},
				"settings": {
					"use_config_proxy": true,
					"allow_redirects": true,
					"timeout": 30000,
					"store_received_cookies": true,
					"pretty_print_response_content": true,
					"accept_invalid_certs": false,
					"accept_invalid_hostnames": false
				},
				"protocol": {
					"type": "http",
					"method": "POST",
					"body": {
						"json": "{\"key\": \"value\"}"
					}
				}
			}
		]
	})
	.to_string()
}

/// A minimal .http file content with one request.
pub fn minimal_http_file() -> &'static str {
	"### Get Users\nGET https://httpbin.org/get\n\n### Post Data\nPOST https://httpbin.org/post\nContent-Type: application/json\n\n{\"name\": \"test\"}\n"
}
