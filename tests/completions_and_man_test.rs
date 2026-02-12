mod helpers;

use helpers::{squrl, temp_dir};
use std::fs;

#[test]
fn test_completions_bash() {
	let dir = temp_dir();

	squrl()
		.args(["completions", "bash", dir.path().to_str().unwrap()])
		.assert()
		.success();

	// Verify a completion file was created
	let entries: Vec<_> = fs::read_dir(dir.path())
		.unwrap()
		.filter_map(|e| e.ok())
		.collect();
	assert!(
		!entries.is_empty(),
		"Completion file should be created in the output directory"
	);
}

#[test]
fn test_completions_zsh() {
	let dir = temp_dir();

	squrl()
		.args(["completions", "zsh", dir.path().to_str().unwrap()])
		.assert()
		.success();
}

#[test]
fn test_completions_fish() {
	let dir = temp_dir();

	squrl()
		.args(["completions", "fish", dir.path().to_str().unwrap()])
		.assert()
		.success();
}

#[test]
fn test_man_page_generation() {
	let dir = temp_dir();

	squrl()
		.args(["man", dir.path().to_str().unwrap()])
		.assert()
		.success();

	// Verify the man page was created
	let man_path = dir.path().join("squrl.1");
	assert!(man_path.exists(), "Man page file squrl.1 should be created");

	let content = fs::read_to_string(&man_path).unwrap();
	assert!(
		content.contains("squrl") || content.contains("SQURL"),
		"Man page should contain the program name"
	);
}
