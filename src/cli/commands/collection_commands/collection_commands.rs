use crate::cli::commands::key::KeyCommand;
use crate::cli::commands::request_commands::send::SendCommand;
use clap::Subcommand;

#[derive(clap::Args, Debug, Clone)]
pub struct CollectionCommand {
	#[command(subcommand)]
	pub collection_subcommand: CollectionSubcommand,

	/// Name of the global environment to use
	#[arg(long, global = true)]
	pub env: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CollectionSubcommand {
	/// List all collections
	List {
		/// Also print request names
		#[arg(long)]
		request_names: bool,
	},

	/// Describe a collection
	Info {
		/// Collection name
		collection_name: String,

		/// Also print request names
		#[arg(long)]
		without_request_names: bool,
	},

	/// Create a new collection
	New {
		/// Collection name
		collection_name: String,
	},

	/// Delete a collection
	Delete {
		/// Collection name
		/// e.g. my_collection, "my collection"
		collection_name: String,
	},

	/// Rename a collection
	Rename {
		/// e.g. my_collection, "my collection"
		collection_name: String,

		/// New collection name
		new_collection_name: String,
	},

	/// Send all the collection's requests
	Send {
		/// e.g. my_collection, "my collection"
		collection_name: String,

		#[clap(flatten)]
		subcommand: SendCommand,
	},

	/// Manage collection-scoped environments
	Env {
		/// Collection name
		collection_name: String,

		#[command(subcommand)]
		subcommand: CollectionEnvSubcommand,
	},
}

#[derive(Subcommand, Debug, Clone)]
pub enum CollectionEnvSubcommand {
	/// List environments in a collection
	List,

	/// Create a new environment in a collection
	Create {
		/// Environment name (e.g. "dev", "prod")
		env_name: String,
	},

	/// Delete an environment from a collection
	Delete {
		/// Environment name
		env_name: String,
	},

	/// Select which collection environment is active
	Select {
		/// Environment name (omit to deselect)
		env_name: Option<String>,
	},

	/// Show info about a collection environment
	Info {
		/// Environment name
		env_name: String,
	},

	/// Manage keys in a collection environment
	Key {
		/// Environment name
		env_name: String,

		#[command(subcommand)]
		subcommand: KeyCommand,
	},
}
