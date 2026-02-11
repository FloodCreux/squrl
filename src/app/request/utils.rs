use std::sync::Arc;

use parking_lot::RwLock;

use crate::app::app::App;
use crate::models::request::Request;

impl App<'_> {
	pub fn get_selected_request_as_local(&self) -> Arc<RwLock<Request>> {
		let selected_request_index = &self.collections_tree.selected.unwrap();
		self.collections[selected_request_index.0].requests[selected_request_index.1].clone()
	}

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
		let local = self.get_request_as_local_from_indexes(&(collection_index, request_index));
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
		let local = self.get_request_as_local_from_indexes(&(collection_index, request_index));

		{
			let mut req = local.write();
			f(&mut req)?;
		}

		self.save_collection_to_file(collection_index);
		Ok(())
	}
}
