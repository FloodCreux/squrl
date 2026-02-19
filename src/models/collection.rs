use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use strum::Display;
use tui_tree_widget::TreeItem;

use crate::app::files::theme::THEME;
use crate::models::folder::Folder;
use crate::models::request::Request;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Collection {
	pub name: String,
	pub last_position: Option<usize>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub folders: Vec<Folder>,
	pub requests: Vec<Arc<RwLock<Request>>>,

	#[serde(skip)]
	pub path: PathBuf,

	#[serde(skip)]
	pub file_format: CollectionFileFormat,
}

#[derive(Debug, Default, Copy, Clone, Display, Serialize, Deserialize)]
pub enum CollectionFileFormat {
	#[default]
	#[serde(alias = "json", alias = "JSON")]
	#[strum(to_string = "json")]
	Json,
	#[serde(alias = "yaml", alias = "YAML")]
	#[strum(to_string = "yaml")]
	Yaml,
	#[serde(alias = "http", alias = "HTTP")]
	#[strum(to_string = "http")]
	Http,
}

impl Collection {
	/// Returns the total number of requests across folders and root-level requests
	pub fn total_request_count(&self) -> usize {
		let folder_requests: usize = self.folders.iter().map(|f| f.requests.len()).sum();
		folder_requests + self.requests.len()
	}

	/// Returns the number of direct children (folders + root requests) for tree display
	pub fn children_count(&self) -> usize {
		self.folders.len() + self.requests.len()
	}

	pub fn to_tree_item<'a>(&self, identifier: usize) -> TreeItem<'a, usize> {
		let name = self.name.clone();

		let line = Line::from(vec![
			Span::raw(name).fg(THEME.read().ui.font_color),
			Span::from(format!(" ({})", self.total_request_count())),
		]);

		// Build children: folders first, then root-level requests
		let mut items: Vec<TreeItem<usize>> = Vec::new();

		// Add folders as children
		let folder_items: Vec<TreeItem<usize>> = self
			.folders
			.par_iter()
			.enumerate()
			.map(|(folder_index, folder)| folder.to_tree_item(folder_index))
			.collect();
		items.extend(folder_items);

		// Add root-level requests as children (identifiers offset by folder count)
		let folder_count = self.folders.len();
		let requests_len = self.requests.len();
		let is_last_folder = self.folders.is_empty();
		let request_items: Vec<TreeItem<usize>> = self
			.requests
			.par_iter()
			.enumerate()
			.map(|(request_index, request)| {
				let is_last = is_last_folder && request_index == requests_len - 1;
				request
					.read()
					.to_tree_item(folder_count + request_index, is_last)
			})
			.collect();
		items.extend(request_items);

		TreeItem::new(identifier, line, items).expect("tree item creation should succeed")
	}

	/// Resolves a child index within this collection to either a folder or a root request.
	/// Returns `ChildRef::Folder(folder_index)` or `ChildRef::RootRequest(request_index)`.
	pub fn resolve_child(&self, child_index: usize) -> ChildRef {
		if child_index < self.folders.len() {
			ChildRef::Folder(child_index)
		} else {
			ChildRef::RootRequest(child_index - self.folders.len())
		}
	}
}

/// Represents what a child index within a collection points to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildRef {
	/// A folder at the given index in `collection.folders`
	Folder(usize),
	/// A root-level request at the given index in `collection.requests`
	RootRequest(usize),
}
