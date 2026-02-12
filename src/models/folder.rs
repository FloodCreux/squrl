use std::sync::Arc;

use parking_lot::RwLock;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tui_tree_widget::TreeItem;

use crate::app::files::theme::THEME;
use crate::models::request::Request;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Folder {
	pub name: String,
	pub requests: Vec<Arc<RwLock<Request>>>,
}

impl Folder {
	pub fn to_tree_item<'a>(&self, identifier: usize) -> TreeItem<'a, usize> {
		let name = self.name.clone();

		let line = Line::from(vec![
			Span::raw("📁 "),
			Span::raw(name).fg(THEME.read().ui.font_color),
			Span::from(format!(" ({})", self.requests.len())),
		]);

		let items: Vec<TreeItem<usize>> = self
			.requests
			.par_iter()
			.enumerate()
			.map(|(request_index, request)| {
				request
					.read()
					.to_tree_item(request_index, request_index == self.requests.len() - 1)
			})
			.collect();

		TreeItem::new(identifier, line, items).expect("tree item creation should succeed")
	}
}
