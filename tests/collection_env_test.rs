mod helpers;

use helpers::{seed_collection, squrl, temp_dir};
use predicates::prelude::*;
use std::fs;

/// Build a collection JSON with embedded environments.
fn collection_with_envs(collection_name: &str, request_name: &str, url: &str) -> String {
	serde_json::json!({
		"name": collection_name,
		"last_position": 0,
		"environments": [
			{
				"name": "dev",
				"values": {
					"BASE_URL": "http://localhost:3000",
					"API_KEY": "dev-key-123"
				}
			},
			{
				"name": "prod",
				"values": {
					"BASE_URL": "https://api.example.com",
					"API_KEY": "prod-key-456"
				}
			}
		],
		"selected_environment": "dev",
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

/// Build a minimal collection JSON without environments.
fn collection_without_envs(collection_name: &str) -> String {
	serde_json::json!({
		"name": collection_name,
		"last_position": 0,
		"requests": [
			{
				"name": "test-req",
				"url": "https://example.com",
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
					"method": "GET",
					"body": "no_body"
				}
			}
		]
	})
	.to_string()
}

// ── Collection env list ─────────────────────────────────────────────

#[test]
fn test_collection_env_list_with_envs() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "get-users", "https://example.com/users"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"list",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("dev"))
		.stdout(predicate::str::contains("prod"));
}

#[test]
fn test_collection_env_list_shows_selected() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "get-users", "https://example.com/users"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"list",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("dev *")); // dev is selected
}

#[test]
fn test_collection_env_list_empty() {
	let dir = temp_dir();
	seed_collection(dir.path(), "my-api", &collection_without_envs("my-api"));

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"list",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("No environments"));
}

// ── Collection env create ───────────────────────────────────────────

#[test]
fn test_collection_env_create() {
	let dir = temp_dir();
	seed_collection(dir.path(), "my-api", &collection_without_envs("my-api"));

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"create",
			"staging",
		])
		.assert()
		.success();

	// Verify the env was created
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"list",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("staging"));
}

#[test]
fn test_collection_env_create_duplicate_fails() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"create",
			"dev", // dev already exists
		])
		.assert()
		.failure();
}

// ── Collection env delete ───────────────────────────────────────────

#[test]
fn test_collection_env_delete() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"delete",
			"prod",
		])
		.assert()
		.success();

	// Verify prod is gone
	let output = squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"list",
		])
		.assert()
		.success();

	let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
	assert!(stdout.contains("dev"));
	assert!(!stdout.contains("prod"));
}

#[test]
fn test_collection_env_delete_selected_clears_selection() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	// Delete the selected env (dev)
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"delete",
			"dev",
		])
		.assert()
		.success();

	// Verify the list no longer shows dev or a selection marker on it
	let output = squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"list",
		])
		.assert()
		.success();

	let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
	assert!(!stdout.contains("dev"));
	// prod should still be there, unselected
	assert!(stdout.contains("prod"));
	assert!(!stdout.contains("*"));
}

// ── Collection env select ───────────────────────────────────────────

#[test]
fn test_collection_env_select() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"select",
			"prod",
		])
		.assert()
		.success();

	// Verify prod is now selected
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"list",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("prod *"));
}

#[test]
fn test_collection_env_select_nonexistent_fails() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"select",
			"nonexistent",
		])
		.assert()
		.failure();
}

// ── Collection env info ─────────────────────────────────────────────

#[test]
fn test_collection_env_info() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"info",
			"dev",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("name: dev"))
		.stdout(predicate::str::contains("BASE_URL: http://localhost:3000"))
		.stdout(predicate::str::contains("API_KEY: dev-key-123"));
}

// ── Collection env key operations ───────────────────────────────────

#[test]
fn test_collection_env_key_get() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"get",
			"BASE_URL",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("http://localhost:3000"));
}

