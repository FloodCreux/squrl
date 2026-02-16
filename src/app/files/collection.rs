use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, anyhow};
use tracing::{info, trace, warn};

use crate::app::App;
use crate::app::files::utils::write_via_temp_file;
use crate::cli::args::ARGS;
use crate::models::collection::CollectionFileFormat::{Json, Yaml};
use crate::models::collection::{Collection, CollectionFileFormat};

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
				.ok_or_else(|| anyhow!("--directory argument is required"))?
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
				.context("Could not serialize collection to JSON")?,
			Yaml => serde_yaml_ng::to_string(collection)
				.context("Could not serialize collection to YAML")?,
		};

		write_via_temp_file(&collection.path, collection_stringed.as_bytes())
			.context("Could not save collection file")?;

		trace!("Collection saved");
		Ok(())
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
