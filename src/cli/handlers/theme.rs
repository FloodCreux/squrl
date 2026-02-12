use std::fs;

use anyhow::anyhow;

use crate::app::files::theme_presets::{
	export_theme, get_all_theme_names, get_builtin_theme_names, load_theme_by_name,
};
use crate::cli::args::ARGS;
use crate::cli::commands::theme::{ThemeCommand, ThemeSubCommand};

pub fn handle_theme_command(theme_command: &ThemeCommand) -> anyhow::Result<()> {
	match &theme_command.subcommand {
		ThemeSubCommand::List => list_themes(),
		ThemeSubCommand::Preview { name } => preview_theme(name),
		ThemeSubCommand::Export { name } => export_theme_to_user_dir(name),
	}
}

fn list_themes() -> anyhow::Result<()> {
	let user_themes_dir = ARGS
		.user_config_directory
		.as_ref()
		.map(|d| d.join("themes"));
	let names = get_all_theme_names(user_themes_dir.as_deref());

	for name in names {
		println!("{}", name);
	}

	Ok(())
}

fn preview_theme(name: &str) -> anyhow::Result<()> {
	let user_themes_dir = ARGS
		.user_config_directory
		.as_ref()
		.map(|d| d.join("themes"));
	let theme =
		load_theme_by_name(name, user_themes_dir.as_deref()).map_err(|e| anyhow!("{}", e))?;

	// Print a colored preview of the theme
	println!("Theme: {}\n", name);

	// Helper to convert Color to ANSI escape code
	fn color_to_ansi(color: ratatui::style::Color) -> String {
		match color {
			ratatui::style::Color::Rgb(r, g, b) => format!("\x1b[38;2;{};{};{}m", r, g, b),
			ratatui::style::Color::Black => "\x1b[30m".to_string(),
			ratatui::style::Color::Red => "\x1b[31m".to_string(),
			ratatui::style::Color::Green => "\x1b[32m".to_string(),
			ratatui::style::Color::Yellow => "\x1b[33m".to_string(),
			ratatui::style::Color::Blue => "\x1b[34m".to_string(),
			ratatui::style::Color::Magenta => "\x1b[35m".to_string(),
			ratatui::style::Color::Cyan => "\x1b[36m".to_string(),
			ratatui::style::Color::Gray => "\x1b[37m".to_string(),
			ratatui::style::Color::DarkGray => "\x1b[90m".to_string(),
			ratatui::style::Color::LightRed => "\x1b[91m".to_string(),
			ratatui::style::Color::LightGreen => "\x1b[92m".to_string(),
			ratatui::style::Color::LightYellow => "\x1b[93m".to_string(),
			ratatui::style::Color::LightBlue => "\x1b[94m".to_string(),
			ratatui::style::Color::LightMagenta => "\x1b[95m".to_string(),
			ratatui::style::Color::LightCyan => "\x1b[96m".to_string(),
			ratatui::style::Color::White => "\x1b[97m".to_string(),
			_ => "\x1b[0m".to_string(),
		}
	}

	fn color_to_ansi_bg(color: ratatui::style::Color) -> String {
		match color {
			ratatui::style::Color::Rgb(r, g, b) => format!("\x1b[48;2;{};{};{}m", r, g, b),
			ratatui::style::Color::Black => "\x1b[40m".to_string(),
			ratatui::style::Color::Red => "\x1b[41m".to_string(),
			ratatui::style::Color::Green => "\x1b[42m".to_string(),
			ratatui::style::Color::Yellow => "\x1b[43m".to_string(),
			ratatui::style::Color::Blue => "\x1b[44m".to_string(),
			ratatui::style::Color::Magenta => "\x1b[45m".to_string(),
			ratatui::style::Color::Cyan => "\x1b[46m".to_string(),
			ratatui::style::Color::Gray => "\x1b[47m".to_string(),
			ratatui::style::Color::DarkGray => "\x1b[100m".to_string(),
			ratatui::style::Color::LightRed => "\x1b[101m".to_string(),
			ratatui::style::Color::LightGreen => "\x1b[102m".to_string(),
			ratatui::style::Color::LightYellow => "\x1b[103m".to_string(),
			ratatui::style::Color::LightBlue => "\x1b[104m".to_string(),
			ratatui::style::Color::LightMagenta => "\x1b[105m".to_string(),
			ratatui::style::Color::LightCyan => "\x1b[106m".to_string(),
			ratatui::style::Color::White => "\x1b[107m".to_string(),
			_ => "\x1b[0m".to_string(),
		}
	}

	let reset = "\x1b[0m";

	// UI colors
	println!("UI Colors:");
	println!(
		"  {}Font Color{}: Sample text",
		color_to_ansi(theme.ui.font_color),
		reset
	);
	println!(
		"  {}Main Foreground{}: Sample text",
		color_to_ansi(theme.ui.main_foreground_color),
		reset
	);
	println!(
		"  {}Secondary Foreground{}: Sample text",
		color_to_ansi(theme.ui.secondary_foreground_color),
		reset
	);
	println!(
		"  {}{}Main Background{}: Sample text",
		color_to_ansi_bg(theme.ui.main_background_color),
		color_to_ansi(theme.ui.font_color),
		reset
	);
	println!(
		"  {}{}Secondary Background{}: Sample text",
		color_to_ansi_bg(theme.ui.secondary_background_color),
		color_to_ansi(theme.ui.font_color),
		reset
	);

	// HTTP Methods
	println!("\nHTTP Methods:");
	println!(
		"  {}GET{}  {}POST{}  {}PUT{}  {}PATCH{}  {}DELETE{}",
		color_to_ansi(theme.http.methods.get),
		reset,
		color_to_ansi(theme.http.methods.post),
		reset,
		color_to_ansi(theme.http.methods.put),
		reset,
		color_to_ansi(theme.http.methods.patch),
		reset,
		color_to_ansi(theme.http.methods.delete),
		reset,
	);
	println!(
		"  {}HEAD{}  {}OPTIONS{}  {}TRACE{}  {}CONNECT{}",
		color_to_ansi(theme.http.methods.head),
		reset,
		color_to_ansi(theme.http.methods.options),
		reset,
		color_to_ansi(theme.http.methods.trace),
		reset,
		color_to_ansi(theme.http.methods.connect),
		reset,
	);

	// Other colors
	println!("\nOther Colors:");
	println!(
		"  {}Selection Highlight{}: Selected item",
		color_to_ansi(theme.others.selection_highlight_color),
		reset
	);
	println!(
		"  {}Environment Variable{}: {{{{variable}}}}",
		color_to_ansi(theme.others.environment_variable_highlight_color),
		reset
	);

	// WebSocket
	println!("\nWebSocket:");
	println!(
		"  {}Connected{}  {}Disconnected{}",
		color_to_ansi(theme.websocket.connection_status.connected),
		reset,
		color_to_ansi(theme.websocket.connection_status.disconnected),
		reset,
	);

	Ok(())
}

fn export_theme_to_user_dir(name: &str) -> anyhow::Result<()> {
	// Verify the theme exists in built-in themes
	if !get_builtin_theme_names().contains(&name) {
		return Err(anyhow!(
			"Theme '{}' is not a built-in theme. Available built-in themes: {}",
			name,
			get_builtin_theme_names().join(", ")
		));
	}

	let user_themes_dir = ARGS
		.user_config_directory
		.as_ref()
		.map(|d| d.join("themes"))
		.ok_or_else(|| anyhow!("Could not determine user config directory"))?;

	// Create themes directory if it doesn't exist
	if !user_themes_dir.exists() {
		fs::create_dir_all(&user_themes_dir)?;
	}

	let output_path = user_themes_dir.join(format!("{}.toml", name));

	if output_path.exists() {
		return Err(anyhow!(
			"Theme file already exists at '{}'. Remove it first if you want to re-export.",
			output_path.display()
		));
	}

	export_theme(name, &output_path).map_err(|e| anyhow!("{}", e))?;

	println!("Exported theme '{}' to '{}'", name, output_path.display());
	println!("You can now customize this file and it will be available as a user theme.");

	Ok(())
}
