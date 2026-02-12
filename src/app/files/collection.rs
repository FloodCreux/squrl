use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;

use tracing::{info, trace, warn};

use crate::app::app::App;
use crate::app::files::utils::write_via_temp_file;
use crate::cli::args::ARGS;
use crate::errors::panic_error;
use crate::models::collection::CollectionFileFormat::{Json, Yaml};
use crate::models::collection::{Collection, CollectionFileFormat};

impl App<'_> {
	/// Set the app request to the requests found in the collection file
	pub fn set_collections_from_file(
		&mut self,
		path_buf: PathBuf,
		file_format: CollectionFileFormat,
	) {
		let mut file_content = String::new();

		trace!("Trying to open \"{}\" collection", path_buf.display());

		let mut collection_file = OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.truncate(false)
			.open(path_buf.clone())
			.expect("\tCould not open collection file");

		collection_file
			.read_to_string(&mut file_content)
			.expect("\tCould not read collection file");

		let mut collection: Collection = match file_format {
			Json => match serde_json::from_str(&file_content) {
				Ok(collection) => collection,
				Err(e) => panic_error(format!(
					"Could not parse JSON collection \"{}\"\n\t{e}",
					path_buf.display()
				)),
			},
			Yaml => match serde_yml::from_str(&file_content) {
				Ok(collection) => collection,
				Err(e) => panic_error(format!(
					"Could not parse YAML collection \"{}\"\n\t{}",
					path_buf.display(),
					e
				)),
			},
		};

		collection.path = path_buf;
		collection.file_format = file_format;

		self.core.collections.push(collection);

		trace!("Collection file parsed!");
	}

	/// Save app collection in the collection file through a temporary file
	pub fn save_collection_to_file(&mut self, collection_index: usize) {
		if !ARGS.should_save {
			warn!("Dry-run, not saving the collection");
			return;
		}

		// Auto-assign a file path for ephemeral collections on first save
		if self.core.collections[collection_index]
			.path
			.as_os_str()
			.is_empty()
		{
			let collection = &self.core.collections[collection_index];
			let file_format = self.core.config.get_preferred_collection_file_format();
			let path = ARGS
				.directory
				.as_ref()
				.unwrap()
				.join(format!("{}.{file_format}", collection.name));

			info!(
				"Ephemeral collection \"{}\" will now be saved to \"{}\"",
				collection.name,
				path.display()
			);

			self.core.collections[collection_index].path = path;
			self.core.collections[collection_index].file_format = file_format;
		}

		let collection = &self.core.collections[collection_index];

		info!("Saving collection \"{}\"", collection.name);

		let collection_stringed = match collection.file_format {
			Json => serde_json::to_string_pretty(collection)
				.expect("Could not serialize collection to JSON"),
			Yaml => {
				serde_yml::to_string(collection).expect("Could not serialize collection to YAML")
			}
		};

		write_via_temp_file(&collection.path, collection_stringed.as_bytes())
			.expect("Could not save collection file");

		trace!("Collection saved");
	}

	/// Delete collection file
	pub fn delete_collection_file(&mut self, collection: Collection) {
		if !ARGS.should_save {
			return;
		}

		fs::remove_file(&collection.path).expect("Could not delete collection file");
	}
}
