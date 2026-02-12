use nestify::nest;
use parking_lot::RwLock;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::{trace, warn};

use crate::app::app::App;
use crate::app::files::utils::expand_tilde;
use crate::cli::args::ARGS;

nest! {
	#[derive(Clone, Serialize, Deserialize)]
	pub struct Theme {
		#[serde(alias = "UI")]
		pub ui:
			#[derive(Clone, Serialize, Deserialize)]
			pub struct ThemeUI {
				pub font_color: Color,
				pub app_background: Option<Color>,

				pub main_foreground_color: Color,
				pub secondary_foreground_color: Color,

				pub main_background_color: Color,
				pub secondary_background_color: Color,

				pub separator_color: Color,
			},

		#[serde(alias = "Others")]
		pub others:
			#[derive(Clone, Serialize, Deserialize)]
			pub struct ThemeOthers {
				pub selection_highlight_color: Color,
				pub environment_variable_highlight_color: Color,
			},

		#[serde(alias = "HTTP")]
		pub http: #[derive(Clone, Serialize, Deserialize)]
			pub struct ThemeHttp {
			#[serde(alias = "Methods")]
			pub methods:
				#[derive(Clone, Serialize, Deserialize)]
				pub struct ThemeMethods {
					#[serde(alias = "GET")]
					pub get: Color,
					#[serde(alias = "POST")]
					pub post: Color,
					#[serde(alias = "PUT")]
					pub put: Color,
					#[serde(alias = "PATCH")]
					pub patch: Color,
					#[serde(alias = "DELETE")]
					pub delete: Color,
					#[serde(alias = "HEAD")]
					pub head: Color,
					#[serde(alias = "OPTIONS")]
					pub options: Color,
					#[serde(alias = "TRACE")]
					pub trace: Color,
					#[serde(alias = "CONNECT")]
					pub connect: Color
				},
		},

		#[serde(alias = "Websocket")]
		pub websocket:
			#[derive(Clone, Serialize, Deserialize)]
			pub struct ThemeWebsocket {
				#[serde(alias = "Connection Statuses")]
				pub connection_status:
					#[derive(Clone, Serialize, Deserialize)]
					pub struct ThemeConnectionStatuses {
						#[serde(alias = "Connected")]
						pub connected: Color,
						#[serde(alias = "Disconnected")]
						pub disconnected: Color,
					},

				#[serde(alias = "Messages")]
				pub messages:
					#[derive(Clone, Serialize, Deserialize)]
					pub struct ThemeMessages {
						#[serde(alias = "server_foreground_color")]
						pub server_foreground_color: Color,
						#[serde(alias = "server_background_color")]
						pub server_background_color: Color,
						#[serde(alias = "you_background_color")]
						pub you_background_color: Color,
						#[serde(alias = "details_color")]
						pub details_color: Color,
					}
			}
	}
}

impl Default for Theme {
	fn default() -> Self {
		// Gruber Darker theme colors
		// Based on https://github.com/rexim/gruber-darker-theme
		let gruber_fg = Color::Rgb(228, 228, 239); // #e4e4ef
		let gruber_niagara = Color::Rgb(150, 166, 200); // #96a6c8
		let gruber_quartz = Color::Rgb(149, 169, 159); // #95a99f
		let gruber_bg1 = Color::Rgb(40, 40, 40); // #282828
		let gruber_bg = Color::Rgb(24, 24, 24); // #181818
		let gruber_bg2 = Color::Rgb(69, 61, 65); // #453d41
		let gruber_yellow = Color::Rgb(255, 221, 51); // #ffdd33
		let gruber_wisteria = Color::Rgb(158, 149, 199); // #9e95c7
		let gruber_green = Color::Rgb(115, 201, 54); // #73c936
		let gruber_red = Color::Rgb(244, 56, 65); // #f43841
		let gruber_red1 = Color::Rgb(255, 79, 88); // #ff4f58
		let gruber_brown = Color::Rgb(204, 140, 60); // #cc8c3c

		Theme {
			ui: ThemeUI {
				font_color: gruber_fg,
				app_background: None,

				main_foreground_color: gruber_niagara,
				secondary_foreground_color: gruber_quartz,

				main_background_color: gruber_bg1,
				secondary_background_color: gruber_bg,

				separator_color: gruber_bg2,
			},
			others: ThemeOthers {
				selection_highlight_color: gruber_yellow,
				environment_variable_highlight_color: gruber_wisteria,
			},
			http: ThemeHttp {
				methods: ThemeMethods {
					get: gruber_green,
					post: gruber_yellow,
					put: gruber_niagara,
					patch: gruber_quartz,
					delete: gruber_red1,
					options: gruber_wisteria,
					head: gruber_green,
					trace: gruber_brown,
					connect: gruber_niagara,
				},
			},
			websocket: ThemeWebsocket {
				connection_status: ThemeConnectionStatuses {
					connected: gruber_green,
					disconnected: gruber_red,
				},

				messages: ThemeMessages {
					server_foreground_color: gruber_niagara,
					server_background_color: gruber_bg1,
					you_background_color: gruber_niagara,
					details_color: gruber_quartz,
				},
			},
		}
	}
}

