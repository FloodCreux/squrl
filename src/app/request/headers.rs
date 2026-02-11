use crate::app::app::App;

impl App<'_> {
	pub fn find_header(
		&mut self,
		collection_index: usize,
		request_index: usize,
		key: &str,
	) -> anyhow::Result<usize> {
		self.find_kv(collection_index, request_index, key, |req| &req.headers)
	}

	pub fn modify_request_header(
		&mut self,
		collection_index: usize,
		request_index: usize,
		value: String,
		column: usize,
		row: usize,
	) -> anyhow::Result<()> {
		self.modify_kv(
			collection_index,
			request_index,
			value,
			column,
			row,
			"Header",
			|req| &mut req.headers,
		)
	}

	pub fn create_new_header(
		&mut self,
		collection_index: usize,
		request_index: usize,
		key: String,
		value: String,
	) -> anyhow::Result<()> {
		self.create_kv(
			collection_index,
			request_index,
			key,
			value,
			"headers",
			|req| &mut req.headers,
		)
	}

	pub fn delete_header(
		&mut self,
		collection_index: usize,
		request_index: usize,
		row: usize,
	) -> anyhow::Result<()> {
		self.delete_kv(collection_index, request_index, row, "Header", |req| {
			&mut req.headers
		})
	}

	pub fn toggle_header(
		&mut self,
		collection_index: usize,
		request_index: usize,
		state: Option<bool>,
		row: usize,
	) -> anyhow::Result<()> {
		self.toggle_kv(
			collection_index,
			request_index,
			state,
			row,
			"Header",
			|req| &mut req.headers,
		)
	}

	pub fn duplicate_header(
		&mut self,
		collection_index: usize,
		request_index: usize,
		row: usize,
	) -> anyhow::Result<()> {
		self.duplicate_kv(collection_index, request_index, row, "Header", |req| {
			&mut req.headers
		})
	}
}
