use crate::app::app::App;

impl App<'_> {
	pub fn find_query_param(
		&mut self,
		collection_index: usize,
		request_index: usize,
		key: &str,
	) -> anyhow::Result<usize> {
		self.find_kv(collection_index, request_index, key, |req| &req.params)
	}

	pub fn modify_request_query_param(
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
			"Query params",
			|req| &mut req.params,
		)
	}

	pub fn create_new_query_param(
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
			"Query params",
			|req| &mut req.params,
		)
	}

	pub fn delete_query_param(
		&mut self,
		collection_index: usize,
		request_index: usize,
		row: usize,
	) -> anyhow::Result<()> {
		self.delete_kv(
			collection_index,
			request_index,
			row,
			"Query params",
			|req| &mut req.params,
		)
	}

	pub fn toggle_query_param(
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
			"Query params",
			|req| &mut req.params,
		)
	}

	pub fn duplicate_query_param(
		&mut self,
		collection_index: usize,
		request_index: usize,
		row: usize,
	) -> anyhow::Result<()> {
		self.duplicate_kv(
			collection_index,
			request_index,
			row,
			"Query params",
			|req| &mut req.params,
		)
	}
}
