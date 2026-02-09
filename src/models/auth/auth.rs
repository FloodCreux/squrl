use clap::Subcommand;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::models::auth::basic::BasicAuth;
use crate::models::auth::bearer_token::BearerToken;
use crate::models::auth::digest::Digest;
use crate::models::auth::jwt::JwtToken;

#[derive(Subcommand, Clone, Default, Debug, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Auth {
	#[default]
	#[strum(to_string = "No Auth")]
	NoAuth,

	#[strum(to_string = "Basic")]
	#[clap(visible_alias = "basic")]
	BasicAuth(BasicAuth),

	#[strum(to_string = "Bearer")]
	#[clap(visible_alias = "bearer")]
	BearerToken(BearerToken),

	#[strum(to_string = "JWT")]
	#[clap(visible_alias = "jwt")]
	JwtToken(JwtToken),

	#[strum(to_string = "Digest")]
	#[clap(visible_alias = "digest")]
	Digest(Digest),
}

impl Auth {
	pub fn get_digest_mut(&mut self) -> &mut Digest {
		match self {
			Auth::Digest(digest) => digest,
			_ => unreachable!(),
		}
	}
}
