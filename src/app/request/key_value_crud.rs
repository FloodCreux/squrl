use tracing::info;

use crate::app::App;
use crate::app::key_value::find_key;
use crate::models::request::{KeyValue, Request};

impl App<'_> {
	pub fn find_kv(
		&mut self,
		collection_index: usize,
		request_index: usize,
		key: &str,
		accessor: impl FnOnce(&Request) -> &Vec<KeyValue>,
	) -> anyhow::Result<usize> {
		// Check if this is a folder request by examining the current selection
		let local = match &self.collections_tree.selected {
			Some(selected)
				if selected.collection_index() == collection_index
					&& selected.request_index() == request_index =>
			{
				self.get_request_from_selection(selected)
			}
			_ => self.get_request_as_local_from_indexes(&(collection_index, request_index)),
		};
		let req = local.read();
		find_key(accessor(&req), key)
	}

	#[allow(clippy::too_many_arguments)]
	pub fn modify_kv(
		&mut self,
		collection_index: usize,
		request_index: usize,
		value: String,
		column: usize,
		row: usize,
		label: &str,
		accessor: impl FnOnce(&mut Request) -> &mut Vec<KeyValue>,
	) -> anyhow::Result<()> {
		self.with_request_write(collection_index, request_index, |req| {
			let kv = accessor(req);
			let kv_type = match column {
				0 => "key",
				1 => "value",
				_ => "",
			};

			info!("{label} {kv_type} set to \"{value}\"");

			match column {
				0 => kv[row].data.0 = value,
				1 => kv[row].data.1 = value,
				_ => {}
			}
		});

		Ok(())
	}

	pub fn create_kv(
		&mut self,
		collection_index: usize,
		request_index: usize,
		key: String,
		value: String,
		label: &str,
		accessor: impl FnOnce(&mut Request) -> &mut Vec<KeyValue>,
	) -> anyhow::Result<()> {
		self.with_request_write(collection_index, request_index, |req| {
			info!("Key \"{key}\" with value \"{value}\" added to the {label}");
			accessor(req).push(KeyValue {
				enabled: true,
				data: (key, value),
			});
		});
		Ok(())
	}

	pub fn delete_kv(
		&mut self,
		collection_index: usize,
		request_index: usize,
		row: usize,
		label: &str,
		accessor: impl FnOnce(&mut Request) -> &mut Vec<KeyValue>,
	) -> anyhow::Result<()> {
		self.with_request_write(collection_index, request_index, |req| {
			info!("{label} deleted");
			accessor(req).remove(row);
		});
		Ok(())
	}

	pub fn toggle_kv(
		&mut self,
		collection_index: usize,
		request_index: usize,
		state: Option<bool>,
		row: usize,
		label: &str,
		accessor: impl FnOnce(&mut Request) -> &mut Vec<KeyValue>,
	) -> anyhow::Result<()> {
		self.with_request_write(collection_index, request_index, |req| {
			let kv = accessor(req);
			let new_state = state.unwrap_or(!kv[row].enabled);
			info!("{label} state set to \"{new_state}\"");
			kv[row].enabled = new_state;
		});
		Ok(())
	}

	pub fn duplicate_kv(
		&mut self,
		collection_index: usize,
		request_index: usize,
		row: usize,
		label: &str,
		accessor: impl Fn(&mut Request) -> &mut Vec<KeyValue>,
	) -> anyhow::Result<()> {
		self.with_request_write(collection_index, request_index, |req| {
			info!("{label} duplicated");
			let item = accessor(req)[row].clone();
			accessor(req).insert(row, item);
		});
		Ok(())
	}
}
