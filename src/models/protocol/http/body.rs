use anyhow::anyhow;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::app::request::http::body::FormError::NotAForm;
use crate::models::protocol::http::body::ContentType::{
	File, Form, Html, Javascript, Json, Multipart, NoBody, Raw, Xml,
};
use crate::models::request::KeyValue;

#[derive(Default, Debug, Clone, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
	#[default]
	#[strum(to_string = "No Body")]
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

impl ContentType {
	pub fn to_content_type(&self) -> String {
		match &self {
			NoBody => String::new(),
			Multipart(_) => String::from("multipart/form-data"),
			Form(_) => String::from("application/x-www-form-urlencoded"),
			Raw(_) => String::from("text/plain"),
			File(_) => String::from("application/octet-stream"),
			Json(_) | Xml(_) | Html(_) | Javascript(_) => {
				format!("application/{}", self.to_string().to_lowercase())
			}
		}
	}

	pub fn from_content_type(content_type: &str, body: String) -> ContentType {
		match content_type {
			//"multipart/form-data" => Multipart(body),
			//"application/x-www-form-urlencoded" => Form(body),
			"application/octet-stream" => File(body),
			"text/plain" => Raw(body),
			"application/json" => Json(body),
			"application/xml" => Json(body),
			"application/html" => Json(body),
			"application/javascript" => Json(body),
			_ => NoBody,
		}
	}

	pub fn get_form(&self) -> anyhow::Result<&Vec<KeyValue>> {
		match self {
			Multipart(form) | Form(form) => Ok(form),
			_ => Err(anyhow!(NotAForm)),
		}
	}

	pub fn get_form_mut(&mut self) -> anyhow::Result<&mut Vec<KeyValue>> {
		match self {
			Multipart(form) | Form(form) => Ok(form),
			_ => Err(anyhow!(NotAForm)),
		}
	}
}

pub fn next_content_type(content_type: &ContentType) -> ContentType {
	match content_type {
		NoBody => Multipart(Vec::new()),
		Multipart(_) => Form(Vec::new()),
		Form(_) => File(String::new()),
		File(_) => Raw(String::new()),
		Raw(body) => Json(body.to_string()),
		Json(body) => Xml(body.to_string()),
		Xml(body) => Html(body.to_string()),
		Html(body) => Javascript(body.to_string()),
		Javascript(_) => NoBody,
	}
}

/// Iter through the headers and tries to catch a file format like `application/<file_format>`
pub fn find_file_format_in_content_type(headers: &Vec<(String, String)>) -> Option<String> {
	if let Some((_, content_type)) = headers
		.par_iter()
		.find_any(|(header, _)| *header == "content-type")
	{
		// Regex that likely catches the file format
		let regex = Regex::new(r"\w+/(?<file_format>\w+)").unwrap();

		return match regex.captures(content_type) {
			// No file format found
			None => None,
			// File format found
			Some(capture) => Some(capture["file_format"].to_string()),
		};
	} else {
		return None;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn make_headers(pairs: Vec<(&str, &str)>) -> Vec<(String, String)> {
		pairs
			.into_iter()
			.map(|(k, v)| (k.to_string(), v.to_string()))
			.collect()
	}

	#[test]
	fn detects_json_content_type() {
		let headers = make_headers(vec![("content-type", "application/json")]);
		assert_eq!(
			find_file_format_in_content_type(&headers),
			Some("json".to_string())
		);
	}

	#[test]
	fn detects_xml_content_type() {
		let headers = make_headers(vec![("content-type", "application/xml")]);
		assert_eq!(
			find_file_format_in_content_type(&headers),
			Some("xml".to_string())
		);
	}

	#[test]
	fn detects_png_content_type() {
		let headers = make_headers(vec![("content-type", "image/png")]);
		assert_eq!(
			find_file_format_in_content_type(&headers),
			Some("png".to_string())
		);
	}

	#[test]
	fn detects_html_content_type() {
		let headers = make_headers(vec![("content-type", "text/html")]);
		assert_eq!(
			find_file_format_in_content_type(&headers),
			Some("html".to_string())
		);
	}

	#[test]
	fn detects_plain_text_content_type() {
		let headers = make_headers(vec![("content-type", "text/plain")]);
		assert_eq!(
			find_file_format_in_content_type(&headers),
			Some("plain".to_string())
		);
	}

	#[test]
	fn returns_none_when_no_content_type_header() {
		let headers = make_headers(vec![("accept", "application/json")]);
		assert_eq!(find_file_format_in_content_type(&headers), None);
	}

	#[test]
	fn returns_none_for_empty_headers() {
		let headers: Vec<(String, String)> = vec![];
		assert_eq!(find_file_format_in_content_type(&headers), None);
	}

	#[test]
	fn finds_content_type_among_multiple_headers() {
		let headers = make_headers(vec![
			("accept", "text/html"),
			("content-type", "application/json"),
			("authorization", "Bearer token"),
		]);
		assert_eq!(
			find_file_format_in_content_type(&headers),
			Some("json".to_string())
		);
	}
}
