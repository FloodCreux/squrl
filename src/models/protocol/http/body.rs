use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::models::request::KeyValue;

#[derive(Default, Debug, Clone, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
	#[default]
	#[strum(to_string = "No body")]
	NoBody,

	#[strum(to_string = "File")]
	File(String),

	#[strum(to_string = "Multipart")]
	Multipart(Vec<KeyValue>),

	#[strum(to_string = "Form")]
	Form(Vec<KeyValue>),

	#[strum(to_string = "Text")]
	Raw(String),

	#[strum(to_string = "JSON")]
	Json(String),

	#[strum(to_string = "XML")]
	Xml(String),

	#[strum(to_string = "HTML")]
	Html(String),

	#[strum(to_string = "Javascript")]
	Javascript(String),
}

pub fn find_file_format_in_content_type(headers: &Vec<(String, String)>) -> Option<String> {
	if let Some((_, content_type)) = headers
		.par_iter()
		.find_any(|(header, _)| *header == "content-type")
	{
		let regex = Regex::new(r"\w+/(?<file_format>\w+)").unwrap();

		regex
			.captures(content_type)
			.map(|capture| capture["file_format"].to_string())
	} else {
		None
	}
}
