use std::path::PathBuf;
use std::{env, fs};

use clap::builder::Styles;
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use lazy_static::lazy_static;

use crate::app::files::utils::expand_tilde;
use crate::cli::commands::env::EnvCommand;
use crate::cli::commands::man::ManCommand;
use crate::cli::commands::try_command::TryCommand;
use crate::errors::panic_error;

#[derive(Parser, Debug)]
#[command(version, about, styles = Styles::styled())]
pub struct Args {
	#[arg(short, long, value_hint = clap::ValueHint::DirPath)]
	pub directory: Option<PathBuf>,

	#[command(subcommand)]
	pub command: Option<Command>,

	#[arg(long, global = true, default_value_t = false)]
	pub dry_run: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
	Env(EnvCommand),
	Man(ManCommand),
	Try(TryCommand),
}

lazy_static! {
	pub static ref ARGS: GlobalArgs = {
		let args = Args::parse();
		let config_directory = get_app_config_dir();

		let (directory, should_parse_directory) = match &args.command {
			Some(command) => match command.clone() {
				Command::Man(ManCommand {
					output_directory, ..
				}) => (output_directory, false),
				_ => (
					Some(choose_app_directory(args.directory, &config_directory)),
					true,
				),
			},
			None => (
				Some(choose_app_directory(args.directory, &config_directory)),
				true,
			),
		};

		GlobalArgs {
			directory,
			config_directory,
			command: args.command,
			should_save: !args.dry_run,
			should_parse_directory,
		}
	};
}

fn get_app_config_dir() -> Option<PathBuf> {
	let project_directory = ProjectDirs::from("com", "flood-creux", "squrl");

	match project_directory {
		Some(project_directory) => {
			let config_directory = project_directory.config_dir().to_path_buf();

			if !config_directory.exists() {
				fs::create_dir_all(&config_directory).expect(&format!(
					"Could not recursively create folder \"{}\"",
					config_directory.display()
				));
			}

			Some(config_directory)
		}
		None => None,
	}
}

fn choose_app_directory(path_buf: Option<PathBuf>, config_directory: &Option<PathBuf>) -> PathBuf {
	match path_buf {
		Some(directory) => expand_tilde(directory),
		None => match env::var("SQURL_MAIN_DIR") {
			Ok(env_directory) => expand_tilde(PathBuf::from(env_directory)),
			Err(_) => match config_directory {
				Some(config_directory) => config_directory.clone(),
				None => panic_error(
					"No directory provided, provide one either with `--directory <dir>` or via the enfironment variable `SQURL_MAIN_DIR`",
				),
			},
		},
	}
}

#[derive(Debug)]
pub struct GlobalArgs {
	pub directory: Option<PathBuf>,
	pub config_directory: Option<PathBuf>,
	pub command: Option<Command>,
	pub should_save: bool,
	pub should_parse_directory: bool,
}
