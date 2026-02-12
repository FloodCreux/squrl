use crate::app::app::App;
use crate::models::protocol::http::method::next_method;
use tracing::info;

impl App<'_> {
	pub fn tui_next_request_method(&mut self) {
		let Some(selected) = self.collections_tree.selected else {
			return;
		};
		let local_selected_request = self.get_request_from_selection(&selected);

		{
			let mut selected_request = local_selected_request.write();
			let selected_http_request = selected_request
				.get_http_request_mut()
				.expect("request should be HTTP");

			let next_method = next_method(&selected_http_request.method);

			info!("Method set to \"{}\"", next_method);

			selected_http_request.method = next_method;
		}

		self.save_collection_to_file(selected.collection_index());
	}
}
