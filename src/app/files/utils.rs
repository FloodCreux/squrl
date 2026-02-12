use directories::UserDirs;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Write `data` to `target_path` atomically by first writing to a temporary file
/// in the same directory, then renaming it over the target.
pub fn write_via_temp_file(target_path: &Path, data: &[u8]) -> std::io::Result<()> {
	let file_name = target_path
		.file_name()
		.and_then(|n| n.to_str())
		.unwrap_or("file");
	let temp_file_name = format!("{file_name}_");
	let temp_file_path = target_path.with_file_name(temp_file_name);

	let mut temp_file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(&temp_file_path)?;

	temp_file.write_all(data)?;
	temp_file.flush()?;

	fs::rename(temp_file_path, target_path)?;
	Ok(())
}

pub fn expand_tilde(path_buf: PathBuf) -> PathBuf {
	if !path_buf.starts_with("~/") {
		return path_buf;
	}

	match UserDirs::new() {
		Some(user_dirs) => {
			let mut home_dir = user_dirs.home_dir().to_path_buf();
			home_dir.push(
				path_buf
					.strip_prefix("~/")
					.expect("path should start with ~/"),
			);
			home_dir
		}
		None => panic!("No home directory found when trying to expand \"~\""),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_expand_tilde_with_tilde() {
		let path = PathBuf::from("~/foo/bar");
		let expanded = expand_tilde(path);

		// Should expand to home directory + foo/bar
		let user_dirs = UserDirs::new().expect("Home directory should exist in test environment");
		let expected = user_dirs.home_dir().join("foo/bar");
		assert_eq!(expanded, expected);
	}

	#[test]
	fn test_expand_tilde_without_tilde() {
		let path = PathBuf::from("/absolute/path");
		let expanded = expand_tilde(path.clone());
		assert_eq!(expanded, path);
	}

	#[test]
	fn test_expand_tilde_relative_path() {
		let path = PathBuf::from("relative/path");
		let expanded = expand_tilde(path.clone());
		assert_eq!(expanded, path);
	}

	#[test]
	fn test_expand_tilde_just_tilde_no_slash() {
		// "~user" should NOT be expanded (only "~/" is expanded)
		let path = PathBuf::from("~user/foo");
		let expanded = expand_tilde(path.clone());
		assert_eq!(expanded, path);
	}

	#[test]
	fn test_expand_tilde_nested_path() {
		let path = PathBuf::from("~/a/b/c/d");
		let expanded = expand_tilde(path);

		let user_dirs = UserDirs::new().expect("Home directory should exist in test environment");
		let expected = user_dirs.home_dir().join("a/b/c/d");
		assert_eq!(expanded, expected);
	}

	#[test]
	fn test_expand_tilde_just_home() {
		let path = PathBuf::from("~/");
		let expanded = expand_tilde(path);

		let user_dirs = UserDirs::new().expect("Home directory should exist in test environment");
		let expected = user_dirs.home_dir().join("");
		assert_eq!(expanded, expected);
	}
}
