use crokey::KeyCombination;

use crate::app::app::App;
use crate::tui::events::AppEvent;
use crate::tui::utils::stateful::table_navigation::TableNavigation;

impl App<'_> {
	pub(in crate::tui::events) fn handle_param_tabs_event(
		&mut self,
		event: &AppEvent,
		_key: KeyCombination,
	) {
		match event {
			/* Param tabs */
			AppEvent::NextParamTab(_) => self.tui_next_request_param_tab(),

			AppEvent::ModifyRequestAuthMethod(_) => self.tui_next_request_auth(),
			AppEvent::ModifyRequestBodyContentType(_) => self.tui_next_request_content_type(),
			AppEvent::ModifyRequestMessageType(_) => self.tui_next_request_message_type(),

			/* Query params */
			AppEvent::EditRequestQueryParam(_) => {
				if self.request_editor.query_params_table.is_selected() {
					self.edit_request_param_state()
				}
			}
			AppEvent::RequestQueryParamsMoveUp(_) => self.request_editor.query_params_table.up(),
			AppEvent::RequestQueryParamsMoveDown(_) => {
				self.request_editor.query_params_table.down()
			}
			AppEvent::RequestQueryParamsMoveLeft(_) | AppEvent::RequestQueryParamsMoveRight(_) => {
				self.request_editor.query_params_table.change_y()
			}
			AppEvent::CreateRequestQueryParam(_) => self.tui_create_new_query_param(),
			AppEvent::DeleteRequestQueryParam(_) => self.tui_delete_query_param(),
			AppEvent::ToggleRequestQueryParam(_) => self.tui_toggle_query_param(),
			AppEvent::DuplicateRequestQueryParam(_) => self.tui_duplicate_query_param(),

			/* Auth */
			AppEvent::EditRequestAuth(_) => {
				if self.request_editor.auth.text_input_selection.usable {
					self.tui_select_request_auth_input_text()
				}
			}
			AppEvent::RequestAuthMoveUp(_) => {
				if self.request_editor.auth.text_input_selection.usable {
					self.request_editor.auth.text_input_selection.previous()
				}
			}
			AppEvent::RequestAuthMoveDown(_) => {
				if self.request_editor.auth.text_input_selection.usable {
					self.request_editor.auth.text_input_selection.next()
				}
			}
			AppEvent::RequestAuthMoveLeft(_) => self.tui_request_auth_move_left(),
			AppEvent::RequestAuthMoveRight(_) => self.tui_request_auth_move_right(),

			/* Headers */
			AppEvent::EditRequestHeader(_) => {
				if self.request_editor.headers_table.is_selected() {
					self.edit_request_header_state()
				}
			}
			AppEvent::RequestHeadersMoveUp(_) => self.request_editor.headers_table.up(),
			AppEvent::RequestHeadersMoveDown(_) => self.request_editor.headers_table.down(),
			AppEvent::RequestHeadersMoveLeft(_) | AppEvent::RequestHeadersMoveRight(_) => {
				self.request_editor.headers_table.change_y()
			}
			AppEvent::CreateRequestHeader(_) => self.tui_create_new_header(),
			AppEvent::DeleteRequestHeader(_) => self.tui_delete_header(),
			AppEvent::ToggleRequestHeader(_) => self.tui_toggle_header(),
			AppEvent::DuplicateRequestHeader(_) => self.tui_duplicate_header(),

			/* Body */
			AppEvent::EditRequestBody(_) => match self.request_editor.body_form_table.is_selected()
			{
				true => self.edit_request_body_table_state(),
				false => self.edit_request_body_file_or_string_state(),
			},

			AppEvent::EditRequestMessage(_) => self.edit_request_message_state(),

			AppEvent::RequestBodyTableMoveUp(_) => self.request_editor.body_form_table.up(),
			AppEvent::RequestBodyTableMoveDown(_) => self.request_editor.body_form_table.down(),
			AppEvent::RequestBodyTableMoveLeft(_) | AppEvent::RequestBodyTableMoveRight(_) => {
				self.request_editor.body_form_table.change_y()
			}
			AppEvent::CreateRequestBodyTableElement(_) => self.tui_create_new_form_data(),
			AppEvent::DeleteRequestBodyTableElement(_) => self.tui_delete_form_data(),
			AppEvent::ToggleRequestBodyTableElement(_) => self.tui_toggle_form_data(),
			AppEvent::DuplicateRequestBodyTableElement(_) => self.tui_duplicate_form_data(),

			/* Scripts */
			AppEvent::EditRequestScript(_) => self.edit_request_script_state(),
			AppEvent::RequestScriptMove(_) => self.script_console.change_selection(),

			_ => unreachable!("handle_param_tabs_event called with non-param-tabs event"),
		}
	}
}
