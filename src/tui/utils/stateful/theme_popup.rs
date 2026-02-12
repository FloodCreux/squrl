use crate::app::files::theme::{Theme, get_theme, load_theme_by_name, set_theme};
use crate::app::files::theme_presets::get_all_theme_names;
use crate::cli::args::ARGS;

/// Popup for selecting a theme with live preview
pub struct ThemePopup {
	/// List of all available theme names
	pub themes: Vec<String>,
	/// Currently selected index
	pub selection: usize,
	/// Original theme to restore on cancel
	original_theme: Theme,
	/// Name of the currently selected theme
	pub selected_theme_name: String,
}

impl Default for ThemePopup {
	fn default() -> Self {
		Self::new()
	}
}

impl ThemePopup {
	pub fn new() -> Self {
		let user_themes_dir = ARGS
			.user_config_directory
			.as_ref()
			.map(|d| d.join("themes"));
		let themes = get_all_theme_names(user_themes_dir.as_deref());

		ThemePopup {
			themes,
			selection: 0,
			original_theme: Theme::default(),
			selected_theme_name: String::from("default"),
		}
	}

	/// Initialize the popup, capturing the current theme for potential cancel
	pub fn init(&mut self) {
		self.original_theme = get_theme();
		self.selected_theme_name = String::from("default");

		// Refresh theme list
		let user_themes_dir = ARGS
			.user_config_directory
			.as_ref()
			.map(|d| d.join("themes"));
		self.themes = get_all_theme_names(user_themes_dir.as_deref());

		// Try to find the current theme in the list and select it
		// For now, just start at index 0
		self.selection = 0;
	}

	/// Move selection to the next theme and apply preview
	pub fn next(&mut self) {
		if self.themes.is_empty() {
			return;
		}

		if self.selection + 1 < self.themes.len() {
			self.selection += 1;
		} else {
			self.selection = 0;
		}

		self.apply_preview();
	}

	/// Move selection to the previous theme and apply preview
	pub fn previous(&mut self) {
		if self.themes.is_empty() {
			return;
		}

		if self.selection > 0 {
			self.selection -= 1;
		} else {
			self.selection = self.themes.len() - 1;
		}

		self.apply_preview();
	}

	/// Apply the currently selected theme as a live preview
	pub fn apply_preview(&mut self) {
		if let Some(theme_name) = self.themes.get(self.selection) {
			let user_themes_dir = ARGS
				.user_config_directory
				.as_ref()
				.map(|d| d.join("themes"));
			if load_theme_by_name(theme_name, user_themes_dir.as_deref()).is_ok() {
				self.selected_theme_name = theme_name.clone();
			}
		}
	}

	/// Get the currently selected theme name
	pub fn get_selected_theme(&self) -> Option<&String> {
		self.themes.get(self.selection)
	}

	/// Confirm the selection - theme is already applied, just return the name
	pub fn confirm(&self) -> Option<String> {
		self.themes.get(self.selection).cloned()
	}

	/// Cancel and restore the original theme
	pub fn cancel(&mut self) {
		set_theme(self.original_theme.clone());
	}
}
