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

	pub algorithm: DigestAlgorithm,
	pub qop: DigestQop,
	pub user_hash: bool,
	pub charset: DigestCharset,

	#[serde(skip, default)]
	pub nc: u32,
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

#[derive(Debug, Default, Clone, ValueEnum, Display, Serialize, Deserialize)]
pub enum DigestQop {
	#[default]
	#[strum(to_string = "None")]
	None,
	#[strum(to_string = "auth")]
	Auth,
	#[strum(to_string = "auth-int")]
	AuthInt,
}

#[derive(Debug, Default, Clone, ValueEnum, Display, Serialize, Deserialize)]
pub enum DigestCharset {
	#[default]
	ASCII,
	UTF8,
}

pub fn extract_www_authenticate_digest_data(
	www_authenticate_header: &str,
) -> Result<
	(
		String,
		String,
		String,
		String,
		bool,
		DigestAlgorithm,
		DigestQop,
		bool,
		DigestCharset,
	),
	DigestError,
> {
	let mut prompt_kv = parse_header_map(www_authenticate_header)?;
	let domains = prompt_kv.remove("domain").unwrap_or_default();
	let realm = match prompt_kv.remove("realm") {
		Some(v) => v,
		None => return Err(MissingRequired("realm")),
	};
	let nonce = match prompt_kv.remove("nonce") {
		Some(v) => v,
		None => return Err(MissingRequired("nonce")),
	};
	let opaque = prompt_kv.remove("opaque").unwrap_or_default();
	let stale = match prompt_kv.get("stale") {
		Some(v) => match v.to_ascii_lowercase().as_str() {
			"true" => true,
			"false" => false,
			_ => return Err(InvalidBooleanValue("stale", v.to_string())),
		},
		None => false,
	};
	let algorithm = match prompt_kv.get("algorithm") {
		Some(a) => match digest_auth::Algorithm::from_str(a.as_str()) {
			Ok(algorithm) => DigestAlgorithm::from_digest_auth_algorithm(algorithm),
			Err(_) => return Err(InvalidAlgorithm(a.to_string())),
		},
		_ => DigestAlgorithm::default(),
	};
	let qop = match prompt_kv.get("qop") {
		Some(domains) => {
			let domains: Vec<&str> = domains.split(',').collect();

			if domains.is_empty() {
				DigestQop::None
			} else if domains.contains(&"auth-int") {
				DigestQop::AuthInt
			} else if domains.contains(&"auth") {
				DigestQop::Auth
			} else {
				return Err(MissingRequired("QOP"));
			}
		}
		None => DigestQop::None,
	};
	let user_hash = match prompt_kv.get("userhash") {
		Some(v) => match v.to_ascii_lowercase().as_str() {
			"true" => true,
			"false" => false,
			_ => return Err(InvalidBooleanValue("userhash", v.to_string())),
		},
		None => false,
	};
	let charset = match prompt_kv.get("charset") {
		Some(v) => match digest_auth::Charset::from_str(v) {
			Ok(charset) => DigestCharset::from_digest_charset(charset),
			Err(_) => return Err(InvalidCharset(v.to_string())),
		},
		None => DigestCharset::ASCII,
	};

	Ok((
		domains, realm, nonce, opaque, stale, algorithm, qop, user_hash, charset,
	))
}
