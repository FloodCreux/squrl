use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GraphqlRequest {
	pub query: String,
	pub variables: String,
	pub operation_name: Option<String>,
}
