use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::from_utf8;
use std::sync::Arc;

use indexmap::IndexMap;
use parking_lot::RwLock;
use std::sync::LazyLock;
use tracing::{info, trace, warn};

use crate::app::app::App;
use crate::app::files::utils::write_via_temp_file;
use crate::cli::args::ARGS;
use crate::errors::panic_error;
use crate::models::environment::Environment;

pub static OS_ENV_VARS: LazyLock<IndexMap<String, String>> =
	LazyLock::new(|| env::vars().collect());

impl App<'_> {
	/// Add the environment file to the app environments
	pub fn add_environment_from_file(&mut self, path_buf: &Path) {
		let file_name = path_buf
			.file_name()
			.unwrap()
			.to_str()
			.unwrap()
			.to_string()
			.replace(".env.", "");

		trace!("Trying to open \"{}\" env file", path_buf.display());

		let env_file: File = match File::open(path_buf) {
			Ok(env_file) => env_file,
			Err(e) => panic_error(format!("Could not open environment file\n\t{e}")),
		};

		let environment = Environment {
			name: file_name,
			values: read_environment_from_file(env_file),
			path: path_buf.to_path_buf(),
		};

		self.environments.push(Arc::new(RwLock::new(environment)));

		trace!("Environment file parsed!");
	}

	pub fn save_environment_to_file(&mut self, env_index: usize) {
		let environment = self.environments[env_index].read();

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

/// Save app environment in a file through a temporary file
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

	write_via_temp_file(&environment.path, data.as_bytes())
		.expect("Could not save environment file");

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
}
