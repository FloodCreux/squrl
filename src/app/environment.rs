use anyhow::anyhow;
use chrono::Utc;
use indexmap::IndexMap;
use indexmap::map::MutableKeys;
use parking_lot::RwLock;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, trace};
use uuid::Uuid;

use crate::app::App;
use crate::app::environment::EnvironmentError::{
	EnvironmentNotFound, KeyAlreadyExists, KeyNotFound,
};
use crate::app::files::environment::OS_ENV_VARS;
use crate::models::environment::Environment;

#[derive(Error, Debug)]
pub enum EnvironmentError {
	#[error("Environment not found")]
	EnvironmentNotFound,

	#[error("Key not found")]
	KeyNotFound,

	#[error("Key already exists")]
	KeyAlreadyExists,
}

impl App<'_> {
	pub fn get_selected_env_as_local(&self) -> Option<Arc<RwLock<Environment>>> {
		self.core
			.environments
			.get(self.core.selected_environment)
			.cloned()
	}

	pub fn get_env_as_local_from_index(&self, index: usize) -> Option<Arc<RwLock<Environment>>> {
		self.core.environments.get(index).cloned()
	}

	pub fn find_environment(&self, environment_name: &str) -> anyhow::Result<usize> {
		trace!("Trying to find environment \"{environment_name}\"");

		let result = self
			.core
			.environments
			.iter()
			.position(|environment| environment.read().name == environment_name);

		match result {
			None => {
				trace!("Not found");
				Err(anyhow!(EnvironmentNotFound))
			}
			Some(index) => {
				trace!("Found");
				Ok(index)
			}
		}
	}

	pub fn get_env_value(&mut self, env_index: usize, key: &str) -> anyhow::Result<()> {
		let local_env = self
			.get_env_as_local_from_index(env_index)
			.ok_or_else(|| anyhow!(EnvironmentNotFound))?;

		{
			let env = local_env.read();

			let value = match env.values.get(key) {
				None => return Err(anyhow!(KeyNotFound)),
				Some(value) => value,
			};

			println!("{value}");
		}

		Ok(())
	}

	pub fn set_env_value(
		&mut self,
		env_index: usize,
		key: &str,
		value: String,
	) -> anyhow::Result<()> {
		let local_env = self
			.get_env_as_local_from_index(env_index)
			.ok_or_else(|| anyhow!(EnvironmentNotFound))?;

		{
			let mut env = local_env.write();

			match env.values.get_mut(key) {
				None => return Err(anyhow!(KeyNotFound)),
				Some(old_value) => {
					info!("Environment key \"{key}\" value set to \"{value}\"");

					*old_value = value;
				}
			}
		}

		self.save_environment_to_file(env_index);
		Ok(())
	}

	pub fn set_env_value_by_index(
		&mut self,
		env_index: usize,
		key_index: usize,
		value: String,
	) -> anyhow::Result<()> {
		let local_env = self
			.get_env_as_local_from_index(env_index)
			.ok_or_else(|| anyhow!(EnvironmentNotFound))?;

		{
			let mut env = local_env.write();

			match env.values.get_index_mut(key_index) {
				None => return Err(anyhow!(KeyNotFound)),
				Some((key, old_value)) => {
					info!("Environment key \"{key}\" value set to \"{value}\"");

					*old_value = value;
				}
			}
		}

		self.save_environment_to_file(env_index);
		Ok(())
	}

	pub fn create_env_value(
		&mut self,
		env_index: usize,
		key: Option<String>,
		value: String,
	) -> anyhow::Result<()> {
		let local_env = self
			.get_env_as_local_from_index(env_index)
			.ok_or_else(|| anyhow!(EnvironmentNotFound))?;

		{
			let mut env = local_env.write();

			let key = match key {
				None => format!("KEY_{}", env.values.len()),
				Some(key) => key,
			};

			match env.values.insert(key.clone(), value.clone()) {
				Some(_) => return Err(anyhow!(KeyAlreadyExists)),
				None => info!("Key \"{key}\" with value \"{value}\" added to the environment"),
			}
		}

		self.save_environment_to_file(env_index);
		Ok(())
	}

	pub fn delete_env_key(&mut self, env_index: usize, key: &str) -> anyhow::Result<()> {
		let local_env = self
			.get_env_as_local_from_index(env_index)
			.ok_or_else(|| anyhow!(EnvironmentNotFound))?;

		{
			let mut env = local_env.write();

			match env.values.shift_remove(key) {
				None => return Err(anyhow!(KeyNotFound)),
				Some(_) => info!("Key \"{key}\" deleted from environment"),
			}
		}

		self.save_environment_to_file(env_index);
		Ok(())
	}

	pub fn delete_env_index(&mut self, env_index: usize, index: usize) -> anyhow::Result<()> {
		let local_env = self
			.get_env_as_local_from_index(env_index)
			.ok_or_else(|| anyhow!(EnvironmentNotFound))?;

		{
			let mut env = local_env.write();

			match env.values.shift_remove_index(index) {
				None => return Err(anyhow!(KeyNotFound)),
				Some((key, _)) => info!("Key \"{key}\" deleted from environment"),
			}
		}

		self.save_environment_to_file(env_index);
		Ok(())
	}

	pub fn rename_env_key(
		&mut self,
		env_index: usize,
		key: &str,
		new_key: &str,
	) -> anyhow::Result<()> {
		let local_env = self
			.get_env_as_local_from_index(env_index)
			.ok_or_else(|| anyhow!(EnvironmentNotFound))?;

		{
			let mut env = local_env.write();

			if env.values.get(new_key).is_some() {
				return Err(anyhow!(KeyAlreadyExists));
			}

			let old_index = match env.values.get_index_of(key) {
				None => return Err(anyhow!(KeyNotFound)),
				Some(index) => index,
			};

			let (key, _) = env
				.values
				.get_index_mut2(old_index)
				.ok_or_else(|| anyhow!(KeyNotFound))?;
			*key = new_key.to_string();

			info!("Environment key \"{key}\" renamed to \"{new_key}\"");
		}

		self.save_environment_to_file(env_index);
		Ok(())
	}

	pub fn rename_env_key_by_index(
		&mut self,
		env_index: usize,
		key_index: usize,
		new_key: String,
	) -> anyhow::Result<()> {
		let local_env = self
			.get_env_as_local_from_index(env_index)
			.ok_or_else(|| anyhow!(EnvironmentNotFound))?;

		{
			let mut env = local_env.write();

			if env.values.get(&new_key).is_some() {
				return Err(anyhow!(KeyAlreadyExists));
			}

			let (key, _) = env
				.values
				.get_index_mut2(key_index)
				.ok_or_else(|| anyhow!(KeyNotFound))?;
			let old_key = key.clone();
			*key = new_key.clone();

			info!("Environment key \"{old_key}\" renamed to \"{new_key}\"");
		}

		self.save_environment_to_file(env_index);
		Ok(())
	}

	pub fn replace_env_keys_by_value(&self, input: &String) -> String {
		if self.core.environments.is_empty() {
			return input.to_string();
		}

		let local_env = self.get_selected_env_as_local();

		let env_values = local_env.map(|local_env| {
			let env = local_env.read();
			let mut values = env.values.clone();
			values.extend(OS_ENV_VARS.clone());
			values
		});

		interpolate_env_keys(input, env_values.as_ref())
	}
}

