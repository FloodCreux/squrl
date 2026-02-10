use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ImportPostmanEnvironmentError {
	#[error("Could not read Postman environment file\n\t{0}")]
	CouldNotReadFile(String),
	#[error("Could not parse Postman environment\n\t{0}")]
	CouldNotParsePostmanEnvironment(String),
}

#[derive(Deserialize)]
pub struct PostmanEnv {
	#[serde(rename = "id")]
	pub _id: Uuid,
	pub name: String,
	pub values: Vec<PostmanEnvVariable>,
	pub _postman_variable_scope: String,
	pub _postman_exported_at: String,
	pub _postman_exported_using: String,
}

#[derive(Deserialize)]
pub struct PostmanEnvVariable {
	pub key: String,
	pub value: String,
	#[serde(rename = "type")]
	pub _type: String,
	pub enabled: bool,
}
