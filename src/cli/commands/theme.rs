use clap::Subcommand;

#[derive(clap::Args, Debug, Clone)]
pub struct ThemeCommand {
	#[command(subcommand)]
	pub subcommand: ThemeSubCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ThemeSubCommand {
	/// List all available themes
	List,

	/// Preview a theme by printing a colored sample
	Preview {
		/// Name of the theme to preview
		name: String,
	},

	/// Export a built-in theme to ~/.config/squrl/themes/ for customization
	Export {
		/// Name of the built-in theme to export
		name: String,
	},
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;

	#[derive(Parser)]
	struct ThemeCli {
		#[command(flatten)]
		cmd: ThemeCommand,
	}

	#[test]
	fn theme_list_parses() {
		let cli = ThemeCli::try_parse_from(["test", "list"]).unwrap();
		assert!(matches!(cli.cmd.subcommand, ThemeSubCommand::List));
	}

	#[test]
	fn theme_preview_parses_name() {
		let cli = ThemeCli::try_parse_from(["test", "preview", "dracula"]).unwrap();
		match cli.cmd.subcommand {
			ThemeSubCommand::Preview { name } => assert_eq!(name, "dracula"),
			_ => panic!("Expected Preview subcommand"),
		}
	}

	#[test]
	fn theme_export_parses_name() {
		let cli = ThemeCli::try_parse_from(["test", "export", "gruvbox"]).unwrap();
		match cli.cmd.subcommand {
			ThemeSubCommand::Export { name } => assert_eq!(name, "gruvbox"),
			_ => panic!("Expected Export subcommand"),
		}
	}

	#[test]
	fn theme_preview_requires_name() {
		let result = ThemeCli::try_parse_from(["test", "preview"]);
		assert!(result.is_err());
	}

	#[test]
	fn theme_export_requires_name() {
		let result = ThemeCli::try_parse_from(["test", "export"]);
		assert!(result.is_err());
	}
}
