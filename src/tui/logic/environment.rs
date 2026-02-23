use indexmap::map::MutableKeys;
use parking_lot::RwLock;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use regex::Regex;
use std::sync::Arc;

use crate::app::App;
use crate::app::files::environment::OS_ENV_VARS;
use crate::app::files::theme::THEME;
use crate::models::environment::Environment;
use crate::models::request::KeyValue;

impl App<'_> {
	/// Get the collection index of the currently selected item in the tree, if any.
	pub fn tui_selected_collection_index(&self) -> Option<usize> {
		let selected = self.collections_tree.state.selected();
		if selected.is_empty() {
			None
		} else {
			Some(selected[0])
		}
	}

	/// Returns true if the currently selected collection has its own environments.
	pub fn tui_active_collection_has_envs(&self) -> bool {
		self.tui_selected_collection_index()
			.and_then(|ci| self.core.collections.get(ci))
			.is_some_and(|c| !c.environments.is_empty())
	}

	pub fn tui_next_environment(&mut self) {
		// If the selected collection has its own environments, cycle those
		if let Some(ci) = self.tui_selected_collection_index()
			&& !self.core.collections[ci].environments.is_empty()
		{
			self.tui_next_collection_environment(ci);
			return;
		}

		// Otherwise cycle global environments
		if self.core.selected_environment + 1 < self.core.environments.len() {
			self.core.selected_environment += 1;
		} else {
			self.core.selected_environment = 0;
		}
	}

	pub fn tui_update_env_variable_table(&mut self) {
		// If the selected collection has its own environments, show those
		if let Some(ci) = self.tui_selected_collection_index() {
			let collection = &self.core.collections[ci];
			if !collection.environments.is_empty() {
				let rows: Vec<KeyValue> =
					if let Some(ref selected_name) = collection.selected_environment {
						collection
							.environments
							.iter()
							.find(|e| &e.name == selected_name)
							.map(|env| {
								env.values
									.iter()
									.map(|(key, value)| KeyValue {
										enabled: true,
										data: (key.clone(), value.clone()),
									})
									.collect()
							})
							.unwrap_or_default()
					} else {
						vec![]
					};

				match rows.is_empty() {
					false => self.env_editor_table.update_selection(Some((0, 0))),
					true => self.env_editor_table.update_selection(None),
				};
				self.env_editor_table.rows = rows;
				return;
			}
		}

		// Fall back to global environment
		let rows: Vec<KeyValue> = match self.get_selected_env_as_local() {
			Some(local_env) => {
				let env = local_env.read();
				env.values
					.iter()
					.map(|(key, value)| KeyValue {
						enabled: true,
						data: (key.clone(), value.clone()),
					})
					.collect()
			}
			None => vec![],
		};

		match rows.is_empty() {
			false => self.env_editor_table.update_selection(Some((0, 0))),
			true => self.env_editor_table.update_selection(None),
		};
		self.env_editor_table.rows = rows;
	}

	pub fn tui_modify_env_variable(&mut self) {
		let Some((row, column)) = self.env_editor_table.selection else {
			return;
		};

		let input_text = self.env_editor_table.selection_text_input.to_string();

		// If the selected collection has its own environments, modify those
		if let Some(ci) = self.tui_selected_collection_index() {
			let collection = &self.core.collections[ci];
			if !collection.environments.is_empty()
				&& let Some(ref selected_name) = collection.selected_environment.clone()
			{
				let env_idx = match self.find_collection_environment(ci, selected_name) {
					Ok(idx) => idx,
					Err(_) => return,
				};
				let env = &mut self.core.collections[ci].environments[env_idx];

				match column {
					0 => {
						// Rename key by index
						if env.values.get(&input_text).is_some() {
							// Key already exists, ignore
						} else if let Some((k, _)) = env.values.get_index_mut2(row) {
							*k = input_text;
						}
					}
					1 => {
						// Set value by index
						if let Some((_, v)) = env.values.get_index_mut(row) {
							*v = input_text;
						}
					}
					_ => {}
				};
				self.save_collection_to_file(ci);
				self.display_env_editor_state();
				return;
			}
		}

		// Fall back to global environment
		let selected_env_index = self.core.selected_environment;

		// Ignore errors to avoid getting locked in the current state
		match column {
			0 => self
				.rename_env_key_by_index(selected_env_index, row, input_text)
				.ok(), // Ignored error, key already exists
			1 => self
				.set_env_value_by_index(selected_env_index, row, input_text)
				.ok(), // Ignored error, key not found
			_ => None,
		};

		self.display_env_editor_state();
	}

	pub fn tui_create_env_variable(&mut self) {
		if let Some(ci) = self.tui_selected_collection_index() {
			let collection = &self.core.collections[ci];

			// If the collection has no environments yet, auto-create a "default" one
			if collection.environments.is_empty() {
				let _ = self.create_collection_environment(ci, "default".to_string());
				let _ = self.select_collection_environment(ci, Some("default".to_string()));
			}

			// Now create the variable in the selected collection environment
			let collection = &self.core.collections[ci];
			if let Some(ref selected_name) = collection.selected_environment.clone() {
				let _ = self.create_collection_env_value(
					ci,
					selected_name,
					format!(
						"KEY_{}",
						self.core.collections[ci]
							.environments
							.iter()
							.find(|e| &e.name == selected_name)
							.map(|e| e.values.len())
							.unwrap_or(0)
					),
					String::from("VALUE"),
				);
				self.tui_update_env_variable_table();
				return;
			}
		}

		// Fall back to global environment
		let selected_env_index = self.core.selected_environment;

		match self.create_env_value(selected_env_index, None, String::from("VALUE")) {
			Ok(_) => {}
			Err(_) => return,
		}

		self.tui_update_env_variable_table();
	}

	pub fn tui_delete_env_variable(&mut self) {
		if self.env_editor_table.rows.is_empty() || self.env_editor_table.selection.is_none() {
			return;
		}

		let Some((row, _)) = self.env_editor_table.selection else {
			return;
		};

		// If the selected collection has its own environments, delete from there
		if let Some(ci) = self.tui_selected_collection_index() {
			let collection = &self.core.collections[ci];
			if !collection.environments.is_empty()
				&& let Some(ref selected_name) = collection.selected_environment.clone()
			{
				let env_idx = match self.find_collection_environment(ci, selected_name) {
					Ok(idx) => idx,
					Err(_) => return,
				};
				let env = &mut self.core.collections[ci].environments[env_idx];
				env.values.shift_remove_index(row);
				self.save_collection_to_file(ci);
				self.tui_update_env_variable_table();
				return;
			}
		}

		// Fall back to global environment
		let selected_env_index = self.core.selected_environment;

		match self.delete_env_index(selected_env_index, row) {
			Ok(_) => {}
			Err(_) => return,
		}

		self.tui_update_env_variable_table();
	}
}

