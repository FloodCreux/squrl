use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::from_utf8;
use std::sync::Arc;

use anyhow::Context;
use indexmap::IndexMap;
use parking_lot::RwLock;
use std::sync::LazyLock;
use tracing::{info, trace, warn};

use crate::app::app::App;
use crate::app::files::utils::write_via_temp_file;
use crate::cli::args::ARGS;
use crate::models::environment::Environment;

pub static OS_ENV_VARS: LazyLock<IndexMap<String, String>> =
	LazyLock::new(|| env::vars().collect());

impl App<'_> {
	/// Add the environment file to the app environments
	pub fn add_environment_from_file(&mut self, path_buf: &Path) -> anyhow::Result<()> {
		let file_name = path_buf
			.file_name()
			.and_then(|n| n.to_str())
			.with_context(|| {
				format!(
					"Could not extract file name from path \"{}\"",
					path_buf.display()
				)
			})?
			.to_string()
			.replace(".env.", "");

		trace!("Trying to open \"{}\" env file", path_buf.display());

		let env_file: File = File::open(path_buf).with_context(|| {
			format!("Could not open environment file \"{}\"", path_buf.display())
		})?;

		let environment = Environment {
			name: file_name,
			values: read_environment_from_file(env_file),
			path: path_buf.to_path_buf(),
		};

		self.core
			.environments
			.push(Arc::new(RwLock::new(environment)));

		trace!("Environment file parsed!");
		Ok(())
	}

	pub fn save_environment_to_file(&mut self, env_index: usize) {
		let environment = self.core.environments[env_index].read();

		save_environment_to_file(&environment);
	}
}

fn read_environment_from_file(file: File) -> IndexMap<String, String> {
	let reader = BufReader::new(file);
	let mut environment_values = IndexMap::new();

	for line in reader.lines().map_while(Result::ok) {
		if let Some((key, value)) = parse_line(line.trim().as_bytes()) {
			environment_values.insert(key, value);
		}
	}

	environment_values
}

// Code from the EnvFile crate
fn parse_line(entry: &[u8]) -> Option<(String, String)> {
	from_utf8(entry).ok().and_then(|l| {
		let line = l.trim();

		// Ignore comment line
		if line.starts_with('#') {
			return None;
		}

		let vline = line.as_bytes();

		vline.iter().position(|&x| x == b'=').and_then(|pos| {
			from_utf8(&vline[..pos]).ok().and_then(|x| {
				from_utf8(&vline[pos + 1..])
					.ok()
					.map(|right| (x.to_owned(), unquote_env_value(right)))
			})
		})
	})
}

/// Strip surrounding quotes from an env-file value and process escape sequences.
///
/// - Single-quoted values are returned verbatim (no escape processing).
/// - Double-quoted values support `\\`, `\"`, `\n`, `\r`, `\t`.
/// - Unquoted values are returned as-is.
fn unquote_env_value(s: &str) -> String {
	if s.len() >= 2 {
		if s.starts_with('\'') && s.ends_with('\'') {
			// Single-quoted: no escape processing
			return s[1..s.len() - 1].to_string();
		}
		if s.starts_with('"') && s.ends_with('"') {
			// Double-quoted: process escape sequences
			let inner = &s[1..s.len() - 1];
			let mut result = String::with_capacity(inner.len());
			let mut chars = inner.chars();
			while let Some(ch) = chars.next() {
				if ch == '\\' {
					match chars.next() {
						Some('n') => result.push('\n'),
						Some('r') => result.push('\r'),
						Some('t') => result.push('\t'),
						Some('\\') => result.push('\\'),
						Some('"') => result.push('"'),
						Some(other) => {
							result.push('\\');
							result.push(other);
						}
						None => result.push('\\'),
					}
				} else {
					result.push(ch);
				}
			}
			return result;
		}
	}
	// Unquoted: return as-is
	s.to_string()
}

