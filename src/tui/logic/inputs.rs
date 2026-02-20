use crate::app::App;
use crate::app::files::key_bindings::{KEY_BINDINGS, TextAreaMode};
use edtui::EditorMode;

macro_rules! for_all_inputs {
	($self:ident, |$input:ident| $body:expr) => {{
		{
			let $input = &mut $self.env_editor_table.selection_text_input;
			$body;
		}
		{
			let $input = &mut $self.collection_popups.new_collection_input;
			$body;
		}
		{
			let $input = &mut $self.collection_popups.new_request_popup.text_input;
			$body;
		}
		{
			let $input = &mut $self.collection_popups.rename_collection_input;
			$body;
		}
		{
			let $input = &mut $self.collection_popups.rename_request_input;
			$body;
		}
		{
			let $input = &mut $self.request_editor.url_input;
			$body;
		}
		{
			let $input = &mut $self.request_editor.query_params_table.selection_text_input;
			$body;
		}
		{
			let $input = &mut $self.request_editor.auth.basic_username;
			$body;
		}
		{
			let $input = &mut $self.request_editor.auth.basic_password;
			$body;
		}
		{
			let $input = &mut $self.request_editor.auth.bearer_token;
			$body;
		}
		{
			let $input = &mut $self.request_editor.auth.jwt_secret;
			$body;
		}
		{
			let $input = &mut $self.request_editor.auth.jwt_payload;
			$body;
		}
		{
			let $input = &mut $self.request_editor.auth.digest_username;
			$body;
		}
		{
			let $input = &mut $self.request_editor.auth.digest_password;
			$body;
		}
		{
			let $input = &mut $self.request_editor.auth.digest_domains;
			$body;
		}
		{
			let $input = &mut $self.request_editor.auth.digest_realm;
			$body;
		}
		{
			let $input = &mut $self.request_editor.auth.digest_nonce;
			$body;
		}
		{
			let $input = &mut $self.request_editor.auth.digest_opaque;
			$body;
		}
		{
			let $input = &mut $self.request_editor.headers_table.selection_text_input;
			$body;
		}
		{
			let $input = &mut $self.request_editor.body_text_area;
			$body;
		}
		{
			let $input = &mut $self.request_editor.body_form_table.selection_text_input;
			$body;
		}
		{
			let $input = &mut $self.request_editor.body_file_input;
			$body;
		}
		{
			let $input = &mut $self.message_text_area;
			$body;
		}
		{
			let $input = &mut $self.graphql_query_text_area;
			$body;
		}
		{
			let $input = &mut $self.graphql_variables_text_area;
			$body;
		}
		{
			let $input = &mut $self.grpc_proto_file_input;
			$body;
		}
		{
			let $input = &mut $self.grpc_service_input;
			$body;
		}
		{
			let $input = &mut $self.grpc_method_input;
			$body;
		}
		{
			let $input = &mut $self.grpc_message_text_area;
			$body;
		}
		{
			let $input = &mut $self.script_console.pre_request_text_area;
			$body;
		}
		{
			let $input = &mut $self.script_console.post_request_text_area;
			$body;
		}
	}};
}

impl App<'_> {
	pub fn reset_inputs_mode(&mut self) {
		for_all_inputs!(self, |input| input.reset_mode());
	}

	pub fn clear_inputs(&mut self) {
		for_all_inputs!(self, |input| input.clear());
	}

	pub fn reset_cursors(&mut self) {
		for_all_inputs!(self, |input| {
			input.reset_cursor_position();
			input.reset_selection()
		});
	}

	pub fn update_text_inputs_handler(&mut self) {
		let default_mode = match KEY_BINDINGS.read().generic.text_input.mode {
			TextAreaMode::Vim => EditorMode::Normal,
			TextAreaMode::Emacs | TextAreaMode::Default | TextAreaMode::Custom(_) => {
				EditorMode::Insert
			}
		};

		for_all_inputs!(self, |input| {
			input.default_mode = default_mode;
			input.is_single_line = true;
			input.update_handler()
		});

		// Override default_mode for table inputs (always Insert mode)
		self.env_editor_table.selection_text_input.default_mode = EditorMode::Insert;
		self.request_editor
			.query_params_table
			.selection_text_input
			.default_mode = EditorMode::Insert;
		self.request_editor
			.headers_table
			.selection_text_input
			.default_mode = EditorMode::Insert;
		self.request_editor
			.body_form_table
			.selection_text_input
			.default_mode = EditorMode::Insert;

		// Override is_single_line for multi-line inputs
		self.request_editor.auth.jwt_payload.is_single_line = false;
		self.request_editor.body_text_area.is_single_line = false;
		self.message_text_area.is_single_line = false;
		self.graphql_query_text_area.is_single_line = false;
		self.graphql_variables_text_area.is_single_line = false;
		self.grpc_message_text_area.is_single_line = false;
		self.script_console.pre_request_text_area.is_single_line = false;
		self.script_console.post_request_text_area.is_single_line = false;

		// Set insert_mode_only for table inputs
		self.env_editor_table.selection_text_input.insert_mode_only = true;
		self.request_editor
			.query_params_table
			.selection_text_input
			.insert_mode_only = true;
		self.request_editor
			.headers_table
			.selection_text_input
			.insert_mode_only = true;
		self.request_editor
			.body_form_table
			.selection_text_input
			.insert_mode_only = true;

		self.reset_inputs_mode();
	}
}
