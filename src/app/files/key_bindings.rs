use std::env;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;

use crokey::{KeyCombination, key};
use lazy_static::lazy_static;
use nestify::nest;
use parking_lot::RwLock;
use ratatui::text::Span;
use serde::Deserialize;
use tracing::{trace, warn};

use crate::app::app::App;
use crate::app::files::utils::expand_tilde;
use crate::cli::args::ARGS;
use crate::errors::panic_error;

#[derive(Default, Copy, Clone, Deserialize)]
#[serde(default)]
pub struct KeyBindingsConfig {
	pub keybindings: KeyBindings,
}

nest! {
	#[derive(Copy, Clone, Deserialize)]
	#[serde(default)]
	pub struct KeyBindings {
		pub main_menu: #[derive(Copy, Clone, Deserialize)] #[serde(default)] pub struct MainMenu {
			/// ctrl-c is implemented by default
			pub exit: KeyCombination,

			pub expand_collection: KeyCombination,
			pub unselect_request: KeyCombination,

			pub move_request_up: KeyCombination,
			pub move_request_down: KeyCombination,

			pub next_environment: KeyCombination,

			pub display_env_editor: KeyCombination,
			pub display_cookies: KeyCombination,
			pub display_logs: KeyCombination,
		},

		pub generic: #[derive(Copy, Clone, Deserialize)] #[serde(default)] pub struct Generic {
			pub display_help: KeyCombination,

			pub text_input: #[derive(Copy, Clone, Deserialize)] #[serde(default)] pub struct TextInput {
				pub quit_without_saving: KeyCombination,
				pub save_and_quit_single_line: KeyCombination,
				pub save_and_quit_area: KeyCombination,
				pub mode: #[derive(Copy, Clone, PartialEq, Deserialize)] pub enum TextAreaMode {
					#[serde(alias = "vim", alias = "VIM")]
					Vim,
					#[serde(alias = "emacs", alias = "EMACS")]
					Emacs,
					#[serde(alias = "default", alias = "DEFAULT")]
					Default,
					#[serde(alias = "custom", alias = "CUSTOM")]
					Custom(CustomTextArea)
				},
			},

			/// Navigation in tables, popups, up and down in the collections list
			pub navigation: #[derive(Copy, Clone, Deserialize)] #[serde(default)] pub struct Navigation {
				pub move_cursor_up: KeyCombination,
				pub move_cursor_down: KeyCombination,
				pub move_cursor_left: KeyCombination,
				pub move_cursor_right: KeyCombination,

				pub alt_move_cursor_up: KeyCombination,
				pub alt_move_cursor_down: KeyCombination,
				#[allow(dead_code)]
				pub alt_move_cursor_left: KeyCombination,
				#[allow(dead_code)]
				pub alt_move_cursor_right: KeyCombination,

				pub go_back: KeyCombination,
				pub select: KeyCombination,
			},

			pub list_and_table_actions: #[derive(Copy, Clone, Deserialize)] #[serde(default)] pub struct ListAndTableActions {
				pub create_element: KeyCombination,
				pub delete_element: KeyCombination,
				pub edit_element: KeyCombination,
				/// Only used in the collections list (main menu)
				pub rename_element: KeyCombination,
				/// Only used in tables (Query params, headers, cookies)
				pub toggle_element: KeyCombination,
				pub duplicate_element: KeyCombination,
			}
		},

		pub request_selected: #[derive(Copy, Clone, Deserialize)] #[serde(default)] pub struct RequestSelected {
			pub param_next_tab: KeyCombination,
			pub change_url: KeyCombination,
			pub change_method: KeyCombination,
			pub request_settings: KeyCombination,
			pub export_request: KeyCombination,

			pub next_view: KeyCombination,

			pub send_request: KeyCombination,
			pub alt_send_request: KeyCombination,

			pub param_tabs: #[derive(Copy, Clone, Deserialize)] #[serde(default)] pub struct ParamTabs {
				pub change_auth_method: KeyCombination,
				pub change_body_content_type: KeyCombination,
				pub change_message_type: KeyCombination,
			},

			pub result_tabs: #[derive(Copy, Clone, Deserialize)] #[serde(default)] pub struct ResultTabs {
				pub scroll_up: KeyCombination,
				pub scroll_down: KeyCombination,
				pub scroll_left: KeyCombination,
				pub scroll_right: KeyCombination,

				pub yank_response_part: KeyCombination,

				/// Will use param_next_tab depending on the selected view
				pub result_next_tab: KeyCombination,

				/// Enter selection mode in response body
				pub select_response_body: KeyCombination,
			}
		},
	}
}

lazy_static! {
	pub static ref KEY_BINDINGS: RwLock<KeyBindings> = RwLock::new(KeyBindings::default());
}

#[derive(Copy, Clone, PartialEq, Deserialize)]
pub struct CustomTextArea {
	pub copy: KeyCombination,
	pub paste: KeyCombination,

	pub search: KeyCombination,
	pub system_editor: KeyCombination,

	pub undo: KeyCombination,
	pub redo: KeyCombination,

	pub new_line: KeyCombination,
	pub indent: KeyCombination,

	pub delete_backward: KeyCombination,
	pub delete_forward: KeyCombination,

	pub skip_word_right: KeyCombination,
	pub skip_word_left: KeyCombination,

	pub move_cursor_up: KeyCombination,
	pub move_cursor_down: KeyCombination,
	pub move_cursor_left: KeyCombination,
	pub move_cursor_right: KeyCombination,
	pub move_cursor_line_start: KeyCombination,
	pub move_cursor_line_end: KeyCombination,
}

