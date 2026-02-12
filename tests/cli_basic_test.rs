mod helpers;

use helpers::squrl;
use predicates::prelude::*;

#[test]
fn test_help_flag() {
	squrl()
		.arg("--help")
		.assert()
		.success()
		.stdout(predicate::str::contains("Usage:"))
		.stdout(predicate::str::contains("squrl"));
}

#[test]
fn test_version_flag() {
	squrl()
		.arg("--version")
		.assert()
		.success()
		.stdout(predicate::str::contains("squrl"));
}

#[test]
fn test_collection_help() {
	squrl()
		.args(["collection", "--help"])
		.assert()
		.success()
		.stdout(predicate::str::contains("list"))
		.stdout(predicate::str::contains("info"))
		.stdout(predicate::str::contains("new"))
		.stdout(predicate::str::contains("delete"))
		.stdout(predicate::str::contains("rename"))
		.stdout(predicate::str::contains("send"));
}

#[test]
fn test_request_help() {
	squrl()
		.args(["request", "--help"])
		.assert()
		.success()
		.stdout(predicate::str::contains("info"))
		.stdout(predicate::str::contains("new"))
		.stdout(predicate::str::contains("delete"))
		.stdout(predicate::str::contains("url"))
		.stdout(predicate::str::contains("method"))
		.stdout(predicate::str::contains("send"))
		.stdout(predicate::str::contains("export"));
}

#[test]
fn test_env_help() {
	squrl()
		.args(["env", "--help"])
		.assert()
		.success()
		.stdout(predicate::str::contains("info"))
		.stdout(predicate::str::contains("key"));
}

#[test]
fn test_import_help() {
	squrl()
		.args(["import", "--help"])
		.assert()
		.success()
		.stdout(predicate::str::contains("postman"))
		.stdout(predicate::str::contains("curl"))
		.stdout(predicate::str::contains("http-file"));
}

#[test]
fn test_theme_help() {
	squrl()
		.args(["theme", "--help"])
		.assert()
		.success()
		.stdout(predicate::str::contains("list"))
		.stdout(predicate::str::contains("preview"))
		.stdout(predicate::str::contains("export"));
}

#[test]
fn test_try_help() {
	squrl()
		.args(["try", "--help"])
		.assert()
		.success()
		.stdout(predicate::str::contains("--url"))
		.stdout(predicate::str::contains("--method"));
}

#[test]
fn test_invalid_subcommand() {
	squrl()
		.arg("nonexistent")
		.assert()
		.failure()
		.stderr(predicate::str::contains("error"));
}
