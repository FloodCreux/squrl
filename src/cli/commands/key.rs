use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum KeyCommand {
	Get { key: String },
	Set { key: String, value: String },
	Add { key: String, value: String },
	Delete { key: String },
	Rename { key: String, new_key: String },
}
