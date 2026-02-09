use std::fs::OpenOptions;
use std::io::Write;
use std::{env, fs};

use indexmap::IndexMap;
use lazy_static::lazy_static;
use rayon::prelude::*;
use tracing::{info, trace, warn};

use crate::{app::app::App, cli::args::ARGS, models::environment::Environment};

lazy_static! {
	pub static ref OS_ENV_VARS: IndexMap<String, String> = env::vars().collect();
}

impl App<'_> {
	pub fn save_environment_to_file(&mut self, env_index: usize) {
		let environment = self.environments[env_index].read();
		save_environment_to_file(&environment);
	}
}

pub fn save_environment_to_file(environment: &Environment) {
	if !ARGS.should_save {
		warn!("Dry-run, not saving the environment");
		return;
	}

	info!("Saving environment \"{}\"", environment.name);

	let temp_file_name = format!(
		"{}_",
		environment.path.file_name().unwrap().to_str().unwrap()
	);

	let temp_file_path = environment.path.with_file_name(temp_file_name);

	let mut temp_file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(&temp_file_path)
		.expect("Could not open temp file");

	let mut data: String = environment
		.values
		.iter()
		.par_bridge()
		.map(|(key, value)| format!("{key}={value}\n"))
		.collect();

	// remove trailing \n
	data.pop();

	temp_file
		.write_all(data.as_bytes())
		.expect("Could not write to temp file");
	temp_file.flush().unwrap();

	fs::rename(temp_file_path, &environment.path)
		.expect("Could not move temp file to environment file");

	trace!("Environment saved!")
}
