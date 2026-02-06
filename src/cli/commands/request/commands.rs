use clap::Subcommand;

#[derive(clap::Args, Debug, Clone)]
pub struct RequestCommand {
	#[command(subcommand)]
	pub request_subcommand: RequestSubcommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum RequestSubcommand {}
