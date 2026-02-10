use clap::Subcommand;

use crate::cli::commands::key::KeyCommand;

#[derive(Subcommand, Debug, Clone)]
pub enum KeyValueCommand {
	#[command(flatten)]
	Key(KeyCommand),

	Toggle {
		key: String,
		state: Option<bool>,
	},

	All,
}
