mod helpers;

use helpers::{
	minimal_collection_json, multi_request_collection_json, seed_collection, squrl, temp_dir,
};
use predicates::prelude::*;
use std::fs;

#[test]
fn test_collection_list_empty_directory() {
	let dir = temp_dir();

	squrl()
		.args(["-d", dir.path().to_str().unwrap(), "collection", "list"])
		.assert()
		.success();
	// Note: if the CWD is a git repo with a requests/ directory,
	// an ephemeral collection may appear in the output.
}

#[test]
fn test_collection_list_with_collections() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"my-api",
		&minimal_collection_json("my-api", "get-users", "https://example.com/users"),
	);
	seed_collection(
		dir.path(),
		"another-api",
		&minimal_collection_json("another-api", "health-check", "https://example.com/health"),
	);

	let output = squrl()
		.args(["-d", dir.path().to_str().unwrap(), "collection", "list"])
		.assert()
		.success();

	let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
	assert!(stdout.contains("my-api"));
	assert!(stdout.contains("another-api"));
}

#[test]
fn test_collection_list_with_request_names() {
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
			"collection",
			"list",
			"--request-names",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("collection: my-api"))
		.stdout(predicate::str::contains("requests:"))
		.stdout(predicate::str::contains("first-request"))
		.stdout(predicate::str::contains("second-request"));
}

#[test]
fn test_collection_info() {
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
			"collection",
			"info",
			"my-api",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("collection: my-api"))
		.stdout(predicate::str::contains("requests:"))
		.stdout(predicate::str::contains("first-request"))
		.stdout(predicate::str::contains("second-request"));
}

#[test]
fn test_collection_info_without_request_names() {
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
			"collection",
			"info",
			"my-api",
			"--without-request-names",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("collection: my-api"))
		.stdout(predicate::str::contains("first-request").not());
}

#[test]
fn test_collection_info_not_found() {
	let dir = temp_dir();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"info",
			"nonexistent",
		])
		.assert()
		.failure();
}

#[test]
fn test_collection_new() {
	let dir = temp_dir();
	// Need an existing collection so the directory is recognized as having content.
	// Actually, new collection can be created on an empty dir.
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"new",
			"fresh-api",
		])
		.assert()
		.success();

	// Verify the file was created
	let json_path = dir.path().join("fresh-api.json");
	let yaml_path = dir.path().join("fresh-api.yaml");
	assert!(
		json_path.exists() || yaml_path.exists(),
		"Collection file should be created"
	);
}

#[test]
fn test_collection_new_then_list() {
	let dir = temp_dir();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"new",
			"test-collection",
		])
		.assert()
		.success();

	squrl()
		.args(["-d", dir.path().to_str().unwrap(), "collection", "list"])
		.assert()
		.success()
		.stdout(predicate::str::contains("test-collection"));
}

#[test]
fn test_collection_delete() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"to-delete",
		&minimal_collection_json("to-delete", "req", "https://example.com"),
	);

	// Verify it exists first
	squrl()
		.args(["-d", dir.path().to_str().unwrap(), "collection", "list"])
		.assert()
		.success()
		.stdout(predicate::str::contains("to-delete"));

	// Delete it
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"delete",
			"to-delete",
		])
		.assert()
		.success();

	// The collection file should be removed
	assert!(!dir.path().join("to-delete.json").exists());
}

#[test]
fn test_collection_delete_not_found() {
	let dir = temp_dir();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"delete",
			"nonexistent",
		])
		.assert()
		.failure();
}

#[test]
fn test_collection_rename() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"old-name",
		&minimal_collection_json("old-name", "req", "https://example.com"),
	);

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"rename",
			"old-name",
			"new-name",
		])
		.assert()
		.success();

	// The file on disk keeps the old filename, but the collection name inside is updated
	assert!(dir.path().join("old-name.json").exists());

	// Verify the name field in the JSON was updated
	let content = fs::read_to_string(dir.path().join("old-name.json")).unwrap();
	let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
	assert_eq!(parsed["name"], "new-name");

	// The collection is now accessible by the new name
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"info",
			"new-name",
		])
		.assert()
		.success()
		.stdout(predicate::str::contains("collection: new-name"));

	// The old name should no longer work
	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"info",
			"old-name",
		])
		.assert()
		.failure();
}

#[test]
fn test_collection_rename_not_found() {
	let dir = temp_dir();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"collection",
			"rename",
			"nonexistent",
			"new-name",
		])
		.assert()
		.failure();
}

#[test]
fn test_collection_dry_run_does_not_write() {
	let dir = temp_dir();

	squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"--dry-run",
			"collection",
			"new",
			"dry-collection",
		])
		.assert()
		.success();

	// With --dry-run, no file should be written
	assert!(!dir.path().join("dry-collection.json").exists());
	assert!(!dir.path().join("dry-collection.yaml").exists());
}

#[test]
fn test_collection_filter() {
	let dir = temp_dir();
	seed_collection(
		dir.path(),
		"api-users",
		&minimal_collection_json("api-users", "req", "https://example.com"),
	);
	seed_collection(
		dir.path(),
		"api-posts",
		&minimal_collection_json("api-posts", "req", "https://example.com"),
	);
	seed_collection(
		dir.path(),
		"other-service",
		&minimal_collection_json("other-service", "req", "https://example.com"),
	);

	// Filter to only api-* collections
	let output = squrl()
		.args([
			"-d",
			dir.path().to_str().unwrap(),
			"--filter",
			"^api-",
			"collection",
			"list",
		])
		.assert()
		.success();

	let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
	assert!(stdout.contains("api-users"));
	assert!(stdout.contains("api-posts"));
	assert!(!stdout.contains("other-service"));
}
