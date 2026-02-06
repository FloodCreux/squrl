use std::sync::Arc;

use anyhow::anyhow;
use indexmap::map::MutableKeys;
use parking_lot::RwLock;
use rayon::prelude::*;
use thiserror::Error;
use tracing::{info, trace};

use crate::app::app::App;
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
	pub fn get_env_as_local_from_index(&self, index: usize) -> Option<Arc<RwLock<Environment>>> {
		match self.environments.get(index) {
			None => None,
			Some(env) => Some(env.clone()),
		}
	}

	pub fn find_environment(&self, environment_name: &str) -> anyhow::Result<usize> {
		trace!("Trying to find environment \"{environment_name}\"");

		let result = self
			.environments
			.par_iter()
			.position_first(|environment| environment.read().name == environment_name);

		match result {
			None => {
				trace!("Not found!");
				Err(anyhow!(EnvironmentError::EnvironmentNotFound))
			}
			Some(index) => {
				trace!("Found!");
				Ok(index)
			}
		}
	}

	pub fn get_env_value(&self, env_index: usize, key: &str) -> anyhow::Result<()> {
		let local_env = self.get_env_as_local_from_index(env_index).unwrap();

		{
			let env = local_env.read();

			let value = match env.values.get(key) {
				None => return Err(anyhow!(EnvironmentError::KeyNotFound)),
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
		let local_env = self.get_env_as_local_from_index(env_index).unwrap();

		{
			let mut env = local_env.write();

			match env.values.get_mut(key) {
				None => return Err(anyhow!(EnvironmentError::KeyNotFound)),
				Some(old_value) => {
					info!("Environment key: \"{key}\" value set to \"{value}\"");
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
		let local_env = self.get_env_as_local_from_index(env_index).unwrap();

		{
			let mut env = local_env.write();

			match env.values.get_index_mut(key_index) {
				None => return Err(anyhow!(EnvironmentError::KeyNotFound)),
				Some((key, old_value)) => {
					info!("Environment key: \"{key}\" value set to \"{value}\"");
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
		let local_env = self.get_env_as_local_from_index(env_index).unwrap();

		{
			let mut env = local_env.write();

			let key = match key {
				None => format!("KEY_{}", env.values.len()),
				Some(key) => key,
			};

			match env.values.insert(key.clone(), value.clone()) {
				Some(_) => return Err(anyhow!(EnvironmentError::KeyAlreadyExists)),
				None => info!("Environment key: \"{key}\" value set to \"{value}\""),
			}
		}

		self.save_environment_to_file(env_index);
		Ok(())
	}

	pub fn delete_env_key(&mut self, env_index: usize, key: &str) -> anyhow::Result<()> {
		let local_env = self.get_env_as_local_from_index(env_index).unwrap();

		{
			let mut env = local_env.write();

			match env.values.shift_remove(key) {
				None => return Err(anyhow!(EnvironmentError::KeyNotFound)),
				Some(_) => info!("Key \"{key}\" deleted from environment"),
			};
		}

		self.save_environment_to_file(env_index);
		Ok(())
	}

	pub fn delete_env_index(&mut self, env_index: usize, index: usize) -> anyhow::Result<()> {
		let local_env = self.get_env_as_local_from_index(env_index).unwrap();

		{
			let mut env = local_env.write();

			match env.values.shift_remove_index(index) {
				None => return Err(anyhow!(EnvironmentError::KeyNotFound)),
				Some((key, _)) => info!("Key \"{key}\" deleted from environment"),
			};
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
		let local_env = self.get_env_as_local_from_index(env_index).unwrap();

		{
			let mut env = local_env.write();

			if env.values.get(new_key).is_some() {
				return Err(anyhow!(EnvironmentError::KeyAlreadyExists));
			}

			let old_index = match env.values.get_index_of(key) {
				None => return Err(anyhow!(EnvironmentError::KeyNotFound)),
				Some(index) => index,
			};

			let (key, _) = env.values.get_index_mut2(old_index).unwrap();
			*key = new_key.to_string();

			info!("Environment key: \"{key}\" renamed to \"{new_key}\"");
		}

		self.save_environment_to_file(env_index);
		Ok(())
	}
}
