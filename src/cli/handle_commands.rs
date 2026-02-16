use crate::app::App;
use crate::cli::args::Command;
use crate::cli::args::Command::*;
use crate::cli::commands::import::ImportType;
use crate::cli::handlers::completions::generate_completions;
use crate::cli::handlers::man::generate_man_pages;
use crate::cli::handlers::theme::handle_theme_command;
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
				ImportType::HttpFile(http_file_import) => self.import_http_file(http_file_import),
			},
			Man(_) => generate_man_pages(),
			Request(request_command) => self.handle_request_command(request_command).await,
			Theme(theme_command) => handle_theme_command(theme_command),
			Try(try_command) => {
				self.try_request(&try_command.new_request_command, &try_command.send_command)
					.await
			}
		};

		if let Err(error) = result {
			panic_error(error.to_string());
		}
	}
}
