use std::sync::Arc;

use parking_lot::RwLock;

use crate::app::app::App;
use crate::models::request::Request;
use crate::tui::utils::stateful::stateful_tree::SelectedRequest;

impl App<'_> {
	pub fn get_selected_request_as_local(&self) -> Arc<RwLock<Request>> {
		let selected = &self.collections_tree.selected.unwrap();
		self.get_request_from_selection(selected)
	}

	/// Resolve a request from a SelectedRequest enum.
	pub fn get_request_from_selection(&self, selected: &SelectedRequest) -> Arc<RwLock<Request>> {
		match selected {
			SelectedRequest::RootRequest(collection_index, request_index) => {
				self.collections[*collection_index].requests[*request_index].clone()
			}
			SelectedRequest::FolderRequest(collection_index, folder_index, request_index) => {
				self.collections[*collection_index].folders[*folder_index].requests[*request_index]
					.clone()
			}
		}
	}

	/// Legacy compatibility: resolve a request from (collection_index, request_index) tuple.
	/// This only works for root-level requests.
	pub fn get_request_as_local_from_indexes(
		&self,
		selected_request_index: &(usize, usize),
	) -> Arc<RwLock<Request>> {
		self.collections[selected_request_index.0].requests[selected_request_index.1].clone()
	}

	pub fn with_request_write<F, R>(
		&mut self,
		collection_index: usize,
		request_index: usize,
		f: F,
	) -> R
	where
		F: FnOnce(&mut Request) -> R,
	{
		// Check if this is actually a folder request by examining the current selection
		let local = match &self.collections_tree.selected {
			Some(selected)
				if selected.collection_index() == collection_index
					&& selected.request_index() == request_index =>
			{
				self.get_request_from_selection(selected)
			}
			_ => self.get_request_as_local_from_indexes(&(collection_index, request_index)),
		};
		let result = {
			let mut req = local.write();
			f(&mut req)
		};

		self.save_collection_to_file(collection_index);
		result
	}

	pub fn with_request_write_result<F>(
		&mut self,
		collection_index: usize,
		request_index: usize,
		f: F,
	) -> anyhow::Result<()>
	where
		F: FnOnce(&mut Request) -> anyhow::Result<()>,
	{
		// Check if this is actually a folder request by examining the current selection
		let local = match &self.collections_tree.selected {
			Some(selected)
				if selected.collection_index() == collection_index
					&& selected.request_index() == request_index =>
			{
				self.get_request_from_selection(selected)
			}
			_ => self.get_request_as_local_from_indexes(&(collection_index, request_index)),
		};

		{
			let mut req = local.write();
			f(&mut req)?;
		}

		self.save_collection_to_file(collection_index);
		Ok(())
	}

	/// Write to a request resolved from a SelectedRequest, saving the collection afterward.
	pub fn with_selected_request_write<F, R>(&mut self, selected: &SelectedRequest, f: F) -> R
	where
		F: FnOnce(&mut Request) -> R,
	{
		let local = self.get_request_from_selection(selected);
		let result = {
			let mut req = local.write();
			f(&mut req)
		};

		self.save_collection_to_file(selected.collection_index());
		result
	}
}
