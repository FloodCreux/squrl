use clap::Subcommand;

use crate::cli::commands::key_value::KeyValueCommand;
use crate::cli::commands::request_commands::auth::AuthCommand;
use crate::cli::commands::request_commands::body::BodySubcommand;
use crate::cli::commands::request_commands::method::MethodCommand;
use crate::cli::commands::request_commands::new::NewRequestCommand;
use crate::cli::commands::request_commands::scripts::ScriptsCommand;
use crate::cli::commands::request_commands::send::SendCommand;
use crate::cli::commands::request_commands::settings::SettingsCommand;
use crate::cli::commands::request_commands::url::UrlCommand;
use crate::cli::utils::arguments_validators::collection_slash_request_validator;
use crate::models::export::ExportFormat;

#[derive(clap::Args, Debug, Clone)]
pub struct RequestCommand {
	#[command(subcommand)]
	pub request_subcommand: RequestSubcommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum RequestSubcommand {
	Info {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),
	},
	New {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),

		#[clap(flatten)]
		subcommand: NewRequestCommand,
	},
	Delete {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),
	},
	Rename {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),

		new_request_name: String,
	},
	Url {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),

		#[command(subcommand)]
		subcommand: UrlCommand,
	},
	Method {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),

		#[command(subcommand)]
		subcommand: MethodCommand,
	},
	Params {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),

		#[command(subcommand)]
		subcommand: KeyValueCommand,
	},
	Auth {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),

		#[command(subcommand)]
		subcommand: AuthCommand,
	},
	Header {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),

		#[command(subcommand)]
		subcommand: KeyValueCommand,
	},
	Body {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),

		#[command(subcommand)]
		subcommand: BodySubcommand,
	},
	Scripts {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),

		#[command(subcommand)]
		subcommand: ScriptsCommand,
	},
	Send {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),

		#[clap(flatten)]
		subcommand: SendCommand,
	},
	Settings {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),

		#[command(subcommand)]
		subcommand: SettingsCommand,
	},
	Export {
		#[arg(value_parser=collection_slash_request_validator)]
		collection_slash_request: (String, String),

		format: ExportFormat,
	},
}
