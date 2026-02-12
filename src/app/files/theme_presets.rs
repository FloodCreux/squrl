use std::fs;
use std::path::Path;

use tracing::{trace, warn};

use crate::app::files::theme::Theme;

/// Built-in theme definitions embedded at compile time
pub mod builtin {
	pub const DEFAULT: &str = include_str!("themes/default.toml");
	pub const DRACULA: &str = include_str!("themes/dracula.toml");
	pub const CATPPUCCIN_MOCHA: &str = include_str!("themes/catppuccin_mocha.toml");
	pub const CATPPUCCIN_MACCHIATO: &str = include_str!("themes/catppuccin_macchiato.toml");
	pub const CATPPUCCIN_FRAPPE: &str = include_str!("themes/catppuccin_frappe.toml");
	pub const CATPPUCCIN_LATTE: &str = include_str!("themes/catppuccin_latte.toml");
	pub const GRUVBOX: &str = include_str!("themes/gruvbox.toml");
	pub const OPENCODE: &str = include_str!("themes/opencode.toml");
	pub const VSCODE_DARK: &str = include_str!("themes/vscode_dark.toml");
}

/// All built-in theme names and their TOML content
pub const BUILTIN_THEMES: &[(&str, &str)] = &[
	("default", builtin::DEFAULT),
	("dracula", builtin::DRACULA),
	("catppuccin_mocha", builtin::CATPPUCCIN_MOCHA),
	("catppuccin_macchiato", builtin::CATPPUCCIN_MACCHIATO),
	("catppuccin_frappe", builtin::CATPPUCCIN_FRAPPE),
	("catppuccin_latte", builtin::CATPPUCCIN_LATTE),
	("gruvbox", builtin::GRUVBOX),
	("opencode", builtin::OPENCODE),
	("vscode_dark", builtin::VSCODE_DARK),
];

/// Returns a list of all built-in theme names
pub fn get_builtin_theme_names() -> Vec<&'static str> {
	BUILTIN_THEMES.iter().map(|(name, _)| *name).collect()
}

/// Load user-defined themes from a directory
/// Returns a vector of (name, toml_content) pairs
pub fn load_user_themes(themes_dir: &Path) -> Vec<(String, String)> {
	let mut user_themes = Vec::new();

	if !themes_dir.exists() {
		trace!(
			"User themes directory does not exist: {}",
			themes_dir.display()
		);
		return user_themes;
	}

	let entries = match fs::read_dir(themes_dir) {
		Ok(entries) => entries,
		Err(e) => {
			warn!("Could not read user themes directory: {}", e);
			return user_themes;
		}
	};

	for entry in entries.flatten() {
		let path = entry.path();
		if path.extension().is_some_and(|ext| ext == "toml")
			&& let Some(stem) = path.file_stem().and_then(|s| s.to_str())
		{
			// Skip if it conflicts with a built-in theme name
			if BUILTIN_THEMES.iter().any(|(name, _)| *name == stem) {
				warn!(
					"User theme '{}' conflicts with built-in theme, skipping",
					stem
				);
				continue;
			}

			match fs::read_to_string(&path) {
				Ok(content) => {
					trace!("Loaded user theme: {}", stem);
					user_themes.push((stem.to_string(), content));
				}
				Err(e) => {
					warn!("Could not read user theme '{}': {}", stem, e);
				}
			}
		}
	}

	user_themes
}

/// Get all available themes (built-in + user themes)
/// Returns a vector of theme names
pub fn get_all_theme_names(user_themes_dir: Option<&Path>) -> Vec<String> {
	let mut names: Vec<String> = get_builtin_theme_names()
		.into_iter()
		.map(String::from)
		.collect();

	if let Some(dir) = user_themes_dir {
		let user_themes = load_user_themes(dir);
		names.extend(user_themes.into_iter().map(|(name, _)| name));
	}

	names
}

/// Parse a theme from TOML content
pub fn parse_theme_toml(content: &str) -> Result<Theme, String> {
	toml::from_str(content).map_err(|e| format!("Failed to parse theme: {}", e))
}

/// Load a theme by name
/// First checks built-in themes, then user themes directory
pub fn load_theme_by_name(name: &str, user_themes_dir: Option<&Path>) -> Result<Theme, String> {
	// Check built-in themes first
	for (builtin_name, content) in BUILTIN_THEMES {
		if *builtin_name == name {
			trace!("Loading built-in theme: {}", name);
			return parse_theme_toml(content);
		}
	}

	// Check user themes directory
	if let Some(dir) = user_themes_dir {
		let theme_path = dir.join(format!("{}.toml", name));
		if theme_path.exists() {
			trace!("Loading user theme: {}", name);
			let content = fs::read_to_string(&theme_path)
				.map_err(|e| format!("Could not read theme file: {}", e))?;
			return parse_theme_toml(&content);
		}
	}

	Err(format!("Theme '{}' not found", name))
}

/// Export a built-in theme to a file
pub fn export_theme(name: &str, output_path: &Path) -> Result<(), String> {
	for (builtin_name, content) in BUILTIN_THEMES {
		if *builtin_name == name {
			fs::write(output_path, content)
				.map_err(|e| format!("Could not write theme file: {}", e))?;
			return Ok(());
		}
	}

	Err(format!("Built-in theme '{}' not found", name))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_builtin_themes_parse() {
		for (name, content) in BUILTIN_THEMES {
			let result = parse_theme_toml(content);
			assert!(
				result.is_ok(),
				"Failed to parse built-in theme '{}': {:?}",
				name,
				result.err()
			);
		}
	}

	#[test]
	fn test_get_builtin_theme_names() {
		let names = get_builtin_theme_names();
		assert!(names.contains(&"default"));
		assert!(names.contains(&"dracula"));
		assert!(names.contains(&"catppuccin_mocha"));
		assert!(names.contains(&"gruvbox"));
	}
}