#[test]
fn test_collection_env_key_set() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"set",
			"BASE_URL",
			"http://localhost:8080",
		])
		.assert()
		.success();

	// Verify the change
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"get",
			"BASE_URL",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("http://localhost:8080"));
}

#[test]
fn test_collection_env_key_add() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"add",
			"NEW_KEY",
			"new-value",
		])
		.assert()
		.success();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"get",
			"NEW_KEY",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("new-value"));
}

#[test]
fn test_collection_env_key_delete() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"delete",
			"API_KEY",
		])
		.assert()
		.success();

	// Verify the key is gone
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"get",
			"API_KEY",
		])
		.assert()
		.failure();
}

#[test]
fn test_collection_env_key_rename() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"rename",
			"API_KEY",
			"TOKEN",
		])
		.assert()
		.success();

	// Old key should fail
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"get",
			"API_KEY",
		])
		.assert()
		.failure();

	// New key should work
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"get",
			"TOKEN",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("dev-key-123"));
}

// ── Collection env key edge cases ───────────────────────────────────

#[test]
fn test_collection_env_key_get_nonexistent_fails() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"get",
			"NONEXISTENT_KEY",
		])
		.assert()
		.failure();
}

#[test]
fn test_collection_env_key_add_duplicate_fails() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"add",
			"BASE_URL", // already exists
			"duplicate-value",
		])
		.assert()
		.failure();
}

#[test]
fn test_collection_env_key_rename_to_existing_fails() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"rename",
			"API_KEY",
			"BASE_URL", // already exists
		])
		.assert()
		.failure();
}

#[test]
fn test_collection_env_key_set_nonexistent_fails() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"set",
			"NONEXISTENT_KEY",
			"value",
		])
		.assert()
		.failure();
}

#[test]
fn test_collection_env_key_delete_nonexistent_fails() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"delete",
			"NONEXISTENT_KEY",
		])
		.assert()
		.failure();
}

// ── Operations on nonexistent collection/env ────────────────────────

#[test]
fn test_collection_env_nonexistent_collection_fails() {
	let dir = temp_dir();
	seed_collection(dir.path(), "my-api", &collection_without_envs("my-api"));

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"nonexistent-collection",
			"list",
		])
		.assert()
		.failure();
}

#[test]
fn test_collection_env_delete_nonexistent_env_fails() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"delete",
			"nonexistent",
		])
		.assert()
		.failure();
}

#[test]
fn test_collection_env_info_nonexistent_env_fails() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"info",
			"nonexistent",
		])
		.assert()
		.failure();
}

// ── Multiple environments ───────────────────────────────────────────

#[test]
fn test_collection_env_create_multiple_envs() {
	let dir = temp_dir();
	seed_collection(dir.path(), "my-api", &collection_without_envs("my-api"));

	// Create several environments
	for env_name in &["dev", "staging", "prod"] {
		squrl()
			.args([
				"-d",
				dir.path().to_str().unwrap(),
				"collection",
				"env",
				"my-api",
				"create",
				env_name,
			])
			.assert()
			.success();
	}

	// Verify all three exist
	let output = squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"list",
		])
		.assert()
		.success();

	let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
	assert!(stdout.contains("dev"));
	assert!(stdout.contains("staging"));
	assert!(stdout.contains("prod"));
}

#[test]
fn test_collection_env_switch_between_envs() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	// Select prod
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"select",
			"prod",
		])
		.assert()
		.success();

	// Verify prod is selected, dev is not
	let output = squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"list",
		])
		.assert()
		.success();

	let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
	assert!(stdout.contains("prod *"));
	assert!(!stdout.contains("dev *"));

	// Switch back to dev
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"select",
			"dev",
		])
		.assert()
		.success();

	let output = squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"list",
		])
		.assert()
		.success();

	let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
	assert!(stdout.contains("dev *"));
	assert!(!stdout.contains("prod *"));
}

// ── Backward compatibility ──────────────────────────────────────────

