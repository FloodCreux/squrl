use clap::Subcommand;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Subcommand, Clone, Default, Debug, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Auth {
	#[default]
	#[strum(to_string = "No Auth")]
	NoAuth,
}
