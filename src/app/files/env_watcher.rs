use std::path::PathBuf;
use std::sync::Arc;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::Mutex;
use tracing::{trace, warn};

use crate::app::App;
use crate::models::collection::CollectionFileFormat;
use crate::tui::app_states::AppState;

pub fn spawn_env_json_watcher(
	watch_dir: PathBuf,
	changed: Arc<Mutex<bool>>,
) -> Option<RecommendedWatcher> {
	let target_file = watch_dir.join("squrl-env.json");

	let watcher = notify::recommended_watcher(move |res: Result<Event, _>| match res {
		Ok(event) => {
			let dominated = event.paths.iter().any(|p| p == &target_file);
			if !dominated {
				return;
			}

			if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
				trace!("squrl-env.json changed on disk");
				*changed.lock() = true;
			}
		}
		Err(e) => warn!("File watcher error: {e}"),
	});

	match watcher {
		Ok(mut w) => {
			if let Err(e) = w.watch(&watch_dir, RecursiveMode::NonRecursive) {
				warn!("Could not watch directory \"{}\": {e}", watch_dir.display());
				return None;
			}

			trace!(
				"Watching \"{}\" for squrl-env.json changes",
				watch_dir.display()
			);
			Some(w)
		}
		Err(e) => {
			warn!("Could not create file watcher: {e}");
			None
		}
	}
}

impl App<'_> {
	pub fn reload_companion_env(&mut self) {
		let Some(collection_index) = self
			.core
			.collections
			.iter()
			.position(|c| matches!(c.file_format, CollectionFileFormat::Http))
		else {
			return;
		};

		let collection_path = self.core.collections[collection_index].path.clone();

		let (new_environments, new_selected) =
			App::load_companion_env_file(&collection_path).unwrap_or_default();

		let collection = &mut self.core.collections[collection_index];
		collection.environments = new_environments;
		collection.selected_environment = new_selected;

		trace!(
			"Reloaded squrl-env.json for collection \"{}\" ({} environments)",
			collection.name,
			collection.environments.len(),
		);

		if matches!(
			self.state,
			AppState::DisplayingEnvEditor | AppState::EditingEnvVariable
		) {
			self.tui_update_env_variable_table();
		}
	}
}
