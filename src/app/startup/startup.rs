use crate::app::app::App;
use crate::app::log::LogCounterLayer;
use crate::app::startup::startup::AppMode::{CLI, TUI};
use crate::cli::args::{ARGS, Command};
use crate::cli::import::http_file;
use crate::errors::panic_error;
use crate::models::collection::{Collection, CollectionFileFormat};
use clap_verbosity_flag::log::LevelFilter;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use tracing::trace;
use tracing_log::AsTrace;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[allow(clippy::large_enum_variant)]
pub enum AppMode {
	TUI,
	CLI(Command),
}

impl<'a> App<'a> {
	/// Method called before running the app, returns the app if the TUI should be started
	pub fn startup(&mut self) -> AppMode {
		// Logging is initialized before anything else
		match ARGS.command.is_some() {
			// CLI
			true => tracing_subscriber::fmt()
				.pretty()
				.with_max_level(ARGS.verbosity.log_level_filter().as_trace())
				.with_file(false)
				.with_line_number(false)
				.with_ansi(ARGS.ansi_log)
				.init(),
			// TUI
			false => {
				let verbosity = match ARGS.verbosity.log_level_filter() {
					LevelFilter::Error => LevelFilter::Debug, // Ensure that at least the debug level is always active
					level => level,
				};

				// Using a separate file allows to redirect the output and avoid printing to screen
				let log_file = self.create_log_file();

				tracing_subscriber::fmt()
					.with_max_level(verbosity.as_trace())
					.with_writer(log_file)
					.with_file(false)
					.with_line_number(false)
					.with_ansi(ARGS.ansi_log)
					.finish()
					.with(LogCounterLayer)
					.init();
			}
		};

		if ARGS.should_parse_directory {
			self.parse_app_directory();
		}

		if let Some(command) = &ARGS.command {
			CLI(command.clone())
		} else {
			self.parse_key_bindings_file();
			self.load_theme();
			self.update_text_inputs_handler();

			TUI
		}
	}

	fn parse_app_directory(&mut self) {
		let paths = match ARGS.directory.as_ref().unwrap().read_dir() {
			Ok(paths) => paths,
			Err(e) => panic_error(format!(
				"Directory \"{}\" not found\n\t{e}",
				ARGS.directory.as_ref().unwrap().display()
			)),
		};

		for path in paths {
			let path = path.unwrap().path();

			if path.is_dir() {
				continue;
			}

			let file_name = path.file_name().unwrap().to_str().unwrap();

			trace!("Checking file \"{}\"", path.display());

			if file_name.starts_with(".env.") {
				self.add_environment_from_file(&path);
				continue;
			} else if file_name == "squrl.toml" {
				self.parse_config_file(&path);
				continue;
			} else if file_name == "squrl.log" {
				trace!("Log file is not parsable");
				continue;
			}

			if let Some(filter) = &ARGS.collection_filter
				&& !filter.is_match(file_name)
			{
				trace!("File \"{file_name}\" does not match filter");
				continue;
			}

			if file_name.ends_with(".json") {
				self.set_collections_from_file(path, CollectionFileFormat::Json);
			} else if file_name.ends_with(".yaml") {
				self.set_collections_from_file(path, CollectionFileFormat::Yaml);
			}
		}

		// Check if the global config file exists
		if let Some(config_directory) = &ARGS.config_directory {
			let global_config_file_path = config_directory.join("global.toml");

			if global_config_file_path.exists() {
				self.parse_global_config_file(&global_config_file_path);
			}
		}

		if let Some(user_config_dir) = &ARGS.user_config_directory
			&& ARGS.config_directory.as_ref() != Some(user_config_dir)
		{
			let user_global_config_path = user_config_dir.join("global.toml");
			if user_global_config_path.exists() {
				self.parse_global_config_file(&user_global_config_path);
			}
		}

		// Ensures that legacy collections and requests gets save as their new version
		if ARGS.should_save {
			for index in 0..self.collections.len() {
				self.save_collection_to_file(index);
			}
		}

		self.collections.sort_by(|a, b| {
			a.last_position
				.unwrap_or(usize::MAX)
				.cmp(&b.last_position.unwrap_or(usize::MAX))
		});

		// Auto-load .http files from a "requests" subdirectory if we're in a git repo
		// Added after the save loop and sort to avoid saving ephemeral collections to disk
		if let Ok(cwd) = std::env::current_dir() {
			let requests_dir = cwd.join("requests");
			if cwd.join(".git").exists() && requests_dir.is_dir() {
				let collection_name = cwd
					.file_name()
					.map(|n| n.to_str().unwrap_or("http-requests"))
					.unwrap_or("http-requests")
					.to_string();

				let http_file_paths: Vec<PathBuf> = match requests_dir.read_dir() {
					Ok(entries) => entries
						.filter_map(|e| e.ok())
						.map(|e| e.path())
						.filter(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "http"))
						.collect(),
					Err(_) => vec![],
				};

				if !http_file_paths.is_empty() {
					trace!(
						"Found {} .http file(s) in requests/ directory, creating ephemeral collection \"{}\"",
						http_file_paths.len(),
						collection_name
					);

					let mut requests = vec![];

					for http_path in &http_file_paths {
						match http_file::parse_http_file(http_path) {
							Ok(parsed_requests) => requests.extend(parsed_requests),
							Err(e) => {
								trace!(
									"Could not parse .http file \"{}\": {}",
									http_path.display(),
									e
								);
							}
						}
					}

					if !requests.is_empty() {
						let collection = Collection {
							name: collection_name,
							last_position: None,
							folders: vec![],
							requests,
							path: PathBuf::new(), // Ephemeral — no file path
							file_format: CollectionFileFormat::default(),
						};

						self.collections.push(collection);
					}
				}
			}
		}
	}

	fn create_log_file(&mut self) -> File {
		let path = ARGS.directory.as_ref().unwrap().join("squrl.log");

		match OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(path)
		{
			Ok(log_file) => log_file,
			Err(e) => panic_error(format!("Could not open log file\n\t{e}")),
		}
	}
}
