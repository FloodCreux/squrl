use crate::cli::commands::env::EnvCommand;
use clap::builder::Styles;
use clap::{Parser, Subcommand};
use lazy_static::lazy_static;

#[derive(Parser, Debug)]
#[command(version, about, styles = Styles::styled())]
pub struct Args {
	#[command(subcommand)]
	pub command: Option<Command>,

	#[arg(long, global = true, default_value_t = false)]
	pub dry_run: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
	Env(EnvCommand),
}

lazy_static! {
	pub static ref ARGS: GlobalArgs = {
		let args = Args::parse();

		GlobalArgs {
			command: args.command,
			should_save: !args.dry_run,
		}
	};
}

#[derive(Debug)]
pub struct GlobalArgs {
	pub command: Option<Command>,
	pub should_save: bool,
}