impl Default for MainMenu {
	fn default() -> Self {
		MainMenu {
			exit: key!(q),

			expand_collection: key!(right),
			unselect_request: key!(left),

			move_request_up: key!(ctrl - up),
			move_request_down: key!(ctrl - down),

			next_environment: key!(e),

			display_env_editor: key!(ctrl - e),
			display_cookies: key!(c),
			display_logs: key!(l),
		}
	}
}

impl Default for Generic {
	fn default() -> Self {
		Generic {
			display_help: key!(Ctrl - h),
			text_input: TextInput::default(),
			navigation: Navigation::default(),
			list_and_table_actions: ListAndTableActions::default(),
		}
	}
}

impl Default for TextInput {
	fn default() -> Self {
		TextInput {
			quit_without_saving: key!(esc),
			save_and_quit_single_line: key!(enter),
			save_and_quit_area: key!(ctrl - s),
			mode: TextAreaMode::Default,
		}
	}
}

#[allow(clippy::derivable_impls)]
impl Default for TextAreaMode {
	fn default() -> Self {
		TextAreaMode::Default
	}
}

impl Default for Navigation {
	fn default() -> Self {
		Navigation {
			move_cursor_up: key!(up),
			move_cursor_down: key!(down),
			move_cursor_left: key!(left),
			move_cursor_right: key!(right),

			alt_move_cursor_up: key!(Up),
			alt_move_cursor_down: key!(Down),
			alt_move_cursor_left: key!(Left),
			alt_move_cursor_right: key!(Right),

			go_back: key!(esc),
			select: key!(enter),
		}
	}
}

impl Default for ListAndTableActions {
	fn default() -> Self {
		ListAndTableActions {
			create_element: key!(n),
			delete_element: key!(d),
			edit_element: key!(enter),
			rename_element: key!(r),
			toggle_element: key!(t),
			duplicate_element: key!(ctrl - d),
		}
	}
}

impl Default for RequestSelected {
	fn default() -> Self {
		RequestSelected {
			param_next_tab: key!(tab),

			change_url: key!(u),
			change_method: key!(m),

			request_settings: key!(s),
			export_request: key!(shift - E),

			next_view: key!(v),

			// Used to be ctrl + enter, but it doesn't register right on many platforms
			// https://github.com/crossterm-rs/crossterm/issues/685
			send_request: key!(space),
			alt_send_request: key!(ctrl - enter),

			param_tabs: ParamTabs::default(),
			result_tabs: ResultTabs::default(),
		}
	}
}

impl Default for ParamTabs {
	fn default() -> Self {
		ParamTabs {
			change_auth_method: key!(ctrl - a),
			change_body_content_type: key!(ctrl - b),
			change_message_type: key!(ctrl - m),
		}
	}
}

impl Default for ResultTabs {
	fn default() -> Self {
		ResultTabs {
			scroll_up: key!(ctrl - up),
			scroll_down: key!(ctrl - down),
			scroll_left: key!(ctrl - left),
			scroll_right: key!(ctrl - right),

			yank_response_part: key!(y),

			result_next_tab: key!(shift - backtab),

			select_response_body: key!(o),
		}
	}
}

#[allow(clippy::derivable_impls)]
impl Default for KeyBindings {
	fn default() -> Self {
		KeyBindings {
			main_menu: MainMenu::default(),
			generic: Generic::default(),
			request_selected: RequestSelected::default(),
		}
	}
}

impl Default for CustomTextArea {
	fn default() -> Self {
		CustomTextArea {
			copy: key!(ctrl - c),
			paste: key!(ctrl - v),

			search: key!(ctrl - f),
			system_editor: key!(ctrl - e),

			undo: key!(ctrl - z),
			redo: key!(ctrl - y),

			new_line: key!(enter),
			indent: key!(tab),

			delete_backward: key!(delete),
			delete_forward: key!(backspace),

			skip_word_right: key!(ctrl - right),
			skip_word_left: key!(ctrl - left),

			move_cursor_up: key!(up),
			move_cursor_down: key!(down),
			move_cursor_left: key!(left),
			move_cursor_right: key!(right),
			move_cursor_line_start: key!(home),
			move_cursor_line_end: key!(end),
		}
	}
}

impl App<'_> {
	pub fn parse_key_bindings_file(&mut self) {
		let path = match env::var("SQURL_KEY_BINDINGS") {
			// If the SQURL_KEY_BINDINGS environment variable exists
			Ok(env_key_bindings) => expand_tilde(PathBuf::from(env_key_bindings)),
			Err(_) => {
				let default_path = ARGS
					.user_config_directory
					.as_ref()
					.map(|dir| dir.join("keybindings.toml"));

				match default_path {
					Some(p) if p.exists() => p,
					_ => {
						warn!("No key bindings file found, using default");
						return;
					}
				}
			}
		};

		trace!("Parsing key bindings file \"{}\"", path.display());

		let mut key_bindings_file = match OpenOptions::new().read(true).open(path) {
			Ok(key_bindings_file) => key_bindings_file,
			Err(e) => panic_error(format!("Could not open key bindings file\n\t{e}")),
		};

		let mut file_content = String::new();
		key_bindings_file
			.read_to_string(&mut file_content)
			.expect("\tCould not read key bindings file");

		let config: KeyBindingsConfig = match toml::from_str(&file_content) {
			Ok(config) => config,
			Err(e) => panic_error(format!("Could not parse key bindings file\n\t{e}")),
		};

		*KEY_BINDINGS.write() = config.keybindings;

		trace!("Key bindings file parsed!");
	}
}

pub fn unique_key_and_help(help: Span<'static>, key: Span<'static>) -> Vec<Span<'static>> {
	if help.to_string() == key.to_string() {
		vec![help]
	} else {
		vec![help, Span::raw(" "), key]
	}
}
