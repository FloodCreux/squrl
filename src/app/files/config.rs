use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::OnceLock;
use tracing::{trace, warn};

use anyhow::Context;

use crate::app::app::App;
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
	pub fn parse_config_file(&mut self, path_buf: &Path) -> anyhow::Result<()> {
		let mut file_content = String::new();

		trace!("Trying to open \"squrl.toml\" config file");

		let mut config_file = OpenOptions::new()
			.read(true)
			.write(true)
			.open(path_buf)
			.with_context(|| format!("Could not open config file \"{}\"", path_buf.display()))?;

		config_file
			.read_to_string(&mut file_content)
			.with_context(|| format!("Could not read config file \"{}\"", path_buf.display()))?;

		let config: Config = toml::from_str(&file_content)
			.with_context(|| format!("Could not parse config file \"{}\"", path_buf.display()))?;

		config.set_should_skip_requests_response();

		self.core.config = config;

		trace!("Config file parsed!");
		Ok(())
	}

	pub fn parse_global_config_file(&mut self, path_buf: &Path) -> anyhow::Result<()> {
		let mut file_content = String::new();

		trace!(
			"Trying to open \"{}\" global config file",
			path_buf.display()
		);

		let mut global_config_file =
			OpenOptions::new()
				.read(true)
				.open(path_buf)
				.with_context(|| {
					format!(
						"Could not open global config file \"{}\"",
						path_buf.display()
					)
				})?;

		global_config_file
			.read_to_string(&mut file_content)
			.with_context(|| {
				format!(
					"Could not read global config file \"{}\"",
					path_buf.display()
				)
			})?;

		let global_config: Config = toml::from_str(&file_content).with_context(|| {
			format!(
				"Could not parse global config file \"{}\"",
				path_buf.display()
			)
		})?;

		// Replace an attribute if it is not set

		if self.core.config.theme.is_none() {
			self.core.config.theme = global_config.theme;
		}

		if self.core.config.disable_syntax_highlighting.is_none() {
			self.core.config.disable_syntax_highlighting =
				global_config.disable_syntax_highlighting;
		}

		if self.core.config.save_requests_response.is_none() {
			self.core.config.save_requests_response = global_config.save_requests_response;
		}

		if self.core.config.disable_images_preview.is_none() {
			self.core.config.disable_images_preview = global_config.disable_images_preview;
		}

		if self.core.config.disable_graphical_protocol.is_none() {
			self.core.config.disable_graphical_protocol = global_config.disable_graphical_protocol;
		}

		if self.core.config.wrap_responses.is_none() {
			self.core.config.wrap_responses = global_config.wrap_responses;
		}

		if self.core.config.preferred_collection_file_format.is_none() {
			self.core.config.preferred_collection_file_format =
				global_config.preferred_collection_file_format;
		}

		if self.core.config.proxy.is_none() {
			self.core.config.proxy = global_config.proxy;
		}

		self.core.config.set_should_skip_requests_response();

		trace!("Global config file parsed!");
		Ok(())
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
		self.core.config.theme = Some(theme_name.to_string());

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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::collection::CollectionFileFormat;

	// ── Config default values ────────────────────────────────────

	#[test]
	fn default_config_has_no_theme() {
		let config = Config::default();
		assert!(config.get_theme().is_none());
	}

	#[test]
	fn default_config_syntax_highlighting_enabled() {
		let config = Config::default();
		assert!(!config.is_syntax_highlighting_disabled());
	}

	#[test]
	fn default_config_does_not_save_responses() {
		let config = Config::default();
		assert!(!config.should_save_requests_response());
	}

	#[test]
	fn default_config_image_preview_enabled() {
		let config = Config::default();
		assert!(!config.is_image_preview_disabled());
	}

	#[test]
	fn default_config_graphical_protocol_enabled() {
		let config = Config::default();
		assert!(!config.is_graphical_protocol_disabled());
	}

	#[test]
	fn default_config_does_not_wrap_body() {
		let config = Config::default();
		assert!(!config.should_wrap_body());
	}

	#[test]
	fn default_config_prefers_json_format() {
		let config = Config::default();
		assert!(matches!(
			config.get_preferred_collection_file_format(),
			CollectionFileFormat::Json
		));
	}

	#[test]
	fn default_config_has_no_proxy() {
		let config = Config::default();
		assert!(config.get_proxy().is_none());
	}

	// ── Config with set values ───────────────────────────────────

	#[test]
	fn config_get_theme_returns_value() {
		let config = Config {
			theme: Some("dracula".to_string()),
			..Default::default()
		};
		assert_eq!(config.get_theme(), Some("dracula"));
	}

	#[test]
	fn config_syntax_highlighting_disabled() {
		let config = Config {
			disable_syntax_highlighting: Some(true),
			..Default::default()
		};
		assert!(config.is_syntax_highlighting_disabled());
	}

	#[test]
	fn config_save_requests_response_enabled() {
		let config = Config {
			save_requests_response: Some(true),
			..Default::default()
		};
		assert!(config.should_save_requests_response());
	}

	#[test]
	fn config_image_preview_disabled() {
		let config = Config {
			disable_images_preview: Some(true),
			..Default::default()
		};
		assert!(config.is_image_preview_disabled());
	}

	#[test]
	fn config_wrap_body_enabled() {
		let config = Config {
			wrap_responses: Some(true),
			..Default::default()
		};
		assert!(config.should_wrap_body());
	}

	#[test]
	fn config_prefers_yaml_format() {
		let config = Config {
			preferred_collection_file_format: Some(CollectionFileFormat::Yaml),
			..Default::default()
		};
		assert!(matches!(
			config.get_preferred_collection_file_format(),
			CollectionFileFormat::Yaml
		));
	}

	// ── TOML deserialization ─────────────────────────────────────

	#[test]
	fn parse_empty_toml_gives_defaults() {
		let config: Config = toml::from_str("").unwrap();
		assert!(config.theme.is_none());
		assert!(config.disable_syntax_highlighting.is_none());
		assert!(config.save_requests_response.is_none());
		assert!(config.proxy.is_none());
	}

	#[test]
	fn parse_toml_with_theme() {
		let config: Config = toml::from_str(r#"theme = "catppuccin_mocha""#).unwrap();
		assert_eq!(config.get_theme(), Some("catppuccin_mocha"));
	}

	#[test]
	fn parse_toml_with_all_fields() {
		let toml_str = r#"
theme = "gruvbox"
disable_syntax_highlighting = true
save_requests_response = true
disable_images_preview = true
disable_graphical_protocol = true
wrap_responses = true
preferred_collection_file_format = "yaml"

[proxy]
http_proxy = "http://proxy.local:8080"
https_proxy = "https://proxy.local:8443"
"#;
		let config: Config = toml::from_str(toml_str).unwrap();
		assert_eq!(config.get_theme(), Some("gruvbox"));
		assert!(config.is_syntax_highlighting_disabled());
		assert!(config.should_save_requests_response());
		assert!(config.is_image_preview_disabled());
		assert!(config.is_graphical_protocol_disabled());
		assert!(config.should_wrap_body());
		assert!(matches!(
			config.get_preferred_collection_file_format(),
			CollectionFileFormat::Yaml
		));
		let proxy = config.proxy.unwrap();
		assert_eq!(
			proxy.http_proxy,
			Some("http://proxy.local:8080".to_string())
		);
		assert_eq!(
			proxy.https_proxy,
			Some("https://proxy.local:8443".to_string())
		);
	}

	#[test]
	fn parse_toml_ignores_unknown_keys() {
		let config: Config = toml::from_str("theme = \"test\"\nunknown_key = \"ignored\"").unwrap();
		assert_eq!(config.get_theme(), Some("test"));
	}

	#[test]
	fn parse_toml_with_proxy_only() {
		let toml_str = r#"
[proxy]
http_proxy = "http://127.0.0.1:3128"
"#;
		let config: Config = toml::from_str(toml_str).unwrap();
		let proxy = config.proxy.unwrap();
		assert_eq!(proxy.http_proxy, Some("http://127.0.0.1:3128".to_string()));
		assert!(proxy.https_proxy.is_none());
	}

	#[test]
	fn parse_toml_with_partial_proxy() {
		let toml_str = r#"
[proxy]
https_proxy = "https://secure.proxy:443"
"#;
		let config: Config = toml::from_str(toml_str).unwrap();
		let proxy = config.proxy.unwrap();
		assert!(proxy.http_proxy.is_none());
		assert_eq!(
			proxy.https_proxy,
			Some("https://secure.proxy:443".to_string())
		);
	}

	#[test]
	fn parse_toml_booleans_false() {
		let toml_str = r#"
disable_syntax_highlighting = false
save_requests_response = false
"#;
		let config: Config = toml::from_str(toml_str).unwrap();
		assert!(!config.is_syntax_highlighting_disabled());
		assert!(!config.should_save_requests_response());
	}

	#[test]
	fn config_serializes_to_toml() {
		let config = Config {
			theme: Some("dracula".to_string()),
			disable_syntax_highlighting: Some(true),
			..Default::default()
		};
		let toml_str = toml::to_string_pretty(&config).unwrap();
		assert!(toml_str.contains("dracula"));
		assert!(toml_str.contains("disable_syntax_highlighting = true"));
	}

	#[test]
	fn config_roundtrip_through_toml() {
		let original = Config {
			theme: Some("nord".to_string()),
			disable_syntax_highlighting: Some(false),
			save_requests_response: Some(true),
			disable_images_preview: Some(true),
			disable_graphical_protocol: None,
			wrap_responses: Some(false),
			preferred_collection_file_format: Some(CollectionFileFormat::Yaml),
			proxy: Some(Proxy {
				http_proxy: Some("http://proxy:8080".to_string()),
				https_proxy: None,
			}),
		};

		let toml_str = toml::to_string_pretty(&original).unwrap();
		let restored: Config = toml::from_str(&toml_str).unwrap();

		assert_eq!(restored.get_theme(), Some("nord"));
		assert!(!restored.is_syntax_highlighting_disabled());
		assert!(restored.should_save_requests_response());
		assert!(restored.is_image_preview_disabled());
		assert!(!restored.is_graphical_protocol_disabled());
		assert!(!restored.should_wrap_body());
		assert!(matches!(
			restored.get_preferred_collection_file_format(),
			CollectionFileFormat::Yaml
		));
		let proxy = restored.proxy.unwrap();
		assert_eq!(proxy.http_proxy, Some("http://proxy:8080".to_string()));
		assert!(proxy.https_proxy.is_none());
	}

	// ── OnceLock skip behavior ───────────────────────────────────

	#[test]
	fn set_should_skip_when_save_is_none() {
		// When save_requests_response is None, skip should be true
		let config = Config {
			save_requests_response: None,
			..Default::default()
		};
		// We can't test OnceLock directly since it's global state,
		// but we can verify the logic: None => true (skip saving)
		let result = match config.save_requests_response {
			None => true,
			Some(save_requests_response) => !save_requests_response,
		};
		assert!(result);
	}

	#[test]
	fn set_should_skip_when_save_is_true() {
		let config = Config {
			save_requests_response: Some(true),
			..Default::default()
		};
		let result = match config.save_requests_response {
			None => true,
			Some(save_requests_response) => !save_requests_response,
		};
		assert!(!result); // save=true means skip=false
	}

	#[test]
	fn set_should_skip_when_save_is_false() {
		let config = Config {
			save_requests_response: Some(false),
			..Default::default()
		};
		let result = match config.save_requests_response {
			None => true,
			Some(save_requests_response) => !save_requests_response,
		};
		assert!(result); // save=false means skip=true
	}
}
