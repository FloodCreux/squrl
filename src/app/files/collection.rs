use std::collections::BTreeMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, anyhow};
use parking_lot::RwLock;
use tracing::{info, trace, warn};

use reqwest::Url;

use crate::app::App;
use crate::app::files::utils::write_via_temp_file;
use crate::cli::args::ARGS;
use crate::models::auth::auth::Auth;
use crate::models::collection::CollectionFileFormat::{Http, Json, Yaml};
use crate::models::collection::{Collection, CollectionFileFormat};
use crate::models::protocol::http::body::ContentType;
use crate::models::protocol::protocol::Protocol;
use crate::models::request::{KeyValue, Request};

impl App<'_> {
	/// Set the app request to the requests found in the collection file
	pub fn set_collections_from_file(
		&mut self,
		path_buf: PathBuf,
		file_format: CollectionFileFormat,
	) -> anyhow::Result<()> {
		let mut file_content = String::new();

		trace!("Trying to open \"{}\" collection", path_buf.display());

		let mut collection_file = OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.truncate(false)
			.open(path_buf.clone())
			.with_context(|| {
				format!("Could not open collection file \"{}\"", path_buf.display())
			})?;

		collection_file
			.read_to_string(&mut file_content)
			.with_context(|| {
				format!("Could not read collection file \"{}\"", path_buf.display())
			})?;

		let mut collection: Collection = match file_format {
			Json => serde_json::from_str(&file_content).with_context(|| {
				format!("Could not parse JSON collection \"{}\"", path_buf.display())
			})?,
			Yaml => serde_yaml_ng::from_str(&file_content).with_context(|| {
				format!("Could not parse YAML collection \"{}\"", path_buf.display())
			})?,
			Http => unreachable!(),
		};

		collection.path = path_buf;
		collection.file_format = file_format;

		self.core.collections.push(collection);

		trace!("Collection file parsed!");
		Ok(())
	}

	/// Save app collection in the collection file through a temporary file.
	/// Logs a warning on failure rather than panicking, since saves happen
	/// frequently from TUI event handlers where error propagation is impractical.
	pub fn save_collection_to_file(&mut self, collection_index: usize) {
		if !ARGS.should_save {
			warn!("Dry-run, not saving the collection");
			return;
		}

		if let Err(e) = self.save_collection_to_file_inner(collection_index) {
			warn!("Failed to save collection: {e:#}");
		}
	}

	fn save_collection_to_file_inner(&mut self, collection_index: usize) -> anyhow::Result<()> {
		if matches!(self.core.collections[collection_index].file_format, Http) {
			return self.save_collection_as_http_files(collection_index);
		}

		// Auto-assign a file path for ephemeral collections on first save
		if self.core.collections[collection_index]
			.path
			.as_os_str()
			.is_empty()
		{
			let file_format = self.core.config.get_preferred_collection_file_format();
			let name = self.core.collections[collection_index].name.clone();
			let path = ARGS
				.directory
				.as_ref()
				.ok_or_else(|| anyhow!("--directory argument is required"))?
				.join(format!("{name}.{file_format}"));

			info!(
				"Ephemeral collection \"{name}\" will now be saved to \"{}\"",
				path.display()
			);

			self.core.collections[collection_index].path = path;
			self.core.collections[collection_index].file_format = file_format;
		}

		let collection = &self.core.collections[collection_index];

		info!("Saving collection \"{}\"", collection.name);

		let collection_stringed = match collection.file_format {
			Json => serde_json::to_string_pretty(collection)
				.context("Could not serialize collection to JSON")?,
			Yaml => serde_yaml_ng::to_string(collection)
				.context("Could not serialize collection to YAML")?,
			Http => unreachable!(),
		};

		write_via_temp_file(&collection.path, collection_stringed.as_bytes())
			.context("Could not save collection file")?;

		trace!("Collection saved");
		Ok(())
	}

	fn save_collection_as_http_files(&mut self, collection_index: usize) -> anyhow::Result<()> {
		let collection = &self.core.collections[collection_index];
		let base_dir = &collection.path;

		// --- Phase 1: Collect all requests and group by source file ---

		// Build a map: source_file_path -> Vec<&Request>
		// We need to gather both root requests and folder requests
		let mut file_groups: BTreeMap<PathBuf, Vec<Arc<RwLock<Request>>>> = BTreeMap::new();

		for req_lock in &collection.requests {
			self.generate_file_group(req_lock, base_dir, None, &mut file_groups);
		}

		for folder in &collection.folders {
			for req_lock in &folder.requests {
				self.generate_file_group(
					req_lock,
					base_dir,
					Some(folder.name.clone()),
					&mut file_groups,
				);
			}
		}

		// --- Phase 2: Serialize and write each file ---

		for (file_path, requests) in &file_groups {
			if let Some(parent) = file_path.parent() {
				fs::create_dir_all(parent)?;
			}

			let content = self.serialize_requests_to_http(requests);

			write_via_temp_file(file_path, content.as_bytes())?;
		}

		Ok(())
	}

	fn generate_file_group(
		&self,
		req_lock: &Arc<RwLock<Request>>,
		base_dir: &Path,
		folder_name: Option<String>,
		file_groups: &mut BTreeMap<PathBuf, Vec<Arc<RwLock<Request>>>>,
	) {
		let req = req_lock.read();
		let file_path = match &req.source_path {
			Some(path) => path.clone(),
			None => {
				let file_name = &req.name;
				match folder_name {
					Some(f_name) => base_dir.join(f_name).join(format!("{file_name}.http")),
					None => base_dir.join(format!("{file_name}.http")),
				}
			}
		};

		file_groups
			.entry(file_path)
			.or_default()
			.push(req_lock.clone());
	}

	fn serialize_requests_to_http(&self, requests: &Vec<Arc<RwLock<Request>>>) -> String {
		let mut results: Vec<String> = vec![];

		for req_lock in requests {
			let req = req_lock.read();
			let mut lines: Vec<String> = vec![];

			// --- Request separator with name ---
			lines.push(format!("### {}", req.name));

			// --- Build the full URL with query params ---
			let full_url = Self::build_url_with_params(&req.url, &req.params);

			// --- Request line ---
			match &req.protocol {
				Protocol::HttpRequest(http) => {
					lines.push(format!(
						"{} {}",
						http.method.to_string().to_uppercase(),
						full_url,
					));
				}
				Protocol::WsRequest(_) => {
					lines.push(format!("WEBSOCKET {}", full_url));
				}
				Protocol::GraphqlRequest(_) => {
					lines.push(format!("GRAPHQL {}", full_url));
				}
				Protocol::GrpcRequest(_) => {
					lines.push(format!("GRPC {}", full_url));
				}
			}

			// --- Authorization header from auth ---
			match &req.auth {
				Auth::BearerToken(bearer) => {
					if !bearer.token.is_empty() {
						lines.push(format!("Authorization: Bearer {}", bearer.token));
					}
				}
				Auth::BasicAuth(basic) => {
					use base64::Engine;
					let encoded = base64::engine::general_purpose::STANDARD
						.encode(format!("{}:{}", basic.username, basic.password));
					lines.push(format!("Authorization: Basic {}", encoded));
				}
				_ => {}
			}

			// --- Content-Type header (for HTTP requests with a body) ---
			// We emit this before user headers so that if the user has an explicit
			// Content-Type header it will appear after and naturally override when
			// parsed back.  However, we skip emitting it if the user already has one
			// in their headers list to avoid duplication.
			let user_has_content_type = req
				.headers
				.iter()
				.any(|h| h.enabled && h.data.0.to_lowercase() == "content-type");

			let body_content = match &req.protocol {
				Protocol::HttpRequest(http) => {
					if !user_has_content_type {
						let ct = http.body.to_content_type();
						if !ct.is_empty() {
							lines.push(format!("Content-Type: {}", ct));
						}
					}
					Self::serialize_body(&http.body)
				}
				Protocol::WsRequest(_) => None,
				Protocol::GraphqlRequest(gql) => {
					if !user_has_content_type {
						lines.push("Content-Type: application/json".to_string());
					}
					let mut body = serde_json::json!({ "query": gql.query });
					if !gql.variables.is_empty()
						&& let Ok(vars) = serde_json::from_str::<serde_json::Value>(&gql.variables)
					{
						body["variables"] = vars;
					}
					if let Some(op) = &gql.operation_name
						&& !op.is_empty()
					{
						body["operationName"] = serde_json::Value::String(op.clone());
					}
					Some(serde_json::to_string_pretty(&body).unwrap_or_default())
				}
				Protocol::GrpcRequest(grpc) => {
					if !grpc.proto_file.is_empty() {
						lines.push(format!("X-Proto-File: {}", grpc.proto_file));
					}
					if !grpc.service.is_empty() {
						lines.push(format!("X-Grpc-Service: {}", grpc.service));
					}
					if !grpc.method.is_empty() {
						lines.push(format!("X-Grpc-Method: {}", grpc.method));
					}
					if !user_has_content_type {
						lines.push("Content-Type: application/grpc+json".to_string());
					}
					if grpc.message.is_empty() {
						None
					} else {
						Some(grpc.message.clone())
					}
				}
			};

			// --- User headers ---
			for header in &req.headers {
				if header.enabled {
					lines.push(format!("{}: {}", header.data.0, header.data.1));
				}
			}

			// --- Body ---
			if let Some(body) = body_content
				&& !body.is_empty()
			{
				lines.push(String::new()); // blank line separating headers from body
				lines.push(body);
			}

			results.push(lines.join("\n"));
		}

		results.join("\n\n")
	}

	/// Reconstruct the full URL including enabled query parameters.
	fn build_url_with_params(base_url: &str, params: &[KeyValue]) -> String {
		let enabled_params: Vec<_> = params.iter().filter(|p| p.enabled).collect();

		if enabled_params.is_empty() {
			return base_url.to_string();
		}

		// Use Url for proper encoding (mirrors how the parser deconstructs URLs)
		if let Ok(mut url) = Url::parse(base_url) {
			{
				let mut query_pairs = url.query_pairs_mut();
				for p in &enabled_params {
					query_pairs.append_pair(&p.data.0, &p.data.1);
				}
			}
			url.to_string()
		} else {
			// Fallback for non-parseable URLs: append manually
			let query_string: String = enabled_params
				.iter()
				.map(|p| format!("{}={}", &p.data.0, &p.data.1))
				.collect::<Vec<_>>()
				.join("&");

			if base_url.contains('?') {
				format!("{}&{}", base_url, query_string)
			} else {
				format!("{}?{}", base_url, query_string)
			}
		}
	}

	/// Serialize an HTTP body `ContentType` into its string representation for an `.http` file.
	/// Returns `None` for `NoBody`.
	fn serialize_body(body: &ContentType) -> Option<String> {
		match body {
			ContentType::NoBody => None,
			ContentType::File(path) => {
				if path.is_empty() {
					None
				} else {
					Some(format!("< {}", path))
				}
			}
			ContentType::Multipart(fields) | ContentType::Form(fields) => {
				// Use a dummy Url to leverage its query-pair encoder, then
				// extract just the query string.  This avoids pulling in a
				// separate url-encoding crate.
				let mut dummy = Url::parse("http://x").expect("static URL should always parse");
				{
					let mut pairs = dummy.query_pairs_mut();
					for kv in fields.iter().filter(|kv| kv.enabled) {
						pairs.append_pair(&kv.data.0, &kv.data.1);
					}
				}
				dummy.query().map(|q| q.to_string())
			}
			ContentType::Raw(s)
			| ContentType::Json(s)
			| ContentType::Xml(s)
			| ContentType::Html(s)
			| ContentType::Javascript(s) => {
				if s.is_empty() {
					None
				} else {
					Some(s.clone())
				}
			}
		}
	}

	/// Delete collection file.
	/// Logs a warning on failure rather than panicking.
	pub fn delete_collection_file(&mut self, collection: Collection) {
		if !ARGS.should_save {
			return;
		}

		if let Err(e) = fs::remove_file(&collection.path) {
			warn!(
				"Could not delete collection file \"{}\": {e}",
				collection.path.display()
			);
		}
	}
}