/// Core interpolation logic: replaces `{{KEY}}` placeholders in `input`
/// with values from `env_values`, then replaces built-in variables
/// (`{{NOW}}`, `{{TIMESTAMP}}`, `{{UUIDv4}}`, `{{UUIDv7}}`).
pub fn interpolate_env_keys(input: &str, env_values: Option<&IndexMap<String, String>>) -> String {
	let mut tmp_string = input.to_string();

	if let Some(values) = env_values {
		for (key, value) in values {
			tmp_string = tmp_string.replace(&format!("{{{{{}}}}}", key), value);
		}
	}

	tmp_string = tmp_string
		.replace("{{NOW}}", &Utc::now().to_string())
		.replace("{{TIMESTAMP}}", &Utc::now().timestamp().to_string())
		.replace("{{UUIDv4}}", &Uuid::new_v4().to_string())
		.replace("{{UUIDv7}}", &Uuid::now_v7().to_string());

	tmp_string
}

#[cfg(test)]
mod tests {
	use super::*;

	// ── Basic key replacement ────────────────────────────────────

	#[test]
	fn replaces_single_key() {
		let mut env = IndexMap::new();
		env.insert("API_KEY".to_string(), "secret123".to_string());

		let result = interpolate_env_keys("Bearer {{API_KEY}}", Some(&env));
		assert_eq!(result, "Bearer secret123");
	}

	#[test]
	fn replaces_multiple_keys() {
		let mut env = IndexMap::new();
		env.insert("HOST".to_string(), "api.example.com".to_string());
		env.insert("PORT".to_string(), "8080".to_string());

		let result = interpolate_env_keys("https://{{HOST}}:{{PORT}}/api", Some(&env));
		assert_eq!(result, "https://api.example.com:8080/api");
	}

	#[test]
	fn replaces_same_key_multiple_times() {
		let mut env = IndexMap::new();
		env.insert("TOKEN".to_string(), "abc".to_string());

		let result = interpolate_env_keys("{{TOKEN}}-{{TOKEN}}", Some(&env));
		assert_eq!(result, "abc-abc");
	}

