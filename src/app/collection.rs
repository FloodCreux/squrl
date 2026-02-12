use anyhow::anyhow;
use std::sync::Arc;

use crate::app::app::App;
use crate::app::collection::CollectionError::{CollectionNameAlreadyExists, CollectionNameIsEmpty};
use crate::app::collection::FolderError::{FolderNameAlreadyExists, FolderNameIsEmpty};
use crate::app::collection::RequestError::RequestNameIsEmpty;
use crate::cli::args::ARGS;
use crate::models::collection::Collection;
use crate::models::folder::Folder;
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

#[derive(Error, Debug)]
pub enum FolderError {
	#[error("The folder name is empty")]
	FolderNameIsEmpty,
	#[error("A folder with this name already exists in this collection")]
	FolderNameAlreadyExists,
}

impl App<'_> {
	pub fn new_collection(&mut self, new_collection_name: String) -> anyhow::Result<()> {
		let new_collection_name = sanitize_name(new_collection_name);

		if new_collection_name.is_empty() {
			return Err(anyhow!(CollectionNameIsEmpty));
		}

		// Check that collection names are unique (like files)
		for collection in &self.core.collections {
			if new_collection_name == collection.name {
				return Err(anyhow!(CollectionNameAlreadyExists));
			}
		}

		info!("Collection \"{new_collection_name}\" created");

		let file_format = self.core.config.get_preferred_collection_file_format();

		let collections_len = self.core.collections.len();
		let last_position = match collections_len == 0 {
			true => None,
			false => Some(collections_len - 1),
		};

		let new_collection = Collection {
			name: new_collection_name.clone(),
			last_position,
			folders: vec![],
			requests: vec![],
			path: ARGS
				.directory
				.as_ref()
				.unwrap()
				.join(format!("{}.{file_format}", new_collection_name)),
			file_format,
		};

		self.core.collections.push(new_collection);

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
			new_request.name, &self.core.collections[collection_index].name
		);

		self.core.collections[collection_index]
			.requests
			.push(Arc::new(RwLock::new(new_request)));
		self.save_collection_to_file(collection_index);

		Ok(())
	}

	pub fn new_request_in_folder(
		&mut self,
		collection_index: usize,
		folder_index: usize,
		mut new_request: Request,
	) -> Result<(), RequestError> {
		new_request.name = sanitize_name(new_request.name);

		if new_request.name.is_empty() {
			return Err(RequestNameIsEmpty);
		}

		let collection = &mut self.core.collections[collection_index];
		let folder = &mut collection.folders[folder_index];

		info!(
			"Request \"{}\" created in folder \"{}\" of collection \"{}\"",
			new_request.name, folder.name, collection.name
		);

		folder.requests.push(Arc::new(RwLock::new(new_request)));
		self.save_collection_to_file(collection_index);

		Ok(())
	}

	pub fn delete_collection(&mut self, collection_index: usize) {
		info!("Collection deleted");

		let collection = self.core.collections.remove(collection_index);
		self.delete_collection_file(collection);
	}

	pub fn delete_request(
		&mut self,
		collection_index: usize,
		request_index: usize,
	) -> anyhow::Result<()> {
		info!("Request deleted");

		self.core.collections[collection_index]
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
		for collection in &self.core.collections {
			if new_collection_name == collection.name {
				return Err(anyhow!(CollectionNameAlreadyExists));
			}
		}

		info!("Collection renamed to \"{new_collection_name}\"");

		self.core.collections[collection_index].name = new_collection_name.to_string();
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
		let mut collection = self.core.collections[collection_index].clone();

		info!("Collection \"{}\" duplicated", collection.name);

		collection.name = format!("{} copy", collection.name);
		collection.path = ARGS
			.directory
			.as_ref()
			.unwrap()
			.join(format!("{}.{}", collection.name, collection.file_format));
		self.core
			.collections
			.insert(collection_index + 1, collection);
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
		self.core.collections[collection_index]
			.requests
			.insert(request_index + 1, Arc::new(RwLock::new(cloned)));
		self.save_collection_to_file(collection_index);
		Ok(())
	}

	pub fn update_collections_last_position(&mut self) {
		for index in 0..self.core.collections.len() {
			let collection = &mut self.core.collections[index];
			collection.last_position = Some(index);
			self.save_collection_to_file(index);
		}
	}

	// ── Folder operations ───────────────────────────────────────────────

	pub fn new_folder(
		&mut self,
		collection_index: usize,
		new_folder_name: String,
	) -> anyhow::Result<()> {
		let new_folder_name = sanitize_name(new_folder_name);

		if new_folder_name.is_empty() {
			return Err(anyhow!(FolderNameIsEmpty));
		}

		// Check for duplicate folder names within the collection
		for folder in &self.core.collections[collection_index].folders {
			if new_folder_name == folder.name {
				return Err(anyhow!(FolderNameAlreadyExists));
			}
		}

		info!(
			"Folder \"{}\" created in collection \"{}\"",
			new_folder_name, &self.core.collections[collection_index].name
		);

		let new_folder = Folder {
			name: new_folder_name,
			requests: vec![],
		};

		self.core.collections[collection_index]
			.folders
			.push(new_folder);
		self.save_collection_to_file(collection_index);

		Ok(())
	}

	/// Delete a folder and move its requests to the collection root.
	pub fn delete_folder(&mut self, collection_index: usize, folder_index: usize) {
		info!("Folder deleted (requests moved to collection root)");

		let folder = self.core.collections[collection_index]
			.folders
			.remove(folder_index);

		// Move all requests from the folder to the collection's root-level requests
		self.core.collections[collection_index]
			.requests
			.extend(folder.requests);

		self.save_collection_to_file(collection_index);
	}

	/// Delete a request inside a folder.
	pub fn delete_folder_request(
		&mut self,
		collection_index: usize,
		folder_index: usize,
		request_index: usize,
	) -> anyhow::Result<()> {
		info!("Request deleted from folder");

		self.core.collections[collection_index].folders[folder_index]
			.requests
			.remove(request_index);
		self.save_collection_to_file(collection_index);

		Ok(())
	}

	pub fn rename_folder(
		&mut self,
		collection_index: usize,
		folder_index: usize,
		new_folder_name: String,
	) -> anyhow::Result<()> {
		let new_folder_name = sanitize_name(new_folder_name);

		if new_folder_name.trim().is_empty() {
			return Err(anyhow!(FolderNameIsEmpty));
		}

		// Check for duplicate folder names
		for folder in &self.core.collections[collection_index].folders {
			if new_folder_name == folder.name {
				return Err(anyhow!(FolderNameAlreadyExists));
			}
		}

		info!("Folder renamed to \"{new_folder_name}\"");

		self.core.collections[collection_index].folders[folder_index].name = new_folder_name;
		self.save_collection_to_file(collection_index);

		Ok(())
	}

	/// Rename a request inside a folder.
	pub fn rename_folder_request(
		&mut self,
		collection_index: usize,
		folder_index: usize,
		request_index: usize,
		new_request_name: String,
	) -> anyhow::Result<()> {
		if new_request_name.trim().is_empty() {
			return Err(anyhow!(RequestNameIsEmpty));
		}

		info!("Request in folder renamed to \"{new_request_name}\"");

		self.core.collections[collection_index].folders[folder_index].requests[request_index]
			.write()
			.name = new_request_name;
		self.save_collection_to_file(collection_index);

		Ok(())
	}

	pub fn duplicate_folder(
		&mut self,
		collection_index: usize,
		folder_index: usize,
	) -> anyhow::Result<()> {
		let folder = self.core.collections[collection_index].folders[folder_index].clone();

		info!("Folder \"{}\" duplicated", folder.name);

		let mut cloned = folder;
		cloned.name = format!("{} copy", cloned.name);
		self.core.collections[collection_index]
			.folders
			.insert(folder_index + 1, cloned);
		self.save_collection_to_file(collection_index);
		Ok(())
	}

	pub fn duplicate_folder_request(
		&mut self,
		collection_index: usize,
		folder_index: usize,
		request_index: usize,
	) -> anyhow::Result<()> {
		let request = self.core.collections[collection_index].folders[folder_index].requests
			[request_index]
			.read()
			.clone();

		info!("Request \"{}\" in folder duplicated", request.name);

		let mut cloned = request;
		cloned.name = format!("{} copy", cloned.name);
		self.core.collections[collection_index].folders[folder_index]
			.requests
			.insert(request_index + 1, Arc::new(RwLock::new(cloned)));
		self.save_collection_to_file(collection_index);
		Ok(())
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
