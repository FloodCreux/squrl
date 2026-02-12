use anyhow::anyhow;
use std::sync::Arc;

use crate::app::app::App;
use crate::app::collection::CollectionError::{CollectionNameAlreadyExists, CollectionNameIsEmpty};
use crate::app::collection::RequestError::RequestNameIsEmpty;
use crate::cli::args::ARGS;
use crate::models::collection::Collection;
use crate::models::request::Request;
use parking_lot::RwLock;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum CollectionError {
	#[error("The collection name is empty")]
	CollectionNameIsEmpty,
	#[error("A collection with this name already exists")]
	CollectionNameAlreadyExists,
}

#[derive(Error, Debug)]
pub enum RequestError {
	#[error("The request name is empty")]
	RequestNameIsEmpty,
}

impl App<'_> {
	pub fn new_collection(&mut self, new_collection_name: String) -> anyhow::Result<()> {
		let new_collection_name = sanitize_name(new_collection_name);

		if new_collection_name.is_empty() {
			return Err(anyhow!(CollectionNameIsEmpty));
		}

		// Check that collection names are unique (like files)
		for collection in &self.collections {
			if new_collection_name == collection.name {
				return Err(anyhow!(CollectionNameAlreadyExists));
			}
		}

		info!("Collection \"{new_collection_name}\" created");

		let file_format = self.config.get_preferred_collection_file_format();

		let collections_len = self.collections.len();
		let last_position = match collections_len == 0 {
			true => None,
			false => Some(collections_len - 1),
		};

		let new_collection = Collection {
			name: new_collection_name.clone(),
			last_position,
			requests: vec![],
			path: ARGS
				.directory
				.as_ref()
				.unwrap()
				.join(format!("{}.{file_format}", new_collection_name)),
			file_format,
		};

		self.collections.push(new_collection);

		let collection_index = collections_len;

		self.save_collection_to_file(collection_index);

		Ok(())
	}

	pub fn new_request(
		&mut self,
		collection_index: usize,
		mut new_request: Request,
	) -> Result<(), RequestError> {
		new_request.name = sanitize_name(new_request.name);

		if new_request.name.is_empty() {
			return Err(RequestNameIsEmpty);
		}

		info!(
			"Request \"{}\" created in collection \"{}\"",
			new_request.name, &self.collections[collection_index].name
		);

		self.collections[collection_index]
			.requests
			.push(Arc::new(RwLock::new(new_request)));
		self.save_collection_to_file(collection_index);

		Ok(())
	}

	pub fn delete_collection(&mut self, collection_index: usize) {
		info!("Collection deleted");

		let collection = self.collections.remove(collection_index);
		self.delete_collection_file(collection);
	}

	pub fn delete_request(
		&mut self,
		collection_index: usize,
		request_index: usize,
	) -> anyhow::Result<()> {
		info!("Request deleted");

		self.collections[collection_index]
			.requests
			.remove(request_index);
		self.save_collection_to_file(collection_index);

		Ok(())
	}

	pub fn rename_collection(
		&mut self,
		collection_index: usize,
		new_collection_name: String,
	) -> anyhow::Result<()> {
		if new_collection_name.trim().is_empty() {
			return Err(anyhow!(CollectionNameIsEmpty));
		}

		// Check that collection names are unique (like files)
		for collection in &self.collections {
			if new_collection_name == collection.name {
				return Err(anyhow!(CollectionNameAlreadyExists));
			}
		}

		info!("Collection renamed to \"{new_collection_name}\"");

		self.collections[collection_index].name = new_collection_name.to_string();
		self.save_collection_to_file(collection_index);

		Ok(())
	}

	pub fn rename_request(
		&mut self,
		collection_index: usize,
		request_index: usize,
		new_request_name: String,
	) -> anyhow::Result<()> {
		if new_request_name.trim().is_empty() {
			return Err(anyhow!(RequestNameIsEmpty));
		}

		self.with_request_write(collection_index, request_index, |req| {
			info!("Request renamed to \"{new_request_name}\"");
			req.name = new_request_name;
		});

		Ok(())
	}

	pub fn duplicate_collection(&mut self, collection_index: usize) -> anyhow::Result<()> {
		let mut collection = self.collections[collection_index].clone();

		info!("Collection \"{}\" duplicated", collection.name);

		collection.name = format!("{} copy", collection.name);
		collection.path = ARGS
			.directory
			.as_ref()
			.unwrap()
			.join(format!("{}.{}", collection.name, collection.file_format));
		self.collections.insert(collection_index + 1, collection);
		self.save_collection_to_file(collection_index + 1);

		Ok(())
	}

	pub fn duplicate_request(
		&mut self,
		collection_index: usize,
		request_index: usize,
	) -> anyhow::Result<()> {
		let local = self.get_request_as_local_from_indexes(&(collection_index, request_index));
		let mut cloned = local.read().clone();

		info!("Request \"{}\" duplicated", cloned.name);

		cloned.name = format!("{} copy", cloned.name);
		self.collections[collection_index]
			.requests
			.insert(request_index + 1, Arc::new(RwLock::new(cloned)));
		self.save_collection_to_file(collection_index);
		Ok(())
	}

	pub fn update_collections_last_position(&mut self) {
		for index in 0..self.collections.len() {
			let collection = &mut self.collections[index];
			collection.last_position = Some(index);
			self.save_collection_to_file(index);
		}
	}
}

fn sanitize_name(name: String) -> String {
	name.trim().replace("/", "").replace("\"", "")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_sanitize_name_normal_string() {
		assert_eq!(sanitize_name("my-collection".to_string()), "my-collection");
	}

	#[test]
	fn test_sanitize_name_trims_whitespace() {
		assert_eq!(
			sanitize_name("  my-collection  ".to_string()),
			"my-collection"
		);
	}

	#[test]
	fn test_sanitize_name_removes_slashes() {
		assert_eq!(sanitize_name("my/collection".to_string()), "mycollection");
	}

	#[test]
	fn test_sanitize_name_removes_quotes() {
		assert_eq!(sanitize_name("my\"name\"".to_string()), "myname");
	}

	#[test]
	fn test_sanitize_name_removes_all_special() {
		assert_eq!(
			sanitize_name("  my/\"collection\"  ".to_string()),
			"mycollection"
		);
	}

	#[test]
	fn test_sanitize_name_empty_after_sanitization() {
		assert_eq!(sanitize_name("  /\"  ".to_string()), "");
	}
}
