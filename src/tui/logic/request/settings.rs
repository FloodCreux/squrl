use crate::app::app::App;

impl App<'_> {
	pub fn tui_modify_request_settings(&mut self) {
		let Some(selected) = self.collections_tree.selected else {
			return;
		};
		let local_selected_request = self.get_request_from_selection(&selected);

		{
			let mut selected_request = local_selected_request.write();

			selected_request
				.settings
				.update_from_vec(&self.request_settings_popup.settings)
		}

		self.save_collection_to_file(selected.collection_index());
		self.select_request_state();
	}
}