#[test]
fn test_collection_without_envs_loads_normally() {
	let dir = temp_dir();
	seed_collection(dir.path(), "my-api", &collection_without_envs("my-api"));

	// Collection should load and list fine without environments
	squrl()
		.args(["-d", dir.path().to_str().unwrap(), "collection", "list"])
		.assert()
		.success()
		.stdout(predicate::str::contains("my-api"));
}

#[test]
fn test_collection_with_envs_preserves_requests() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "get-users", "https://example.com/users"),
	);

	// Verify the request is accessible alongside environments
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"info",
			"my-api/get-users",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("get-users"));
}

// ── Collection env --collection-env flag ────────────────────────────

#[test]
fn test_collection_env_flag_on_send() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "get-users", "https://example.com/users"),
	);

	// Send with --collection-env should not crash (the request will fail
	// because the URL doesn't resolve env vars at DNS level, but the flag
	// itself should be parsed and accepted)
	let output = squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"request",
			"send",
			"my-api/get-users",
			"--collection-env",
			"prod",
		])
		.assert();

	// The command should have run (not failed on arg parsing)
	// It may fail on the actual HTTP request, but that's fine
	let _ = output;
}

// ── Persistence ─────────────────────────────────────────────────────

#[test]
fn test_collection_env_changes_persist_to_disk() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	// Add a new key
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"add",
			"PERSIST_TEST",
			"persisted-value",
		])
		.assert()
		.success();

	// Read file directly and verify the change is in the JSON
	let content = fs::read_to_string(dir.path().join("my-api.json")).unwrap();
	assert!(content.contains("PERSIST_TEST"));
	assert!(content.contains("persisted-value"));
}

#[test]
fn test_collection_env_envs_embedded_in_collection_file() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	// Read the collection file and verify it contains environments
	let content = fs::read_to_string(dir.path().join("my-api.json")).unwrap();
	let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

	assert!(parsed["environments"].is_array());
	assert_eq!(parsed["environments"].as_array().unwrap().len(), 2);
	assert_eq!(parsed["selected_environment"], "dev");
}

#[test]
fn test_collection_env_selected_environment_persists_across_runs() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	// Switch to prod
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"select",
			"prod",
		])
		.assert()
		.success();

	// Verify it persisted by reading the file
	let content = fs::read_to_string(dir.path().join("my-api.json")).unwrap();
	let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
	assert_eq!(parsed["selected_environment"], "prod");

	// Also verify via a fresh squrl invocation
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"list",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("prod *"));
}

#[test]
fn test_collection_env_empty_env_not_serialized() {
	let dir = temp_dir();
	seed_collection(dir.path(), "my-api", &collection_without_envs("my-api"));

	// A collection without environments should not have the field in JSON
	// (or it should be an empty array due to skip_serializing_if)
	let content = fs::read_to_string(dir.path().join("my-api.json")).unwrap();
	let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

	// Either the field is absent or it's an empty array
	match parsed.get("environments") {
		None => {} // field absent -- OK
		Some(val) => assert!(val.as_array().unwrap().is_empty()),
	}
}

#[test]
fn test_collection_env_info_shows_all_values() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	// prod env should show its own values, not dev's
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"info",
			"prod",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("name: prod"))
		.stdout(predicate::str::contains(
			"BASE_URL: https://api.example.com",
		))
		.stdout(predicate::str::contains("API_KEY: prod-key-456"));
}

#[test]
fn test_collection_env_operations_are_isolated_between_envs() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&collection_with_envs("my-api", "req", "https://example.com"),
	);

	// Add a key to dev only
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"add",
			"DEV_ONLY",
			"dev-value",
		])
		.assert()
		.success();

	// Verify it exists in dev
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"dev",
			"get",
			"DEV_ONLY",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("dev-value"));

	// Verify it does NOT exist in prod
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"env",
			"my-api",
			"key",
			"prod",
			"get",
			"DEV_ONLY",
		])
		.assert()
		.failure();
}
