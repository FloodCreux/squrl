use std::path::PathBuf;

#[derive(clap::Args, Debug, Clone)]
pub struct CompletionsCommand {
	pub shell: String,

	#[clap(value_hint = clap::ValueHint::FilePath)]
	pub output_directory: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;

	#[derive(Parser)]
	struct CompletionsCli {
		#[command(flatten)]
		cmd: CompletionsCommand,
	}

	// === CompletionsCommand defaults ===

	#[test]
	fn completions_defaults_output_directory_to_none() {
		let cli = CompletionsCli::try_parse_from(["test", "bash"]).unwrap();
		assert!(cli.cmd.output_directory.is_none());
	}

	// === CompletionsCommand parsing ===

	#[test]
	fn completions_parses_bash_shell() {
		let cli = CompletionsCli::try_parse_from(["test", "bash"]).unwrap();
		assert_eq!(cli.cmd.shell, "bash");
	}

	#[test]
	fn completions_parses_zsh_shell() {
		let cli = CompletionsCli::try_parse_from(["test", "zsh"]).unwrap();
		assert_eq!(cli.cmd.shell, "zsh");
	}

	#[test]
	fn completions_parses_fish_shell() {
		let cli = CompletionsCli::try_parse_from(["test", "fish"]).unwrap();
		assert_eq!(cli.cmd.shell, "fish");
	}

	#[test]
	fn completions_parses_elvish_shell() {
		let cli = CompletionsCli::try_parse_from(["test", "elvish"]).unwrap();
		assert_eq!(cli.cmd.shell, "elvish");
	}

	#[test]
	fn completions_parses_powershell_shell() {
		let cli = CompletionsCli::try_parse_from(["test", "powershell"]).unwrap();
		assert_eq!(cli.cmd.shell, "powershell");
	}

	#[test]
	fn completions_parses_output_directory() {
		let dir = std::env::temp_dir().join("completions");
		let dir_str = dir.to_str().expect("temp dir should be valid UTF-8");
		let cli = CompletionsCli::try_parse_from(["test", "bash", dir_str]).unwrap();
		assert_eq!(cli.cmd.output_directory.unwrap(), dir);
	}

	#[test]
	fn completions_requires_shell_argument() {
		let result = CompletionsCli::try_parse_from(["test"]);
		assert!(result.is_err());
	}

	#[test]
	fn completions_accepts_arbitrary_shell_string() {
		let cli = CompletionsCli::try_parse_from(["test", "nushell"]).unwrap();
		assert_eq!(cli.cmd.shell, "nushell");
	}
}
