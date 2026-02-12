use std::fs;
use std::sync::Arc;

use anyhow::anyhow;
use indexmap::IndexMap;
use openapiv3::{OpenAPI, ReferenceOr};
use parking_lot::RwLock;
use rayon::prelude::*;
use reqwest::Url;

use crate::app::app::App;
use crate::cli::args::ARGS;
use crate::cli::commands::import::CurlImport;
use crate::cli::commands::import::HttpFileImport;
use crate::cli::commands::import::OpenApiImport;
use crate::cli::commands::import::PostmanEnvImport;
use crate::cli::commands::import::PostmanImport;
use crate::cli::import::curl;
use crate::cli::import::http_file;
use crate::cli::import::openapi;
use crate::cli::import::openapi::ImportOpenApiError;
use crate::cli::import::postman_collection;
use crate::cli::import::postman_collection::ImportPostmanError::{
	CollectionAlreadyExists, CouldNotParseCollection,
};
use crate::cli::import::postman_env::{ImportPostmanEnvironmentError, PostmanEnv};
use crate::models::collection::Collection;
use crate::models::environment::Environment;

impl App<'_> {
	pub fn import_postman_collection(
		&mut self,
		postman_import: &PostmanImport,
	) -> anyhow::Result<()> {
		let path_buf = &postman_import.import_path;
		let max_depth = postman_import.max_depth.unwrap_or(99);

		println!("Parsing Postman collection");

		let mut postman_collection = match parse_postman_collection::from_path(path_buf) {
			Ok(postman_collection) => postman_collection,
			Err(e) => {
				return Err(anyhow!(CouldNotParseCollection(
					path_buf.display().to_string(),
					e.to_string()
				)));
			}
		};

		let collection_name = postman_collection.info.name.clone();

		println!("Collection name: {}", collection_name);

		for existing_collection in &self.collections {
			if existing_collection.name == collection_name {
				return Err(anyhow!(CollectionAlreadyExists(collection_name)));
			}
		}

		let file_format = self.config.get_preferred_collection_file_format();

		let mut collections: Vec<Collection> = vec![Collection {
			name: collection_name.clone(),
			last_position: Some(self.collections.len() - 1),
			requests: vec![],
			path: ARGS
				.directory
				.as_ref()
				.unwrap()
				.join(format!("{}.{}", collection_name, file_format)),
			file_format,
		}];

		let mut depth_level: u16 = 0;

		if max_depth == 0 {
			for item in postman_collection.item.iter_mut() {
				collections[0]
					.requests
					.extend(postman_collection::recursive_get_requests(item)?);
			}
		} else {
			for mut item in postman_collection.item {
				if item.name.is_none() {
					continue;
				}

				// If this is a folder
				if postman_collection::is_folder(&item) {
					let mut temp_nesting_prefix = String::new();
					let new_collections: Vec<Collection> = vec![];

					let file_format = self.config.get_preferred_collection_file_format();

					postman_collection::recursive_has_requests(
						&mut item,
						&mut collections,
						&mut temp_nesting_prefix,
						&mut depth_level,
						max_depth,
						file_format,
					)?;

					collections.extend(new_collections);
				} else {
					collections[0].requests.push(Arc::new(RwLock::new(
						postman_collection::parse_request(item)?,
					)));
				}
			}
		}

		// Prevent from having an empty collection
		if collections.len() > 1 && collections[0].requests.is_empty() {
			collections.remove(0);
		}

		let collections_length = collections.len();

		let start_index = self.collections.len();
		self.collections.extend(collections);

		for collection_index in start_index..start_index + collections_length {
			self.save_collection_to_file(collection_index);
		}

		Ok(())
	}

	pub fn import_postman_environment(
		&mut self,
		postman_env_import: &PostmanEnvImport,
	) -> anyhow::Result<()> {
		let path_buf = &postman_env_import.import_path;

		println!("Parsing Postman environment");

		// Read the file content
		let file_content = match fs::read_to_string(path_buf) {
			Ok(content) => content,
			Err(e) => {
				return Err(anyhow!(ImportPostmanEnvironmentError::CouldNotReadFile(
					e.to_string()
				)));
			}
		};

		let postman_environment = match serde_yaml::from_str::<PostmanEnv>(&file_content) {
			Ok(postman_environment) => postman_environment,
			Err(e) => {
				return Err(anyhow!(
					ImportPostmanEnvironmentError::CouldNotParsePostmanEnvironment(e.to_string())
				));
			}
		};

		println!("Postman environment name: {}", postman_environment.name);

		let filename = format!(
			".env.{}",
			postman_environment.name.to_lowercase().replace(" ", "_")
		);
		let path = ARGS.directory.as_ref().unwrap().join(filename);

		let mut env = Environment {
			name: postman_environment.name,
			values: IndexMap::new(),
			path,
		};

		for env_variable in postman_environment.values {
			if !postman_env_import.use_disabled && !env_variable.enabled {
				continue;
			}

			let key = match postman_env_import.force_uppercase_keys {
				true => env_variable.key.to_uppercase(),
				false => env_variable.key.clone(),
			};

			env.values.insert(key, env_variable.value.clone());
		}

		let env_count = self.environments.len();
		self.environments.push(Arc::new(RwLock::new(env)));

		self.save_environment_to_file(env_count);

		Ok(())
	}

	pub fn import_openapi_collection(
		&mut self,
		openapi_import: &OpenApiImport,
	) -> anyhow::Result<()> {
		let path_buf = &openapi_import.import_path;

		println!("Parsing OpenAPI specification");

		// Read the file content
		let spec_content = match fs::read_to_string(path_buf) {
			Ok(content) => content,
			Err(e) => {
				return Err(anyhow!(ImportOpenApiError::CouldNotReadFile(e.to_string())));
			}
		};

		// Parse based on file extension
		let spec: OpenAPI = if path_buf.extension().is_some_and(|ext| ext == "json") {
			match serde_json::from_str(&spec_content) {
				Ok(spec) => spec,
				Err(e) => {
					return Err(anyhow!(ImportOpenApiError::CouldNotParseSpec(
						path_buf.display().to_string(),
						e.to_string()
					)));
				}
			}
		} else {
			// Assume YAML if not JSON
			match serde_yaml::from_str(&spec_content) {
				Ok(spec) => spec,
				Err(e) => {
					return Err(anyhow!(ImportOpenApiError::CouldNotParseSpec(
						path_buf.display().to_string(),
						e.to_string()
					)));
				}
			}
		};

		// Determine collection name
		let collection_name = spec.info.title.clone();

		println!("Collection name: {}", collection_name);

		// Check if collection already exists
		for existing_collection in &self.collections {
			if existing_collection.name == collection_name {
				return Err(anyhow!(ImportOpenApiError::CollectionAlreadyExists(
					collection_name
				)));
			}
		}

		let file_format = self.config.get_preferred_collection_file_format();

		// Create a new collection
		let mut collection = Collection {
			name: collection_name.clone(),
			last_position: Some(self.collections.len() - 1),
			requests: Vec::new(),
			path: ARGS
				.directory
				.as_ref()
				.unwrap()
				.join(format!("{}.{}", collection_name, file_format)),
			file_format,
		};

		// Parse and add all requests from paths
		let base_url = match spec.servers.first() {
			Some(server) => match Url::parse(server.url.clone().as_str()) {
				Ok(url) => url.to_string(),
				Err(error) => {
					return Err(anyhow!(ImportOpenApiError::InvalidUrl(error.to_string())));
				}
			},
			None => String::from("https://example.com"),
		};

		// Process all paths and operations
		for (path, path_item) in spec.paths.iter().by_ref() {
			match path_item {
				ReferenceOr::Reference { reference: _ } => {
					// Handle references - would need to resolve them
					println!("\tSkipping reference for path: {}", path);
				}
				ReferenceOr::Item(path_item) => {
					// Process each HTTP method in this path
					openapi::process_path_operations(
						&mut collection,
						path_item,
						path,
						&base_url,
						&spec,
					)?;
				}
			}
		}

		println!(
			"\tFound {} requests in OpenAPI spec",
			collection.requests.len()
		);

		// Add the collection to app's collections
		self.collections.push(collection);

		// Save the collection to file
		self.save_collection_to_file(self.collections.len() - 1);

		Ok(())
	}

	pub fn import_http_file(&mut self, http_file_import: &HttpFileImport) -> anyhow::Result<()> {
		let path_buf = &http_file_import.import_path;
		let recursive = &http_file_import.recursive;
		let max_depth = http_file_import.max_depth.unwrap_or(99);

		println!("Parsing .http file(s)");

		// Derive collection name from argument or from path
		let collection_name = match &http_file_import.collection_name {
			Some(name) => name.clone(),
			None => path_buf
				.file_stem()
				.unwrap_or_default()
				.to_str()
				.unwrap_or("http-import")
				.to_string(),
		};

		println!("Collection name: {}", collection_name);

		let (collection_index, collection) = match self
			.collections
			.par_iter_mut()
			.enumerate()
			.find_any(|(_, collection)| collection.name == collection_name.as_str())
		{
			Some((index, collection)) => (index, collection),
			None => {
				println!("Collection does not exist. Creating it...");

				let file_format = self.config.get_preferred_collection_file_format();

				let collection = Collection {
					name: collection_name.clone(),
					last_position: Some(self.collections.len() - 1),
					requests: vec![],
					path: ARGS
						.directory
						.as_ref()
						.unwrap()
						.join(format!("{}.{}", collection_name, file_format)),
					file_format,
				};

				self.collections.push(collection);

				(
					self.collections.len() - 1,
					self.collections.last_mut().unwrap(),
				)
			}
		};

		let requests = if path_buf.is_file() {
			http_file::parse_http_file(path_buf)?
		} else {
			http_file::parse_http_files_recursively(path_buf, *recursive, max_depth)?
		};

		collection.requests.extend(requests);

		self.save_collection_to_file(collection_index);

		Ok(())
	}

	pub fn import_curl_file(&mut self, curl_import: &CurlImport) -> anyhow::Result<()> {
		let path_buf = &curl_import.import_path;
		let collection_name = &curl_import.collection_name;
		let request_name = &curl_import.request_name;
		let recursive = &curl_import.recursive;
		let max_depth = curl_import.max_depth.unwrap_or(99);

		println!("Parsing cURL request");

		println!("Collection name: {}", collection_name);

		let (collection_index, collection) = match self
			.collections
			.par_iter_mut()
			.enumerate()
			.find_any(|(_, collection)| collection.name == collection_name.as_str())
		{
			Some((index, collection)) => (index, collection),
			None => {
				println!("Collection does not exist. Creating it...");

				let file_format = self.config.get_preferred_collection_file_format();

				let collection = Collection {
					name: collection_name.clone(),
					last_position: Some(self.collections.len() - 1),
					requests: vec![],
					path: ARGS.directory.as_ref().unwrap().join(format!(
						"{}.{}",
						collection_name.clone(),
						file_format
					)),
					file_format,
				};

				self.collections.push(collection);

				(
					self.collections.len() - 1,
					self.collections.last_mut().unwrap(),
				)
			}
		};

		let request_name = match request_name {
			None => path_buf.file_stem().unwrap().to_str().unwrap().to_string(),
			Some(request_name) => request_name.clone(),
		};

		let requests = match path_buf.is_file() {
			true => vec![curl::parse_request(path_buf, request_name)?],
			false => curl::parse_requests_recursively(path_buf, *recursive, max_depth)?,
		};

		// Add the parsed request to the collection
		collection.requests.extend(requests);

		self.save_collection_to_file(collection_index);

		Ok(())
	}
}
