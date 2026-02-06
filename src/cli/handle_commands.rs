use crate::app::app::App;
use crate::cli::args::Command;
use crate::cli::args::Command::*;
use crate::cli::commands::env::{EnvCommand, EnvSubCommand};
use crate::cli::commands::key::KeyCommand;
use crate::cli::man::generate_man_pages;
use crate::errors::panic_error;

impl App {
	pub async fn handle_command(&mut self, command: Command) {
		let result = match &command {
			Env(env_command) => self.handle_env_commands(env_command),
			Man(_) => generate_man_pages(),
		};

		if let Err(error) = result {
			panic_error(error.to_string());
		}
	}

	fn handle_env_commands(&mut self, env_command: &EnvCommand) -> anyhow::Result<()> {
		let env_index = match &env_command.env_subcommand {
			EnvSubCommand::Info { env_name, .. } | EnvSubCommand::Key { env_name, .. } => {
				self.find_environment(env_name)?
			}
		};

		match &env_command.env_subcommand {
			EnvSubCommand::Info { os_vars, .. } => self.cli_describe_env(env_index, *os_vars),
			EnvSubCommand::Key { subcommand, .. } => match subcommand {
				KeyCommand::Get { key } => self.get_env_value(env_index, key),
				KeyCommand::Set { key, value } => self.set_env_value(env_index, key, value.clone()),
				KeyCommand::Add { key, value } => {
					self.create_env_value(env_index, Some(key.clone()), value.clone())
				}
				KeyCommand::Delete { key } => self.delete_env_key(env_index, key),
				KeyCommand::Rename { key, new_key } => self.rename_env_key(env_index, key, new_key),
			},
		}
	}
}
