use anyhow::anyhow;
use chrono::Utc;
use indexmap::IndexMap;
use indexmap::map::MutableKeys;
use parking_lot::RwLock;
use std::path::PathBuf;
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

		match local_env {
			Some(local_env) => {
				let env = local_env.read();
				interpolate_env_keys(input, &[&env.values, &OS_ENV_VARS])
			}
			None => interpolate_env_keys(input, &[]),
		}
	}

	/// Get the active environment values for a collection, if the collection
	/// has a selected environment. Returns `None` if the collection has no
	/// environments or no environment is selected.
	pub fn get_collection_env_values(
		&self,
		collection_index: usize,
	) -> Option<IndexMap<String, String>> {
		let collection = self.core.collections.get(collection_index)?;
		let selected_name = collection.selected_environment.as_ref()?;

		collection
			.environments
			.iter()
			.find(|env| &env.name == selected_name)
			.map(|env| env.values.clone())
	}

	/// Replace `{{KEY}}` placeholders using the collection's environment (highest priority),
	/// then the global environment, then OS env vars, then built-in variables.
	///
	/// This is the primary interpolation method used during request preparation.
	pub fn replace_env_keys_for_collection(&self, input: &str, collection_index: usize) -> String {
		let collection_env = self.get_collection_env_values(collection_index);
		let global_env = self.get_selected_env_as_local();

		let mut maps: Vec<&IndexMap<String, String>> = Vec::new();

		// Collection env takes highest priority
		if let Some(ref coll_env) = collection_env {
			maps.push(coll_env);
		}

		// Global env is next
		let global_env_guard;
		if let Some(ref env_lock) = global_env {
			global_env_guard = env_lock.read();
			maps.push(&global_env_guard.values);
		}

		// OS env vars are last
		maps.push(&OS_ENV_VARS);

		interpolate_env_keys(input, &maps)
	}

	// ── Collection-scoped environment operations ──────────────────────────

	/// Find a collection environment by name. Returns its index in the collection's
	/// environments vector.
	pub fn find_collection_environment(
		&self,
		collection_index: usize,
		env_name: &str,
	) -> anyhow::Result<usize> {
		let collection = self
			.core
			.collections
			.get(collection_index)
			.ok_or_else(|| anyhow!("Collection not found"))?;

		collection
			.environments
			.iter()
			.position(|env| env.name == env_name)
			.ok_or_else(|| anyhow!(EnvironmentNotFound))
	}

	/// Create a new environment in a collection.
	pub fn create_collection_environment(
		&mut self,
		collection_index: usize,
		env_name: String,
	) -> anyhow::Result<()> {
		let collection = self
			.core
			.collections
			.get(collection_index)
			.ok_or_else(|| anyhow!("Collection not found"))?;

		if collection.environments.iter().any(|e| e.name == env_name) {
			return Err(anyhow!(
				"Collection environment \"{env_name}\" already exists"
			));
		}

		let new_env = Environment {
			name: env_name.clone(),
			values: IndexMap::new(),
			path: PathBuf::new(),
		};

		self.core.collections[collection_index]
			.environments
			.push(new_env);

		info!("Collection environment \"{env_name}\" created");
		self.save_collection_to_file(collection_index);
		Ok(())
	}

	/// Delete an environment from a collection.
	pub fn delete_collection_environment(
		&mut self,
		collection_index: usize,
		env_name: &str,
	) -> anyhow::Result<()> {
		let env_idx = self.find_collection_environment(collection_index, env_name)?;

		self.core.collections[collection_index]
			.environments
			.remove(env_idx);

		// If the deleted env was the selected one, clear the selection
		if self.core.collections[collection_index]
			.selected_environment
			.as_deref()
			== Some(env_name)
		{
			self.core.collections[collection_index].selected_environment = None;
		}

		info!("Collection environment \"{env_name}\" deleted");
		self.save_collection_to_file(collection_index);
		Ok(())
	}

	/// Select a collection environment by name. Pass `None` to deselect.
	pub fn select_collection_environment(
		&mut self,
		collection_index: usize,
		env_name: Option<String>,
	) -> anyhow::Result<()> {
		if let Some(ref name) = env_name {
			// Verify the environment exists
			self.find_collection_environment(collection_index, name)?;
		}

		self.core.collections[collection_index].selected_environment = env_name.clone();

		match env_name {
			Some(name) => info!("Collection environment set to \"{name}\""),
			None => info!("Collection environment deselected"),
		}

		self.save_collection_to_file(collection_index);
		Ok(())
	}

	/// Get a value from a collection environment.
	pub fn get_collection_env_value(
		&self,
		collection_index: usize,
		env_name: &str,
		key: &str,
	) -> anyhow::Result<String> {
		let env_idx = self.find_collection_environment(collection_index, env_name)?;
		let env = &self.core.collections[collection_index].environments[env_idx];

		env.values
			.get(key)
			.cloned()
			.ok_or_else(|| anyhow!(KeyNotFound))
	}

	/// Set a value in a collection environment (key must already exist).
	pub fn set_collection_env_value(
		&mut self,
		collection_index: usize,
		env_name: &str,
		key: &str,
		value: String,
	) -> anyhow::Result<()> {
		let env_idx = self.find_collection_environment(collection_index, env_name)?;
		let env = &mut self.core.collections[collection_index].environments[env_idx];

		match env.values.get_mut(key) {
			None => return Err(anyhow!(KeyNotFound)),
			Some(old_value) => {
				info!("Collection env key \"{key}\" value set to \"{value}\"");
				*old_value = value;
			}
		}

		self.save_collection_to_file(collection_index);
		Ok(())
	}

	/// Add a new key-value pair to a collection environment.
	pub fn create_collection_env_value(
		&mut self,
		collection_index: usize,
		env_name: &str,
		key: String,
		value: String,
	) -> anyhow::Result<()> {
		let env_idx = self.find_collection_environment(collection_index, env_name)?;
		let env = &mut self.core.collections[collection_index].environments[env_idx];

		match env.values.insert(key.clone(), value.clone()) {
			Some(_) => return Err(anyhow!(KeyAlreadyExists)),
			None => info!("Key \"{key}\" with value \"{value}\" added to collection environment"),
		}

		self.save_collection_to_file(collection_index);
		Ok(())
	}

	/// Delete a key from a collection environment.
	pub fn delete_collection_env_key(
		&mut self,
		collection_index: usize,
		env_name: &str,
		key: &str,
	) -> anyhow::Result<()> {
		let env_idx = self.find_collection_environment(collection_index, env_name)?;
		let env = &mut self.core.collections[collection_index].environments[env_idx];

		match env.values.shift_remove(key) {
			None => return Err(anyhow!(KeyNotFound)),
			Some(_) => info!("Key \"{key}\" deleted from collection environment"),
		}

		self.save_collection_to_file(collection_index);
		Ok(())
	}

	/// Rename a key in a collection environment.
	pub fn rename_collection_env_key(
		&mut self,
		collection_index: usize,
		env_name: &str,
		key: &str,
		new_key: &str,
	) -> anyhow::Result<()> {
		let env_idx = self.find_collection_environment(collection_index, env_name)?;
		let env = &mut self.core.collections[collection_index].environments[env_idx];

		if env.values.get(new_key).is_some() {
			return Err(anyhow!(KeyAlreadyExists));
		}

		let old_index = match env.values.get_index_of(key) {
			None => return Err(anyhow!(KeyNotFound)),
			Some(index) => index,
		};

		let (k, _) = env
			.values
			.get_index_mut2(old_index)
			.ok_or_else(|| anyhow!(KeyNotFound))?;
		*k = new_key.to_string();

		info!("Collection env key \"{key}\" renamed to \"{new_key}\"");

		self.save_collection_to_file(collection_index);
		Ok(())
	}

	/// Cycle to the next collection environment (for TUI). Wraps around.
	/// If the collection has no environments, does nothing.
	pub fn tui_next_collection_environment(&mut self, collection_index: usize) {
		let collection = &self.core.collections[collection_index];

		if collection.environments.is_empty() {
			return;
		}

		let current_idx = collection
			.selected_environment
			.as_ref()
			.and_then(|name| collection.environments.iter().position(|e| &e.name == name))
			.unwrap_or(0);

		let next_idx = if current_idx + 1 < collection.environments.len() {
			current_idx + 1
		} else {
			0
		};

		self.core.collections[collection_index].selected_environment =
			Some(collection.environments[next_idx].name.clone());

		self.save_collection_to_file(collection_index);
	}
}

