use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};
use strum::Display;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DigestError {
	#[error("Invalid header syntax: {0}")]
	InvalidHeaderSyntax(String),

	#[error("Missing required {0}")]
	MissingRequired(&'static str),

	#[error("Invalid algorithm: {0}")]
	InvalidAlgorithm(String),

	#[error("Invalid boolean for {0}: {1}")]
	InvalidBooleanValue(&'static str, String),

	#[error("Invalid charset: {0}")]
	InvalidCharset(String),
}

#[derive(Args, Default, Clone, Debug, Serialize, Deserialize)]
pub struct Digest {
	pub username: String,
	pub password: String,

	pub domains: String,
	pub realm: String,
	pub nonce: String,
	pub opaque: String,
	pub stale: bool,
}

#[derive(Debug, Default, Clone, ValueEnum, Display, Serialize, Deserialize)]
pub enum DigestAlgorithm {
	#[default]
	#[strum(to_string = "MD5")]
	MD5,
	#[strum(to_string = "MD5-sess")]
	MD5Sess,
	#[strum(to_string = "SHA-256")]
	SHA256,
	#[strum(to_string = "SHA-256-sess")]
	SHA256Sess,
	#[strum(to_string = "SHA-512")]
	SHA512,
	#[strum(to_string = "SHA-512-sess")]
	SHA512Sess,
}