pub fn tui_add_color_to_env_keys<'a>(
	local_env: &Option<Arc<RwLock<Environment>>>,
	input: String,
) -> Line<'a> {
	tui_add_color_to_env_keys_with_collection(local_env, None, input)
}

/// Like `tui_add_color_to_env_keys` but also highlights keys from a
/// collection-scoped environment.
pub fn tui_add_color_to_env_keys_with_collection<'a>(
	local_env: &Option<Arc<RwLock<Environment>>>,
	collection_env: Option<&Environment>,
	input: String,
) -> Line<'a> {
	if !input.contains('{') {
		return Line::raw(input);
	}

	let mut spans: Vec<Span> = vec![];

	let regex = Regex::new(r"\{\{(\w+)}}").expect("valid env variable regex");
	let mut tmp_index: usize = 0;

	// Gather known keys from all sources
	let has_any_env = local_env.is_some() || collection_env.is_some();

	if has_any_env {
		let mut keys: Vec<String> = Vec::new();

		// Collection env keys (highest priority)
		if let Some(coll_env) = collection_env {
			keys.extend(coll_env.values.keys().cloned());
		}

		// Global env keys
		if let Some(local_env) = local_env {
			let env = local_env.read();
			keys.extend(env.values.keys().cloned());
		}

		// OS env and built-in keys
		keys.extend(OS_ENV_VARS.keys().cloned());
		keys.extend(
			["NOW", "TIMESTAMP", "UUIDv4", "UUIDv7"]
				.iter()
				.map(|s| s.to_string()),
		);

		for match_ in regex.captures_iter(&input) {
			for sub_match in match_.iter().flatten() {
				for key in &keys {
					if sub_match.as_str() == format!("{{{{{}}}}}", key) {
						let range = sub_match.range();

						spans.push(Span::raw(input[tmp_index..range.start].to_string()));
						spans.push(
							Span::raw(sub_match.as_str().to_owned())
								.fg(THEME.read().others.environment_variable_highlight_color),
						);

						tmp_index = range.end;
					}
				}
			}
		}

		spans.push(Span::raw(String::from(&input[tmp_index..input.len()])));
	} else {
		spans.push(Span::raw(input.to_string()));
	}

	Line::from(spans)
}
