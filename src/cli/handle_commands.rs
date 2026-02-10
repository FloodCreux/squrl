use crate::app::app::App;
use crate::cli::args::Command;
use crate::cli::args::Command::*;
use crate::cli::commands::env::{EnvCommand, EnvSubCommand};
use crate::cli::commands::import::ImportType;
use crate::cli::commands::key::KeyCommand;
use crate::cli::handlers::completions::generate_completions;
use crate::cli::handlers::man::generate_man_pages;
use crate::errors::panic_error;

impl App<'_> {
	pub async fn handle_command(&mut self, command: Command) {
		let result = match &command {
			Collection(collection_command) => {
				self.handle_collection_command(collection_command).await
			}
			Completions(completions_command) => generate_completions(completions_command),
			Env(env_command) => self.handle_env_commands(env_command),
			Import(import_command) => match &import_command.import_type {
				ImportType::Postman(postman_import) => {
					self.import_postman_collection(postman_import)
				}
				ImportType::PostmanEnv(postman_env_import) => {
					self.import_postman_environment(postman_env_import)
				}
				ImportType::Curl(curl_import) => self.import_curl_file(curl_import),
				ImportType::OpenApi(openapi_import) => {
					self.import_openapi_collection(openapi_import)
				}
			},
			Man(_) => generate_man_pages(),
			Request(request_command) => self.handle_request_command(request_command).await,
			Try(try_command) => {
				self.try_request(&try_command.new_request_command, &try_command.send_command)
					.await
			}
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
