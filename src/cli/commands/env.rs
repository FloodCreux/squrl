use crate::cli::commands::key::KeyCommand;
use clap::Subcommand;

#[derive(clap::Args, Debug, Clone)]
pub struct EnvCommand {
	#[command(subcommand)]
	pub env_subcommand: EnvSubCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum EnvSubCommand {
	Info {
		env_name: String,

		#[clap(short, long, default_value_t = false)]
		os_vars: bool,
	},
	Key {
		env_name: String,

		#[command(subcommand)]
		subcommand: KeyCommand,
	},
}
