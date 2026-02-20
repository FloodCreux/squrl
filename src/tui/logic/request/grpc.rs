use crate::app::App;
use crate::models::protocol::protocol::Protocol;

impl App<'_> {
	pub fn tui_modify_grpc_proto_file(&mut self) {
		let Some(selected) = self.collections_tree.selected else {
			return;
		};
		let local_selected_request = self.get_request_from_selection(&selected);

		{
			let mut selected_request = local_selected_request.write();

			if let Protocol::GrpcRequest(grpc) = &mut selected_request.protocol {
				grpc.proto_file = self.grpc_proto_file_input.to_string();
			}
		}

		self.save_collection_to_file(selected.collection_index());
		self.select_request_state();
	}

	pub fn tui_modify_grpc_service(&mut self) {
		let Some(selected) = self.collections_tree.selected else {
			return;
		};
		let local_selected_request = self.get_request_from_selection(&selected);

		{
			let mut selected_request = local_selected_request.write();

			if let Protocol::GrpcRequest(grpc) = &mut selected_request.protocol {
				grpc.service = self.grpc_service_input.to_string();
			}
		}

		self.save_collection_to_file(selected.collection_index());
		self.select_request_state();
	}

	pub fn tui_modify_grpc_method(&mut self) {
		let Some(selected) = self.collections_tree.selected else {
			return;
		};
		let local_selected_request = self.get_request_from_selection(&selected);

		{
			let mut selected_request = local_selected_request.write();

			if let Protocol::GrpcRequest(grpc) = &mut selected_request.protocol {
				grpc.method = self.grpc_method_input.to_string();
			}
		}

		self.save_collection_to_file(selected.collection_index());
		self.select_request_state();
	}

	pub fn tui_modify_grpc_message(&mut self) {
		let Some(selected) = self.collections_tree.selected else {
			return;
		};
		let local_selected_request = self.get_request_from_selection(&selected);

		{
			let mut selected_request = local_selected_request.write();

			if let Protocol::GrpcRequest(grpc) = &mut selected_request.protocol {
				grpc.message = self.grpc_message_text_area.to_string();
			}
		}

		self.save_collection_to_file(selected.collection_index());
		self.select_request_state();
	}
}
