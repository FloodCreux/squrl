mod helpers;

use helpers::{seed_environment, squrl, temp_dir};
use predicates::prelude::*;
use std::fs;

#[test]
fn test_env_info() {
	let dir = temp_dir();
	seed_environment(
		dir.path(),
		"staging",
		"API_URL=https://staging.api.com\nAPI_KEY=secret123\n",
	);

	squrl()
		.args(["-d", dir.path().to_str().unwrap(), "env", "info", "staging"])
		.assert()
		.success()
		.stdout(predicate::str::contains("name: staging"))
		.stdout(predicate::str::contains("values:"))
		.stdout(predicate::str::contains("API_URL: https://staging.api.com"))
		.stdout(predicate::str::contains("API_KEY: secret123"));
}

#[test]
fn test_env_info_not_found() {
	let dir = temp_dir();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
			"info",
			"nonexistent",
		])
		.assert()
		.failure();
}

#[test]
fn test_env_key_get() {
	let dir = temp_dir();
	seed_environment(
		dir.path(),
		"dev",
		"BASE_URL=http://localhost:3000\nDEBUG=true\n",
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
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
fn test_env_key_get_not_found() {
	let dir = temp_dir();
	seed_environment(dir.path(), "dev", "KEY=value\n");

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
			"key",
			"dev",
			"get",
			"NONEXISTENT",
		])
		.assert()
		.failure();
}

#[test]
fn test_env_key_set() {
	let dir = temp_dir();
	seed_environment(dir.path(), "dev", "KEY=old-value\n");

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
			"key",
			"dev",
			"set",
			"KEY",
			"new-value",
		])
		.assert()
		.success();

	// Verify the change
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
			"key",
			"dev",
			"get",
			"KEY",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("new-value"));
}

#[test]
fn test_env_key_add() {
	let dir = temp_dir();
	seed_environment(dir.path(), "dev", "EXISTING=value\n");

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
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
			"env",
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
fn test_env_key_delete() {
	let dir = temp_dir();
	seed_environment(dir.path(), "dev", "TO_DELETE=value\nKEEP=other\n");

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
			"key",
			"dev",
			"delete",
			"TO_DELETE",
		])
		.assert()
		.success();

	// Deleted key should fail
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
			"key",
			"dev",
			"get",
			"TO_DELETE",
		])
		.assert()
		.failure();

	// Other key should still exist
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
			"key",
			"dev",
			"get",
			"KEEP",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("other"));
}

#[test]
fn test_env_key_rename() {
	let dir = temp_dir();
	seed_environment(dir.path(), "dev", "OLD_NAME=value\n");

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
			"key",
			"dev",
			"rename",
			"OLD_NAME",
			"NEW_NAME",
		])
		.assert()
		.success();

	// Old key should fail
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
			"key",
			"dev",
			"get",
			"OLD_NAME",
		])
		.assert()
		.failure();

	// New key should work
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
			"key",
			"dev",
			"get",
			"NEW_NAME",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("value"));
}

#[test]
fn test_env_changes_persist_to_disk() {
	let dir = temp_dir();
	seed_environment(dir.path(), "dev", "KEY=value\n");

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"env",
			"key",
			"dev",
			"set",
			"KEY",
			"updated",
		])
		.assert()
		.success();

	// Read file directly
	let content = fs::read_to_string(dir.path().join(".env.dev")).unwrap();
	assert!(content.contains("updated"));
}

#[test]
fn test_env_dry_run_does_not_write() {
	let dir = temp_dir();
	seed_environment(dir.path(), "dev", "KEY=original\n");

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"--dry-run",
			"env",
			"key",
			"dev",
			"set",
			"KEY",
			"modified",
		])
		.assert()
		.success();

	// File should still contain original value
	let content = fs::read_to_string(dir.path().join(".env.dev")).unwrap();
	assert!(content.contains("original"));
	assert!(!content.contains("modified"));
}