/// Save app environment in a file through a temporary file.
/// Logs a warning on failure rather than panicking.
pub fn save_environment_to_file(environment: &Environment) {
	if !ARGS.should_save {
		warn!("Dry-run, not saving the environment");
		return;
	}

	info!("Saving environment \"{}\"", environment.name);

	let mut data: String = environment
		.values
		.iter()
		.map(|(key, value)| format!("{key}={value}\n"))
		.collect();

	// Remove trailing \n
	data.pop();

	if let Err(e) = write_via_temp_file(&environment.path, data.as_bytes()) {
		warn!("Could not save environment file: {e}");
		return;
	}

	trace!("Environment saved")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_line_valid() {
		let result = parse_line(b"KEY=value");
		assert_eq!(result, Some(("KEY".to_string(), "value".to_string())));
	}

	#[test]
	fn test_parse_line_comment() {
		let result = parse_line(b"# this is a comment");
		assert_eq!(result, None);
	}

	#[test]
	fn test_parse_line_empty() {
		let result = parse_line(b"");
		assert_eq!(result, None);
	}

	#[test]
	fn test_parse_line_whitespace_only() {
		let result = parse_line(b"   ");
		assert_eq!(result, None);
	}

	#[test]
	fn test_parse_line_quoted_value() {
		let result = parse_line(b"KEY=\"quoted value\"");
		assert_eq!(
			result,
			Some(("KEY".to_string(), "quoted value".to_string()))
		);
	}

	#[test]
	fn test_parse_line_single_quoted_value() {
		let result = parse_line(b"KEY='single quoted'");
		assert_eq!(
			result,
			Some(("KEY".to_string(), "single quoted".to_string()))
		);
	}

	#[test]
	fn test_parse_line_equals_in_value() {
		let result = parse_line(b"KEY=a=b=c");
		assert_eq!(result, Some(("KEY".to_string(), "a=b=c".to_string())));
	}

	#[test]
	fn test_parse_line_no_equals() {
		let result = parse_line(b"INVALID");
		assert_eq!(result, None);
	}

	#[test]
	fn test_parse_line_whitespace_trimmed() {
		let result = parse_line(b"  KEY=value  ");
		assert_eq!(result, Some(("KEY".to_string(), "value".to_string())));
	}

	#[test]
	fn test_parse_line_empty_value() {
		let result = parse_line(b"KEY=");
		assert_eq!(result, Some(("KEY".to_string(), "".to_string())));
	}

	#[test]
	fn test_parse_line_comment_with_leading_whitespace() {
		let result = parse_line(b"  # indented comment");
		assert_eq!(result, None);
	}

	#[test]
	fn test_parse_line_url_value() {
		let result = parse_line(b"API_URL=https://api.example.com/v1");
		assert_eq!(
			result,
			Some((
				"API_URL".to_string(),
				"https://api.example.com/v1".to_string()
			))
		);
	}

	// ── unquote_env_value ────────────────────────────────────────

	#[test]
	fn unquote_unquoted_value() {
		assert_eq!(unquote_env_value("plain"), "plain");
	}

	#[test]
	fn unquote_double_quoted_basic() {
		assert_eq!(unquote_env_value("\"hello world\""), "hello world");
	}

	#[test]
	fn unquote_single_quoted_basic() {
		assert_eq!(unquote_env_value("'hello world'"), "hello world");
	}

	#[test]
	fn unquote_double_quoted_escape_newline() {
		assert_eq!(unquote_env_value("\"line1\\nline2\""), "line1\nline2");
	}

	#[test]
	fn unquote_double_quoted_escape_tab() {
		assert_eq!(unquote_env_value("\"col1\\tcol2\""), "col1\tcol2");
	}

	#[test]
	fn unquote_double_quoted_escape_carriage_return() {
		assert_eq!(unquote_env_value("\"a\\rb\""), "a\rb");
	}

	#[test]
	fn unquote_double_quoted_escaped_backslash() {
		assert_eq!(
			unquote_env_value("\"path\\\\to\\\\file\""),
			"path\\to\\file"
		);
	}

	#[test]
	fn unquote_double_quoted_escaped_quote() {
		assert_eq!(
			unquote_env_value("\"she said \\\"hi\\\"\""),
			"she said \"hi\""
		);
	}

	#[test]
	fn unquote_double_quoted_unknown_escape_preserved() {
		// Unknown escape like \x should keep the backslash
		assert_eq!(unquote_env_value("\"hello\\xworld\""), "hello\\xworld");
	}

	#[test]
	fn unquote_double_quoted_trailing_backslash() {
		// Trailing backslash with no following char
		assert_eq!(unquote_env_value("\"trailing\\\""), "trailing\\");
	}

	#[test]
	fn unquote_single_quoted_no_escape_processing() {
		// Single quotes should NOT process escape sequences
		assert_eq!(unquote_env_value("'no\\nescape'"), "no\\nescape");
	}

	#[test]
	fn unquote_empty_string() {
		assert_eq!(unquote_env_value(""), "");
	}

	#[test]
	fn unquote_single_char() {
		assert_eq!(unquote_env_value("x"), "x");
	}

	#[test]
	fn unquote_mismatched_quotes_not_stripped() {
		// Starts with " but ends with ' — not matching, returned as-is
		assert_eq!(unquote_env_value("\"mixed'"), "\"mixed'");
	}

	#[test]
	fn unquote_just_double_quotes() {
		assert_eq!(unquote_env_value("\"\""), "");
	}

	#[test]
	fn unquote_just_single_quotes() {
		assert_eq!(unquote_env_value("''"), "");
	}

	#[test]
	fn unquote_double_quoted_multiple_escapes() {
		assert_eq!(unquote_env_value("\"\\n\\t\\r\\\\\""), "\n\t\r\\");
	}

	// ── parse_line with escape sequences in values ───────────────

	#[test]
	fn test_parse_line_double_quoted_with_newline() {
		let result = parse_line(b"MSG=\"hello\\nworld\"");
		assert_eq!(
			result,
			Some(("MSG".to_string(), "hello\nworld".to_string()))
		);
	}

	#[test]
	fn test_parse_line_single_quoted_preserves_backslash() {
		let result = parse_line(b"MSG='hello\\nworld'");
		assert_eq!(
			result,
			Some(("MSG".to_string(), "hello\\nworld".to_string()))
		);
	}
}
