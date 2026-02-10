use crate::models::auth::auth::Auth;
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum AuthCommand {
	/// Print the current request auth method
	Get,
	/// Set the request auth method
	Set {
		#[command(subcommand)]
		auth_method: Auth,
	},
}
