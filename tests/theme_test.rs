mod helpers;

use helpers::squrl;
use predicates::prelude::*;

#[test]
fn test_theme_list() {
	squrl()
		.args(["theme", "list"])
		.assert()
		.success()
		.stdout(predicate::str::contains("dracula").or(predicate::str::contains("Dracula")));
}

#[test]
fn test_theme_preview() {
	squrl()
		.args(["theme", "preview", "dracula"])
		.assert()
		.success();
}

#[test]
fn test_theme_preview_nonexistent() {
	squrl()
		.args(["theme", "preview", "nonexistent-theme"])
		.assert()
		.failure();
}

#[test]
fn test_theme_export() {
	// The export goes to ~/.config/squrl/themes/ by default.
	// If it already exists, the command fails with an error -- both outcomes are valid.
	let output = squrl().args(["theme", "export", "dracula"]).assert();

	let status = output.get_output().status;
	if !status.success() {
		// Should only fail if the file already exists
		let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
		assert!(
			stdout.contains("already exists"),
			"Expected 'already exists' error, got: {stdout}"
		);
	}
}
