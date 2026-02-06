use serde::{Deserialize, Serialize};

#[derive(clap::Args, Clone, Default, Debug, Serialize, Deserialize)]
pub struct BasicAuth {
	pub username: String,
	pub password: String,
}
