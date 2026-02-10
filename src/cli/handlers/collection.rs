use crate::app::app::App;
use crate::cli::commands::collection_commands::collection_commands::{
	CollectionCommand, CollectionSubcommand,
};
use crate::models::collection::Collection;

impl App<'_> {
	pub async fn handle_collection_command(
		&mut self,
		collection_command: &CollectionCommand,
	) -> anyhow::Result<()> {
		if let Some(environment_name) = &collection_command.env {
			let environment_index = self.find_environment(&environment_name)?;
			self.selected_environment = environment_index;
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
		}
	}

	pub fn list_collections(&mut self, with_request_names: bool) -> anyhow::Result<()> {
		for collection in &self.collections {
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
		let collection = &self.collections[collection_index];

		print_collection(&collection, false, !without_request_names);

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
