use crate::app::app::App;
use crate::app::log::LogCounterLayer;
use crate::app::startup::startup::AppMode::{CLI, TUI};
use crate::cli::args::{ARGS, Command};
use crate::cli::import::http_file;
use crate::errors::panic_error;
use crate::models::collection::{Collection, CollectionFileFormat};
use crate::models::folder::Folder;
use clap_verbosity_flag::log::LevelFilter;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use tracing::{trace, warn};
use tracing_log::AsTrace;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use walkdir::WalkDir;

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
		let paths = match ARGS
			.directory
			.as_ref()
			.expect("--directory argument is required")
			.read_dir()
		{
			Ok(paths) => paths,
			Err(e) => panic_error(format!(
				"Directory \"{}\" not found\n\t{e}",
				ARGS.directory
					.as_ref()
					.expect("--directory argument is required")
					.display()
			)),
		};

		for path in paths {
			let path = path.expect("directory entry should be readable").path();

			if path.is_dir() {
				continue;
			}

			let file_name = path
				.file_name()
				.expect("path should have a file name")
				.to_str()
				.expect("file name should be valid UTF-8");

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
			for index in 0..self.core.collections.len() {
				self.save_collection_to_file(index);
			}
		}

		self.core.collections.sort_by(|a, b| {
			a.last_position
				.unwrap_or(usize::MAX)
				.cmp(&b.last_position.unwrap_or(usize::MAX))
		});

		// Auto-load .http files from a "requests" subdirectory if we're in a git repo
		// Recursively searches subdirectories and groups files into folders
		// named after the top-level child directory of requests/
		// Added after the save loop and sort to avoid saving ephemeral collections to disk
		if let Ok(cwd) = std::env::current_dir() {
			let requests_dir = cwd.join("requests");
			if cwd.join(".git").exists() && requests_dir.is_dir() {
				let collection_name = cwd
					.file_name()
					.map(|n| n.to_str().unwrap_or("http-requests"))
					.unwrap_or("http-requests")
					.to_string();

				// Recursively collect all .http file paths and sort for alphabetical ordering
				let mut http_file_paths: Vec<PathBuf> = WalkDir::new(&requests_dir)
					.into_iter()
					.filter_map(|e| e.ok())
					.filter(|e| {
						e.file_type().is_file()
							&& e.path().extension().is_some_and(|ext| ext == "http")
					})
					.map(|e| e.path().to_path_buf())
					.collect();

				http_file_paths.sort();

				// Group files: root-level files go to collection.requests,
				// files in subdirectories go to folders named after the top-level child
				let mut root_requests = vec![];
				let mut folder_map: BTreeMap<String, Vec<_>> = BTreeMap::new();

				for http_path in &http_file_paths {
					let relative = match http_path.strip_prefix(&requests_dir) {
						Ok(p) => p,
						Err(_) => continue,
					};

					let components: Vec<_> = relative.components().collect();

					let parsed_requests = match http_file::parse_http_file(http_path) {
						Ok(reqs) => reqs,
						Err(e) => {
							warn!(
								"Could not parse .http file \"{}\": {}",
								http_path.display(),
								e
							);
							continue;
						}
					};

					if components.len() == 1 {
						// File directly in requests/ → root-level requests
						root_requests.extend(parsed_requests);
					} else {
						// File in a subdirectory → folder named after the first path component
						let folder_name = components[0]
							.as_os_str()
							.to_str()
							.unwrap_or("unknown")
							.to_string();

						folder_map
							.entry(folder_name)
							.or_default()
							.extend(parsed_requests);
					}
				}

				// Build folders from BTreeMap (alphabetically ordered by key)
				let folders: Vec<Folder> = folder_map
					.into_iter()
					.map(|(name, requests)| Folder { name, requests })
					.collect();

				if !root_requests.is_empty() || !folders.is_empty() {
					trace!(
						"Found {} root request(s) and {} folder(s) in requests/ directory, \
						 creating ephemeral collection \"{}\"",
						root_requests.len(),
						folders.len(),
						collection_name
					);

					let collection = Collection {
						name: collection_name,
						last_position: None,
						folders,
						requests: root_requests,
						path: PathBuf::new(), // Ephemeral — no file path
						file_format: CollectionFileFormat::default(),
					};

					self.core.collections.push(collection);
				}
			}
		}
	}

	fn create_log_file(&mut self) -> File {
		let path = ARGS
			.directory
			.as_ref()
			.expect("--directory argument is required")
			.join("squrl.log");

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