	#[test]
	fn leaves_unknown_keys_as_is() {
		let env = IndexMap::new();

		let result = interpolate_env_keys("{{UNKNOWN_KEY}}", Some(&env));
		assert_eq!(result, "{{UNKNOWN_KEY}}");
	}

	#[test]
	fn no_env_values_leaves_placeholders() {
		let result = interpolate_env_keys("{{KEY}}", None);
		// Only built-in keys are replaced; user keys remain
		assert_eq!(result, "{{KEY}}");
	}

	#[test]
	fn empty_input_returns_empty() {
		let env = IndexMap::new();
		let result = interpolate_env_keys("", Some(&env));
		assert_eq!(result, "");
	}

	#[test]
	fn input_without_placeholders_unchanged() {
		let mut env = IndexMap::new();
		env.insert("KEY".to_string(), "value".to_string());

		let result = interpolate_env_keys("no placeholders here", Some(&env));
		assert_eq!(result, "no placeholders here");
	}

	#[test]
	fn handles_empty_value() {
		let mut env = IndexMap::new();
		env.insert("EMPTY".to_string(), String::new());

		let result = interpolate_env_keys("pre-{{EMPTY}}-post", Some(&env));
		assert_eq!(result, "pre--post");
	}

	#[test]
	fn handles_key_with_special_chars_in_value() {
		let mut env = IndexMap::new();
		env.insert(
			"URL".to_string(),
			"https://example.com/path?q=1&r=2".to_string(),
		);

		let result = interpolate_env_keys("{{URL}}", Some(&env));
		assert_eq!(result, "https://example.com/path?q=1&r=2");
	}

	// ── Built-in variable replacement ────────────────────────────

	#[test]
	fn replaces_now_with_datetime() {
		let result = interpolate_env_keys("time: {{NOW}}", None);
		assert!(!result.contains("{{NOW}}"), "result was: {result}");
		assert!(result.starts_with("time: "));
		// The result should contain a date-like string (at least year)
		assert!(result.contains("20"), "result was: {result}");
	}

	#[test]
	fn replaces_timestamp_with_number() {
		let result = interpolate_env_keys("ts: {{TIMESTAMP}}", None);
		assert!(!result.contains("{{TIMESTAMP}}"), "result was: {result}");
		// The timestamp part should be numeric
		let ts_part = result.strip_prefix("ts: ").unwrap();
		assert!(ts_part.parse::<i64>().is_ok(), "result was: {result}");
	}

	#[test]
	fn replaces_uuid_v4() {
		let result = interpolate_env_keys("id: {{UUIDv4}}", None);
		assert!(!result.contains("{{UUIDv4}}"), "result was: {result}");
		let uuid_part = result.strip_prefix("id: ").unwrap();
		// UUIDv4 format: 8-4-4-4-12 hex chars
		assert_eq!(uuid_part.len(), 36, "result was: {result}");
		assert_eq!(uuid_part.matches('-').count(), 4, "result was: {result}");
	}

	#[test]
	fn replaces_uuid_v7() {
		let result = interpolate_env_keys("id: {{UUIDv7}}", None);
		assert!(!result.contains("{{UUIDv7}}"), "result was: {result}");
		let uuid_part = result.strip_prefix("id: ").unwrap();
		assert_eq!(uuid_part.len(), 36, "result was: {result}");
	}

	#[test]
	fn each_uuid_call_generates_unique_value() {
		let result1 = interpolate_env_keys("{{UUIDv4}}", None);
		let result2 = interpolate_env_keys("{{UUIDv4}}", None);
		assert_ne!(result1, result2);
	}

	// ── Mixed env + built-in ─────────────────────────────────────

	#[test]
	fn replaces_both_env_keys_and_builtins() {
		let mut env = IndexMap::new();
		env.insert("HOST".to_string(), "api.test.com".to_string());

		let result = interpolate_env_keys("https://{{HOST}}/{{UUIDv4}}", Some(&env));
		assert!(
			result.starts_with("https://api.test.com/"),
			"result was: {result}"
		);
		assert!(!result.contains("{{UUIDv4}}"), "result was: {result}");
	}

	// ── Single brace edge cases ──────────────────────────────────

	#[test]
	fn single_braces_not_treated_as_placeholder() {
		let env = IndexMap::new();
		let result = interpolate_env_keys("{KEY}", Some(&env));
		assert_eq!(result, "{KEY}");
	}

	#[test]
	fn triple_braces_partial_replacement() {
		let mut env = IndexMap::new();
		env.insert("KEY".to_string(), "val".to_string());

		// {{{KEY}}} => The inner {{KEY}} gets replaced, leaving {val}
		let result = interpolate_env_keys("{{{KEY}}}", Some(&env));
		assert_eq!(result, "{val}");
	}
}
