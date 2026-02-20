use crate::app::App;
use crate::models::protocol::protocol::Protocol;

impl App<'_> {
	pub fn tui_modify_graphql_query(&mut self) {
		let Some(selected) = self.collections_tree.selected else {
			return;
		};
		let local_selected_request = self.get_request_from_selection(&selected);

		{
			let mut selected_request = local_selected_request.write();

			if let Protocol::GraphqlRequest(gql) = &mut selected_request.protocol {
				gql.query = self.graphql_query_text_area.to_string();
			}
		}

		self.save_collection_to_file(selected.collection_index());
		self.select_request_state();
	}

	pub fn tui_modify_graphql_variables(&mut self) {
		let Some(selected) = self.collections_tree.selected else {
			return;
		};
		let local_selected_request = self.get_request_from_selection(&selected);

		{
			let mut selected_request = local_selected_request.write();

			if let Protocol::GraphqlRequest(gql) = &mut selected_request.protocol {
				gql.variables = self.graphql_variables_text_area.to_string();
			}
		}

		self.save_collection_to_file(selected.collection_index());
		self.select_request_state();
	}
}
