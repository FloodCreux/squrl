use crate::app::app::App;

impl App<'_> {
	/// Reset selection of if params are provided, either set it to none
	pub fn tui_update_query_params_selection(&mut self) {
		let Some(local_selected_request) = self.get_selected_request_as_local() else {
			return;
		};
		let selected_request = local_selected_request.read();

		match selected_request.params.is_empty() {
			false => self
				.request_editor
				.query_params_table
				.update_selection(Some((0, 0))),
			true => self
				.request_editor
				.query_params_table
				.update_selection(None),
		}
	}

	pub fn tui_modify_request_query_param(&mut self) {
		let input_text = self
			.request_editor
			.query_params_table
			.selection_text_input
			.to_string();
		let Some(selected) = self.collections_tree.selected else {
			return;
		};

		let Some(selection) = self.request_editor.query_params_table.selection else {
			return;
		};

		match self.modify_request_query_param(
			selected.collection_index(),
			selected.request_index(),
			input_text,
			selection.1,
			selection.0,
		) {
			Ok(_) => {}
			Err(_) => return,
		}

		self.select_request_state();
	}

	pub fn tui_create_new_query_param(&mut self) {
		let Some(selected) = self.collections_tree.selected else {
			return;
		};

		match self.create_new_query_param(
			selected.collection_index(),
			selected.request_index(),
			String::from("param"),
			String::from("value"),
		) {
			Ok(_) => {}
			Err(_) => return,
		}

		self.tui_update_query_params_selection();
		self.update_inputs();
	}

	pub fn tui_delete_query_param(&mut self) {
		if self.request_editor.query_params_table.rows.is_empty()
			|| self.request_editor.query_params_table.selection.is_none()
		{
			return;
		}

		let Some(selection) = self.request_editor.query_params_table.selection else {
			return;
		};
		let Some(selected) = self.collections_tree.selected else {
			return;
		};

		match self.delete_query_param(
			selected.collection_index(),
			selected.request_index(),
			selection.0,
		) {
			Ok(_) => {}
			Err(_) => return,
		}

		self.tui_update_query_params_selection();
		self.update_inputs();
	}

	pub fn tui_toggle_query_param(&mut self) {
		if self.request_editor.query_params_table.rows.is_empty()
			|| self.request_editor.query_params_table.selection.is_none()
		{
			return;
		}

		let Some(selection) = self.request_editor.query_params_table.selection else {
			return;
		};
		let row = selection.0;
		let Some(selected) = self.collections_tree.selected else {
			return;
		};

		match self.toggle_query_param(
			selected.collection_index(),
			selected.request_index(),
			None,
			row,
		) {
			Ok(_) => {}
			Err(_) => return,
		}

		self.update_inputs();
	}

	pub fn tui_duplicate_query_param(&mut self) {
		if self.request_editor.query_params_table.rows.is_empty()
			|| self.request_editor.query_params_table.selection.is_none()
		{
			return;
		}

		let Some(selection) = self.request_editor.query_params_table.selection else {
			return;
		};
		let row = selection.0;
		let Some(selected) = self.collections_tree.selected else {
			return;
		};

		match self.duplicate_query_param(selected.collection_index(), selected.request_index(), row)
		{
			Ok(_) => {}
			Err(_) => return,
		}

		self.update_inputs();
	}
}
