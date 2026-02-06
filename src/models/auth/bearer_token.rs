use serde::{Deserialize, Serialize};

#[derive(clap::Args, Clone, Default, Debug, Serialize, Deserialize)]
pub struct BearerToken {
	pub token: String,
}
