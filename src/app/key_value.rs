use anyhow::anyhow;
use ratatui::prelude::Stylize;
use thiserror::Error;
use tracing::trace;

use crate::app::files::theme::THEME;
use crate::models::request::KeyValue;

#[derive(Error, Debug)]
pub enum KeyValueError {
	#[error("Key not found")]
	KeyNotFound,
}

pub fn find_key(key_value_array: &[KeyValue], key: &str) -> anyhow::Result<usize> {
	trace!("Trying to find key \"{}\"", key);

	let result = key_value_array
		.iter()
		.position(|key_value| key_value.data.0 == key);

	match result {
		None => {
			trace!("Not found");
			Err(anyhow!(KeyValueError::KeyNotFound))
		}
		Some(index) => {
			trace!("Found");
			Ok(index)
		}
	}
}

pub fn print_key_value_vector(array: &Vec<KeyValue>, prefix: Option<&str>) {
	let prefix = prefix.unwrap_or("");

	for key_value in array {
		let text = format!("{prefix}{}: {}", key_value.data.0, key_value.data.1);

		if key_value.enabled {
			println!("{}", text);
		} else {
			println!("{}", text.fg(THEME.read().ui.secondary_foreground_color));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn create_key_value(key: &str, value: &str) -> KeyValue {
		KeyValue {
			enabled: true,
			data: (key.to_string(), value.to_string()),
		}
	}

	#[test]
	fn test_find_key_exists() {
		let array = vec![
			create_key_value("Content-Type", "application/json"),
			create_key_value("Authorization", "Bearer token"),
		];
		let result = find_key(&array, "Content-Type");
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), 0);
	}

	#[test]
	fn test_find_key_exists_at_end() {
		let array = vec![
			create_key_value("Content-Type", "application/json"),
			create_key_value("Authorization", "Bearer token"),
		];
		let result = find_key(&array, "Authorization");
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), 1);
	}

	#[test]
	fn test_find_key_not_found() {
		let array = vec![create_key_value("Content-Type", "application/json")];
		let result = find_key(&array, "Authorization");
		assert!(result.is_err());
	}

	#[test]
	fn test_find_key_empty_array() {
		let array: Vec<KeyValue> = vec![];
		let result = find_key(&array, "Any-Key");
		assert!(result.is_err());
	}

	#[test]
	fn test_find_key_case_sensitive() {
		let array = vec![create_key_value("Content-Type", "application/json")];
		// Keys are case-sensitive, so "content-type" should not match "Content-Type"
		let result = find_key(&array, "content-type");
		assert!(result.is_err());
	}
}
