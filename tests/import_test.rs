mod helpers;

use helpers::{minimal_http_file, squrl, temp_dir};
use predicates::prelude::*;
use std::fs;

// ── .http file import ─────────────────────────────────────────

#[test]
fn test_import_http_file() {
	let dir = temp_dir();

	// Create the .http file to import
	let http_file_path = dir.path().join("test-api.http");
	fs::write(&http_file_path, minimal_http_file()).unwrap();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"import",
			"http-file",
			http_file_path.to_str().unwrap(),
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("Parsing .http file"));

	// Verify the collection was created
	squrl()
		.args(["-d", dir.path().to_str().unwrap(), "collection", "list"])
		.assert()
		.success()
		.stdout(predicate::str::contains("test-api"));
}

#[test]
fn test_import_http_file_with_custom_collection_name() {
	let dir = temp_dir();

	let http_file_path = dir.path().join("requests.http");
	fs::write(&http_file_path, minimal_http_file()).unwrap();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"import",
			"http-file",
			http_file_path.to_str().unwrap(),
			"custom-name",
		])
		.assert()
		.success();

	squrl()
		.args(["-d", dir.path().to_str().unwrap(), "collection", "list"])
		.assert()
		.success()
		.stdout(predicate::str::contains("custom-name"));
}

#[test]
fn test_import_http_file_creates_requests() {
	let dir = temp_dir();

	let http_file_path = dir.path().join("test-api.http");
	fs::write(&http_file_path, minimal_http_file()).unwrap();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"import",
			"http-file",
			http_file_path.to_str().unwrap(),
		])
		.assert()
		.success();

	// Verify individual requests were created
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"info",
			"test-api",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("Get Users"))
		.stdout(predicate::str::contains("Post Data"));
}

#[test]
fn test_import_http_file_request_details() {
	let dir = temp_dir();

	let http_file_path = dir.path().join("test-api.http");
	fs::write(&http_file_path, minimal_http_file()).unwrap();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"import",
			"http-file",
			http_file_path.to_str().unwrap(),
		])
		.assert()
		.success();

	// Check GET request details
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"test-api/Get Users",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("method: GET"))
		.stdout(predicate::str::contains("https://httpbin.org/get"));

	// Check POST request details
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"test-api/Post Data",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("method: POST"))
		.stdout(predicate::str::contains("https://httpbin.org/post"))
		.stdout(predicate::str::contains("body: JSON"));
}

#[test]
fn test_import_http_file_nonexistent_succeeds_with_empty_collection() {
	let dir = temp_dir();

	// The import command doesn't error on a nonexistent file path --
	// it creates a new empty collection instead.
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"import",
			"http-file",
			"/nonexistent/file.http",
		])
		.assert()
		.success();
}

// ── .http file import into existing collection ────────────────

#[test]
fn test_import_http_file_into_existing_collection() {
	let dir = temp_dir();

	// Create a collection first
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"new",
			"existing-col",
		])
		.assert()
		.success();

	// Import .http file into the existing collection
	let http_file_path = dir.path().join("extra.http");
	fs::write(
		&http_file_path,
		"### Extra Request\nGET https://httpbin.org/anything\n",
	)
	.unwrap();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"import",
			"http-file",
			http_file_path.to_str().unwrap(),
			"existing-col",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"info",
			"existing-col",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("Extra Request"));
}

// ── curl import ───────────────────────────────────────────────

#[test]
fn test_import_curl_file() {
	let dir = temp_dir();

	let curl_file_path = dir.path().join("request.curl");
	fs::write(
		&curl_file_path,
		"curl -X GET https://httpbin.org/get -H 'Accept: application/json'",
	)
	.unwrap();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"import",
			"curl",
			curl_file_path.to_str().unwrap(),
			"curl-collection",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("Parsing cURL request"));

	// Verify collection was created
	squrl()
		.args(["-d", dir.path().to_str().unwrap(), "collection", "list"])
		.assert()
		.success()
		.stdout(predicate::str::contains("curl-collection"));
}

#[test]
fn test_import_curl_with_request_name() {
	let dir = temp_dir();

	let curl_file_path = dir.path().join("req.curl");
	fs::write(
		&curl_file_path,
		"curl -X POST https://httpbin.org/post -d '{\"key\": \"value\"}' -H 'Content-Type: application/json'",
	)
	.unwrap();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"import",
			"curl",
			curl_file_path.to_str().unwrap(),
			"my-curl-api",
			"post-data",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"my-curl-api/post-data",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("name: post-data"))
		.stdout(predicate::str::contains("POST"));
}

// ── OpenAPI import ────────────────────────────────────────────

#[test]
fn test_import_openapi_json() {
	// Use separate directories: one for squrl data, one for the spec file.
	// This avoids squrl trying to parse the spec as a collection during startup.
	let squrl_dir = temp_dir();
	let spec_dir = temp_dir();

	let openapi_spec = serde_json::json!({
		"openapi": "3.0.0",
		"info": {
			"title": "Test API",
			"version": "1.0.0"
		},
		"servers": [
			{"url": "https://api.example.com"}
		],
		"paths": {
			"/users": {
				"get": {
					"summary": "List users",
					"operationId": "listUsers",
					"responses": {
						"200": {
							"description": "OK"
						}
					}
				}
			},
			"/users/{id}": {
				"get": {
					"summary": "Get user by ID",
					"operationId": "getUser",
					"parameters": [
						{
							"name": "id",
							"in": "path",
							"required": true,
							"schema": {"type": "string"}
						}
					],
					"responses": {
						"200": {
							"description": "OK"
						}
					}
				}
			}
		}
	});

	let spec_path = spec_dir.path().join("openapi.json");
	fs::write(
		&spec_path,
		serde_json::to_string_pretty(&openapi_spec).unwrap(),
	)
	.unwrap();

	squrl()
		.args([
			"-d",
			squrl_dir.path().to_str().unwrap(),
			"import",
			"open-api",
			spec_path.to_str().unwrap(),
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("Parsing OpenAPI specification"))
		.stdout(predicate::str::contains("Collection name: Test API"));

	// Verify collection was created
	squrl()
		.args([
			"-d",
			squrl_dir.path().to_str().unwrap(),
			"collection",
			"list",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("Test API"));
}

#[test]
fn test_import_openapi_yaml() {
	let squrl_dir = temp_dir();
	let spec_dir = temp_dir();

	let openapi_yaml = r#"
openapi: "3.0.0"
info:
  title: "YAML API"
  version: "1.0.0"
servers:
  - url: "https://api.yamltest.com"
paths:
  /health:
    get:
      summary: "Health check"
      operationId: "healthCheck"
      responses:
        "200":
          description: "OK"
"#;

	let spec_path = spec_dir.path().join("openapi.yaml");
	fs::write(&spec_path, openapi_yaml).unwrap();

	squrl()
		.args([
			"-d",
			squrl_dir.path().to_str().unwrap(),
			"import",
			"open-api",
			spec_path.to_str().unwrap(),
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("YAML API"));
}