/// Core interpolation logic: replaces `{{KEY}}` placeholders in `input`
/// with values from the given environment maps (checked in order), then
/// replaces built-in variables (`{{NOW}}`, `{{TIMESTAMP}}`, `{{UUIDv4}}`,
/// `{{UUIDv7}}`).
///
/// Accepts a slice of `IndexMap` references so callers can pass multiple
/// maps (e.g. user env + OS env) without cloning or merging them.
pub fn interpolate_env_keys(input: &str, env_maps: &[&IndexMap<String, String>]) -> String {
	let mut tmp_string = input.to_string();

	for map in env_maps {
		for (key, value) in *map {
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

		let result = interpolate_env_keys("Bearer {{API_KEY}}", &[&env]);
		assert_eq!(result, "Bearer secret123");
	}

	#[test]
	fn replaces_multiple_keys() {
		let mut env = IndexMap::new();
		env.insert("HOST".to_string(), "api.example.com".to_string());
		env.insert("PORT".to_string(), "8080".to_string());

		let result = interpolate_env_keys("https://{{HOST}}:{{PORT}}/api", &[&env]);
		assert_eq!(result, "https://api.example.com:8080/api");
	}

	#[test]
	fn replaces_same_key_multiple_times() {
		let mut env = IndexMap::new();
		env.insert("TOKEN".to_string(), "abc".to_string());

		let result = interpolate_env_keys("{{TOKEN}}-{{TOKEN}}", &[&env]);
		assert_eq!(result, "abc-abc");
	}

	#[test]
	fn leaves_unknown_keys_as_is() {
		let env = IndexMap::new();

		let result = interpolate_env_keys("{{UNKNOWN_KEY}}", &[&env]);
		assert_eq!(result, "{{UNKNOWN_KEY}}");
	}

	#[test]
	fn no_env_values_leaves_placeholders() {
		let result = interpolate_env_keys("{{KEY}}", &[]);
		// Only built-in keys are replaced; user keys remain
		assert_eq!(result, "{{KEY}}");
	}

	#[test]
	fn empty_input_returns_empty() {
		let env = IndexMap::new();
		let result = interpolate_env_keys("", &[&env]);
		assert_eq!(result, "");
	}

	#[test]
	fn input_without_placeholders_unchanged() {
		let mut env = IndexMap::new();
		env.insert("KEY".to_string(), "value".to_string());

		let result = interpolate_env_keys("no placeholders here", &[&env]);
		assert_eq!(result, "no placeholders here");
	}

	#[test]
	fn handles_empty_value() {
		let mut env = IndexMap::new();
		env.insert("EMPTY".to_string(), String::new());

		let result = interpolate_env_keys("pre-{{EMPTY}}-post", &[&env]);
		assert_eq!(result, "pre--post");
	}

	#[test]
	fn handles_key_with_special_chars_in_value() {
		let mut env = IndexMap::new();
		env.insert(
			"URL".to_string(),
			"https://example.com/path?q=1&r=2".to_string(),
		);

		let result = interpolate_env_keys("{{URL}}", &[&env]);
		assert_eq!(result, "https://example.com/path?q=1&r=2");
	}

	// ── Built-in variable replacement ────────────────────────────

	#[test]
	fn replaces_now_with_datetime() {
		let result = interpolate_env_keys("time: {{NOW}}", &[]);
		assert!(!result.contains("{{NOW}}"), "result was: {result}");
		assert!(result.starts_with("time: "));
		// The result should contain a date-like string (at least year)
		assert!(result.contains("20"), "result was: {result}");
	}

	#[test]
	fn replaces_timestamp_with_number() {
		let result = interpolate_env_keys("ts: {{TIMESTAMP}}", &[]);
		assert!(!result.contains("{{TIMESTAMP}}"), "result was: {result}");
		// The timestamp part should be numeric
		let ts_part = result.strip_prefix("ts: ").unwrap();
		assert!(ts_part.parse::<i64>().is_ok(), "result was: {result}");
	}

	#[test]
	fn replaces_uuid_v4() {
		let result = interpolate_env_keys("id: {{UUIDv4}}", &[]);
		assert!(!result.contains("{{UUIDv4}}"), "result was: {result}");
		let uuid_part = result.strip_prefix("id: ").unwrap();
		// UUIDv4 format: 8-4-4-4-12 hex chars
		assert_eq!(uuid_part.len(), 36, "result was: {result}");
		assert_eq!(uuid_part.matches('-').count(), 4, "result was: {result}");
	}

	#[test]
	fn replaces_uuid_v7() {
		let result = interpolate_env_keys("id: {{UUIDv7}}", &[]);
		assert!(!result.contains("{{UUIDv7}}"), "result was: {result}");
		let uuid_part = result.strip_prefix("id: ").unwrap();
		assert_eq!(uuid_part.len(), 36, "result was: {result}");
	}

	#[test]
	fn each_uuid_call_generates_unique_value() {
		let result1 = interpolate_env_keys("{{UUIDv4}}", &[]);
		let result2 = interpolate_env_keys("{{UUIDv4}}", &[]);
		assert_ne!(result1, result2);
	}

	// ── Mixed env + built-in ─────────────────────────────────────

	#[test]
	fn replaces_both_env_keys_and_builtins() {
		let mut env = IndexMap::new();
		env.insert("HOST".to_string(), "api.test.com".to_string());

		let result = interpolate_env_keys("https://{{HOST}}/{{UUIDv4}}", &[&env]);
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
		let result = interpolate_env_keys("{KEY}", &[&env]);
		assert_eq!(result, "{KEY}");
	}

	#[test]
	fn triple_braces_partial_replacement() {
		let mut env = IndexMap::new();
		env.insert("KEY".to_string(), "val".to_string());

		// {{{KEY}}} => The inner {{KEY}} gets replaced, leaving {val}
		let result = interpolate_env_keys("{{{KEY}}}", &[&env]);
		assert_eq!(result, "{val}");
	}

	// ── Multiple maps ───────────────────────────────────────────

	#[test]
	fn multiple_maps_are_checked_in_order() {
		let mut user_env = IndexMap::new();
		user_env.insert("KEY".to_string(), "user_value".to_string());

		let mut os_env = IndexMap::new();
		os_env.insert("KEY".to_string(), "os_value".to_string());
		os_env.insert("OS_ONLY".to_string(), "from_os".to_string());

		// User env is checked first, so KEY resolves to user_value.
		// After user env replaces {{KEY}}, os_env's KEY replacement is a no-op.
		// OS_ONLY is only in the second map.
		let result = interpolate_env_keys("{{KEY}} and {{OS_ONLY}}", &[&user_env, &os_env]);
		assert_eq!(result, "user_value and from_os");
	}
}