pub static THEME: LazyLock<RwLock<Theme>> = LazyLock::new(|| RwLock::new(Theme::default()));

/// Set the global theme
pub fn set_theme(theme: Theme) {
	*THEME.write() = theme;
}

/// Get a clone of the current theme
pub fn get_theme() -> Theme {
	THEME.read().clone()
}

/// Load and apply a theme by name from presets or user themes directory
pub fn load_theme_by_name(
	name: &str,
	user_themes_dir: Option<&std::path::Path>,
) -> Result<(), String> {
	use crate::app::files::theme_presets::load_theme_by_name as load_preset;

	let theme = load_preset(name, user_themes_dir)?;
	set_theme(theme);
	trace!("Applied theme: {}", name);
	Ok(())
}

impl App<'_> {
	/// Load theme with the following priority order:
	/// 1. CLI --theme flag
	/// 2. SQURL_THEME environment variable (path to custom theme file)
	/// 3. ~/.config/squrl/theme.toml (custom theme file)
	/// 4. Config file theme field (preset name)
	/// 5. Default theme
	pub fn load_theme(&mut self) {
		let user_themes_dir = ARGS
			.user_config_directory
			.as_ref()
			.map(|d| d.join("themes"));

		// 1. Check CLI --theme flag first
		if let Some(theme_name) = &ARGS.theme {
			trace!("Loading theme from CLI flag: {}", theme_name);
			match load_theme_by_name(theme_name, user_themes_dir.as_deref()) {
				Ok(()) => return,
				Err(e) => {
					warn!("Could not load theme '{}' from CLI flag: {}", theme_name, e);
					// Fall through to other methods
				}
			}
		}

		// 2. Check SQURL_THEME environment variable
		if let Ok(env_theme) = env::var("SQURL_THEME") {
			let path = expand_tilde(PathBuf::from(env_theme));
			trace!("Loading theme from SQURL_THEME env: {}", path.display());
			if self.load_theme_from_file(&path) {
				return;
			}
		}

		// 3. Check ~/.config/squrl/theme.toml
		if let Some(theme_path) = ARGS
			.user_config_directory
			.as_ref()
			.map(|dir| dir.join("theme.toml"))
			.filter(|p| p.exists())
		{
			trace!("Loading theme from user config: {}", theme_path.display());
			if self.load_theme_from_file(&theme_path) {
				return;
			}
		}

		// 4. Check config file theme field
		if let Some(theme_name) = self.core.config.get_theme() {
			trace!("Loading theme from config file: {}", theme_name);
			match load_theme_by_name(theme_name, user_themes_dir.as_deref()) {
				Ok(()) => return,
				Err(e) => {
					warn!("Could not load theme '{}' from config: {}", theme_name, e);
				}
			}
		}

		// 5. Fall back to default
		trace!("Using default theme");
	}

	/// Load a theme from a file path, returns true if successful
	fn load_theme_from_file(&self, path: &PathBuf) -> bool {
		let mut theme_file = match OpenOptions::new().read(true).open(path) {
			Ok(theme_file) => theme_file,
			Err(e) => {
				warn!("Could not open theme file '{}': {}", path.display(), e);
				return false;
			}
		};

		let mut file_content = String::new();
		if let Err(e) = theme_file.read_to_string(&mut file_content) {
			warn!("Could not read theme file '{}': {}", path.display(), e);
			return false;
		}

		match toml::from_str::<Theme>(&file_content) {
			Ok(theme) => {
				set_theme(theme);
				trace!("Theme loaded from file: {}", path.display());
				true
			}
			Err(e) => {
				warn!("Could not parse theme file '{}': {}", path.display(), e);
				false
			}
		}
	}
}
