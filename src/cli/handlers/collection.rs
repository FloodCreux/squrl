use crate::app::App;
use crate::cli::commands::collection_commands::collection_commands::{
	CollectionCommand, CollectionEnvSubcommand, CollectionSubcommand,
};
use crate::cli::commands::key::KeyCommand;
use crate::models::collection::Collection;

impl App<'_> {
	pub async fn handle_collection_command(
		&mut self,
		collection_command: &CollectionCommand,
	) -> anyhow::Result<()> {
		if let Some(environment_name) = &collection_command.env {
			let environment_index = self.find_environment(environment_name)?;
			self.core.selected_environment = environment_index;
		}

		match &collection_command.collection_subcommand {
			CollectionSubcommand::Info {
				collection_name,
				without_request_names: with_request_names,
			} => self.describe_collection(collection_name, *with_request_names),
			CollectionSubcommand::List {
				request_names: with_request_names,
			} => self.list_collections(*with_request_names),
			CollectionSubcommand::New { collection_name } => {
				self.new_collection(collection_name.clone())
			}
			CollectionSubcommand::Delete { collection_name } => {
				self.cli_delete_collection(collection_name)
			}
			CollectionSubcommand::Rename {
				collection_name,
				new_collection_name,
			} => self.cli_rename_collection(collection_name, new_collection_name.clone()),
			CollectionSubcommand::Send {
				collection_name,
				subcommand,
			} => self.cli_send_collection(collection_name, subcommand).await,
			CollectionSubcommand::Env {
				collection_name,
				subcommand,
			} => self.handle_collection_env_command(collection_name, subcommand),
		}
	}

	fn handle_collection_env_command(
		&mut self,
		collection_name: &str,
		subcommand: &CollectionEnvSubcommand,
	) -> anyhow::Result<()> {
		let collection_index = self.find_collection(collection_name)?;

		match subcommand {
			CollectionEnvSubcommand::List => {
				let collection = &self.core.collections[collection_index];
				if collection.environments.is_empty() {
					println!(
						"No environments defined in collection \"{}\"",
						collection_name
					);
				} else {
					let selected = collection.selected_environment.as_deref();
					for env in &collection.environments {
						let marker = if selected == Some(&env.name) {
							" *"
						} else {
							""
						};
						println!("{}{}", env.name, marker);
					}
				}
				Ok(())
			}
			CollectionEnvSubcommand::Create { env_name } => {
				self.create_collection_environment(collection_index, env_name.clone())
			}
			CollectionEnvSubcommand::Delete { env_name } => {
				self.delete_collection_environment(collection_index, env_name)
			}
			CollectionEnvSubcommand::Select { env_name } => {
				self.select_collection_environment(collection_index, env_name.clone())
			}
			CollectionEnvSubcommand::Info { env_name } => {
				let env_idx = self.find_collection_environment(collection_index, env_name)?;
				let env = &self.core.collections[collection_index].environments[env_idx];

				println!("name: {}", env.name);
				println!("values:");
				for (key, value) in &env.values {
					println!("\t{key}: {value}");
				}
				Ok(())
			}
			CollectionEnvSubcommand::Key {
				env_name,
				subcommand,
			} => match subcommand {
				KeyCommand::Get { key } => {
					let value = self.get_collection_env_value(collection_index, env_name, key)?;
					println!("{value}");
					Ok(())
				}
				KeyCommand::Set { key, value } => {
					self.set_collection_env_value(collection_index, env_name, key, value.clone())
				}
				KeyCommand::Add { key, value } => self.create_collection_env_value(
					collection_index,
					env_name,
					key.clone(),
					value.clone(),
				),
				KeyCommand::Delete { key } => {
					self.delete_collection_env_key(collection_index, env_name, key)
				}
				KeyCommand::Rename { key, new_key } => {
					self.rename_collection_env_key(collection_index, env_name, key, new_key)
				}
			},
		}
	}

	pub fn list_collections(&mut self, with_request_names: bool) -> anyhow::Result<()> {
		for collection in &self.core.collections {
			print_collection(collection, !with_request_names, with_request_names);

			if with_request_names {
				println!();
			}
		}

		Ok(())
	}

	pub fn describe_collection(
		&mut self,
		collection_name: &str,
		without_request_names: bool,
	) -> anyhow::Result<()> {
		let collection_index = self.find_collection(collection_name)?;
		let collection = &self.core.collections[collection_index];

		print_collection(collection, false, !without_request_names);

		Ok(())
	}

	pub fn cli_delete_collection(&mut self, collection_name: &str) -> anyhow::Result<()> {
		let collection_index = self.find_collection(collection_name)?;

		self.delete_collection(collection_index);

		Ok(())
	}

	pub fn cli_rename_collection(
		&mut self,
		collection_name: &str,
		new_collection_name: String,
	) -> anyhow::Result<()> {
		let collection_index = self.find_collection(collection_name)?;

		self.rename_collection(collection_index, new_collection_name)?;

		Ok(())
	}
}

fn print_collection(collection: &Collection, shortened: bool, with_request_names: bool) {
	if shortened {
		println!("{}", collection.name);
	} else {
		println!("collection: {}", collection.name);
	}

	if with_request_names {
		println!("requests:");
		for request in &collection.requests {
			let local_request = request.read();
			println!("\t{}", local_request.name);
		}
	}
}
