use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::OnceLock;
use tracing::{trace, warn};

use crate::app::app::App;
use crate::errors::panic_error;
use crate::models::collection::CollectionFileFormat;

pub static SKIP_SAVE_REQUESTS_RESPONSE: OnceLock<bool> = OnceLock::new();

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
	#[serde(default)]
	/// Theme preset name (e.g., "dracula", "catppuccin_mocha", "gruvbox")
	pub theme: Option<String>,

	#[serde(default)]
	/// Should disable syntax highlighting for responses
	pub disable_syntax_highlighting: Option<bool>,

	#[serde(default)]
	/// Should save requests response
	pub save_requests_response: Option<bool>,

	#[serde(default)]
	/// Should not display images
	pub disable_images_preview: Option<bool>,

	#[serde(default)]
	/// Should avoid using the terminal emulator graphical protocol when displaying an image. Using halfblocks instead.
	pub disable_graphical_protocol: Option<bool>,

	#[serde(default)]
	/// Should wrap response without overflowing in the response area
	pub wrap_responses: Option<bool>,

	#[serde(default)]
	/// Should use either JSON or YAML as preferred collection file format
	pub preferred_collection_file_format: Option<CollectionFileFormat>,

	#[serde(default)]
	/// Proxy usage
	pub proxy: Option<Proxy>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Proxy {
	pub http_proxy: Option<String>,
	pub https_proxy: Option<String>,
}

impl Config {
	pub fn get_theme(&self) -> Option<&str> {
		self.theme.as_deref()
	}

	pub fn is_syntax_highlighting_disabled(&self) -> bool {
		self.disable_syntax_highlighting.unwrap_or(false)
	}

	pub fn should_save_requests_response(&self) -> bool {
		self.save_requests_response.unwrap_or(false)
	}
	pub fn set_should_skip_requests_response(&self) {
		SKIP_SAVE_REQUESTS_RESPONSE.get_or_init(|| match self.save_requests_response {
			None => true,
			Some(save_requests_response) => !save_requests_response,
		});
	}

	pub fn is_image_preview_disabled(&self) -> bool {
		self.disable_images_preview.unwrap_or(false)
	}

	pub fn is_graphical_protocol_disabled(&self) -> bool {
		self.disable_graphical_protocol.unwrap_or(false)
	}

	pub fn should_wrap_body(&self) -> bool {
		self.wrap_responses.unwrap_or(false)
	}

	pub fn get_preferred_collection_file_format(&self) -> CollectionFileFormat {
		match &self.preferred_collection_file_format {
			None => CollectionFileFormat::default(),
			Some(file_format) => *file_format,
		}
	}

	pub fn get_proxy(&self) -> &Option<Proxy> {
		&self.proxy
	}
}

impl App<'_> {
	pub fn parse_config_file(&mut self, path_buf: &Path) {
		let mut file_content = String::new();

		trace!("Trying to open \"squrl.toml\" config file");

		let mut config_file = OpenOptions::new()
			.read(true)
			.write(true)
			.open(path_buf)
			.expect("\tCould not open config file");

		config_file
			.read_to_string(&mut file_content)
			.expect("\tCould not read config file");

		let config: Config = match toml::from_str(&file_content) {
			Ok(config) => config,
			Err(e) => panic_error(format!("Could not parse config file\n\t{e}")),
		};

		config.set_should_skip_requests_response();

		self.config = config;

		trace!("Config file parsed!");
	}

	pub fn parse_global_config_file(&mut self, path_buf: &Path) {
		let mut file_content = String::new();

		trace!(
			"Trying to open \"{}\" global config file",
			path_buf.display()
		);

		let mut global_config_file = OpenOptions::new()
			.read(true)
			.open(path_buf)
			.expect("\tCould not open global config file");

		global_config_file
			.read_to_string(&mut file_content)
			.expect("\tCould not read global config file");

		let global_config: Config = match toml::from_str(&file_content) {
			Ok(config) => config,
			Err(e) => panic_error(format!("Could not parse config file\n\t{e}")),
		};

		// Replace an attribute if it is not set

		if self.config.theme.is_none() {
			self.config.theme = global_config.theme;
		}

		if self.config.disable_syntax_highlighting.is_none() {
			self.config.disable_syntax_highlighting = global_config.disable_syntax_highlighting;
		}

		if self.config.save_requests_response.is_none() {
			self.config.save_requests_response = global_config.save_requests_response;
		}

		if self.config.disable_images_preview.is_none() {
			self.config.disable_images_preview = global_config.disable_images_preview;
		}

		if self.config.disable_graphical_protocol.is_none() {
			self.config.disable_graphical_protocol = global_config.disable_graphical_protocol;
		}

		if self.config.wrap_responses.is_none() {
			self.config.wrap_responses = global_config.wrap_responses;
		}

		if self.config.preferred_collection_file_format.is_none() {
			self.config.preferred_collection_file_format =
				global_config.preferred_collection_file_format;
		}

		if self.config.proxy.is_none() {
			self.config.proxy = global_config.proxy;
		}

		self.config.set_should_skip_requests_response();

		trace!("Global config file parsed!");
	}

	/// Save theme selection to global config file
	pub fn save_theme_to_global_config(&mut self, theme_name: &str) {
		use crate::cli::args::ARGS;

		let global_config_path = match &ARGS.user_config_directory {
			Some(dir) => dir.join("global.toml"),
			None => {
				warn!("Could not determine user config directory for saving theme");
				return;
			}
		};

		// Read existing config or create new one
		let mut config: Config = if global_config_path.exists() {
			match fs::read_to_string(&global_config_path) {
				Ok(content) => toml::from_str(&content).unwrap_or_default(),
				Err(_) => Config::default(),
			}
		} else {
			Config::default()
		};

		// Update theme
		config.theme = Some(theme_name.to_string());

		// Also update our local config
		self.config.theme = Some(theme_name.to_string());

		// Serialize and write
		match toml::to_string_pretty(&config) {
			Ok(content) => {
				// Ensure directory exists
				if let Some(parent) = global_config_path.parent() {
					let _ = fs::create_dir_all(parent);
				}

				match OpenOptions::new()
					.write(true)
					.create(true)
					.truncate(true)
					.open(&global_config_path)
				{
					Ok(mut file) => {
						if let Err(e) = file.write_all(content.as_bytes()) {
							warn!("Could not write global config file: {}", e);
						} else {
							trace!("Saved theme '{}' to global config", theme_name);
						}
					}
					Err(e) => {
						warn!("Could not open global config file for writing: {}", e);
					}
				}
			}
			Err(e) => {
				warn!("Could not serialize config: {}", e);
			}
		}
	}
}
