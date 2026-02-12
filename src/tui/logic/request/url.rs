use crate::app::app::App;
use tracing::info;

impl App<'_> {
	pub fn tui_modify_request_url(&mut self) {
		let input_text = self.url_text_input.to_string();

		if input_text.trim().is_empty() {
			return;
		}

		let Some(selected) = self.collections_tree.selected else {
			return;
		};

		self.with_selected_request_write(&selected, |req| {
			req.update_url_and_params(input_text);
			info!("URL set to \"{}\"", &req.url);
		});

		// In case new params were inputted or deleted
		self.tui_update_query_params_selection();
		self.select_request_state();
	}
}
