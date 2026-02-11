use crate::app::app::App;
use crate::app::files::key_bindings::{KEY_BINDINGS, TextAreaMode};
use edtui::EditorMode;

macro_rules! for_all_inputs {
	($self:ident, |$input:ident| $body:expr) => {{
		{ let $input = &mut $self.env_editor_table.selection_text_input; $body; }
		{ let $input = &mut $self.new_collection_input; $body; }
		{ let $input = &mut $self.new_request_popup.text_input; $body; }
		{ let $input = &mut $self.rename_collection_input; $body; }
		{ let $input = &mut $self.rename_request_input; $body; }
		{ let $input = &mut $self.url_text_input; $body; }
		{ let $input = &mut $self.query_params_table.selection_text_input; $body; }
		{ let $input = &mut $self.auth_basic_username_text_input; $body; }
		{ let $input = &mut $self.auth_basic_password_text_input; $body; }
		{ let $input = &mut $self.auth_bearer_token_text_input; $body; }
		{ let $input = &mut $self.auth_jwt_secret_text_input; $body; }
		{ let $input = &mut $self.auth_jwt_payload_text_area; $body; }
		{ let $input = &mut $self.auth_digest_username_text_input; $body; }
		{ let $input = &mut $self.auth_digest_password_text_input; $body; }
		{ let $input = &mut $self.auth_digest_domains_text_input; $body; }
		{ let $input = &mut $self.auth_digest_realm_text_input; $body; }
		{ let $input = &mut $self.auth_digest_nonce_text_input; $body; }
		{ let $input = &mut $self.auth_digest_opaque_text_input; $body; }
		{ let $input = &mut $self.headers_table.selection_text_input; $body; }
		{ let $input = &mut $self.body_text_area; $body; }
		{ let $input = &mut $self.body_form_table.selection_text_input; $body; }
		{ let $input = &mut $self.body_file_text_input; $body; }
		{ let $input = &mut $self.message_text_area; $body; }
		{ let $input = &mut $self.script_console.pre_request_text_area; $body; }
		{ let $input = &mut $self.script_console.post_request_text_area; $body; }
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
		self.query_params_table.selection_text_input.default_mode = EditorMode::Insert;
		self.headers_table.selection_text_input.default_mode = EditorMode::Insert;
		self.body_form_table.selection_text_input.default_mode = EditorMode::Insert;

		// Override is_single_line for multi-line inputs
		self.auth_jwt_payload_text_area.is_single_line = false;
		self.body_text_area.is_single_line = false;
		self.message_text_area.is_single_line = false;
		self.script_console.pre_request_text_area.is_single_line = false;
		self.script_console.post_request_text_area.is_single_line = false;

		// Set insert_mode_only for table inputs
		self.env_editor_table.selection_text_input.insert_mode_only = true;
		self.query_params_table.selection_text_input.insert_mode_only = true;
		self.headers_table.selection_text_input.insert_mode_only = true;
		self.body_form_table.selection_text_input.insert_mode_only = true;

		self.reset_inputs_mode();
	}
}
