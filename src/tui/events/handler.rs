use crokey::KeyCombination;
use ratatui::Terminal;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyEventKind};
use ratatui::prelude::CrosstermBackend;
use std::io::Stdout;
use tracing::debug;

use crate::app::app::App;
use crate::app::files::key_bindings::KEY_BINDINGS;
use crate::tui::app_states::AVAILABLE_EVENTS;
use crate::tui::events::AppEvent;
use crate::tui::utils::stateful::table_navigation::TableNavigation;

impl App<'_> {
	/// Handle events
	pub async fn handle_events(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
		// Refreshes the app every tick_rate
		let has_event = match event::poll(self.tick_rate) {
			Ok(has_event) => has_event,
			Err(_) => return,
		};
		if has_event {
			// Block while a key is pressed
			let event = match event::read() {
				Ok(event) => event,
				Err(_) => return,
			};
			if let Event::Key(key_event) = event {
				// We do not need
				if key_event.kind != KeyEventKind::Press {
					return;
				}

				let key = KeyCombination::from(key_event);
				let is_input_missed = self.handle_key(key, terminal).await;

				if !is_input_missed {
					debug!("Key pressed: {}", key);
				}
			}
		}

		let received_response = *self.core.received_response.lock();
		if received_response {
			self.tui_highlight_response_body_and_console();
			self.tui_refresh_result_scrollbars();

			if self.core.config.should_save_requests_response() {
				let selection = self.collections_tree.state.selected().to_vec();
				if !selection.is_empty() {
					self.save_collection_to_file(selection[0]);
				}
			}

			*self.core.received_response.lock() = false;
		}
	}

	async fn handle_key(
		&mut self,
		key: KeyCombination,
		terminal: &mut Terminal<CrosstermBackend<Stdout>>,
	) -> bool {
		{
			let key_bindings = KEY_BINDINGS.read();

			// Help is being displayed
			if self.should_display_help {
				match key {
					key if key == key_bindings.generic.navigation.go_back => {
						self.should_display_help = false
					}
					key if key == key_bindings.generic.navigation.move_cursor_left => {
						self.help_popup.previous_page()
					}
					key if key == key_bindings.generic.navigation.move_cursor_right => {
						self.help_popup.next_page()
					}

					_ => {}
				}

				// Avoid triggering other keys
				return false;
			}
			// Help is not being displayed
			else if key == key_bindings.generic.display_help && !self.in_input() {
				self.should_display_help = true;
				self.help_popup.selection = self.state;
				return false;
			}
		}

		let mut miss_input = false;

		// Find the matching event, clone it, and drop the read guard before the
		// match block. This avoids holding the AVAILABLE_EVENTS lock across
		// .await calls (e.g. tui_send_request, tui_send_request_message).
		let matching_event: Option<AppEvent> = {
			let available_app_events = AVAILABLE_EVENTS.read();

			let mut found = None;
			for possible_event in available_app_events.iter() {
				let event_key_bindings = possible_event.get_event_key_bindings();

				// Either the key is contained in the trigger condition list OR if the list is empty and no modifiers has been pressed, means 'any char'
				if event_key_bindings.keys.contains(&key) || event_key_bindings.keys.is_empty() {
					found = Some(possible_event.clone());
					break;
				}
			}
			found
		};
		// Read guard is dropped here — safe to await

		match matching_event.as_ref() {
			None => miss_input = true,
			Some(event) => match event {
				/* Main menu */
				AppEvent::ExitApp(_) => self.should_quit = true,

				AppEvent::MoveCollectionCursorUp(_) => self.collections_tree.up(),
				AppEvent::MoveCollectionCursorDown(_) => self.collections_tree.down(),

				AppEvent::SelectRequestOrExpandCollection(_) => {
					self.select_request_or_expand_collection()
				}
				AppEvent::ExpandCollection(_) => {
					self.collections_tree.state.toggle_selected();
				}
				AppEvent::UnselectRequest(_) => self.unselect_request(),

				AppEvent::CreateElement(_) => self.choose_element_to_create_state(),
				AppEvent::DeleteElement(_) => self.delete_element(),
				AppEvent::RenameElement(_) => self.rename_element(),
				AppEvent::DuplicateElement(_) => self.duplicate_element(),

				AppEvent::MoveElementUp(_) => self.tui_move_element_up(),
				AppEvent::MoveElementDown(_) => self.tui_move_element_down(),

				AppEvent::NextEnvironment(_) => self.tui_next_environment(),
				AppEvent::DisplayEnvEditor(_) => self.display_env_editor_state(),
				AppEvent::DisplayCookies(_) => self.display_cookies_state(),
				AppEvent::DisplayLogs(_) => self.display_logs_state(),
				AppEvent::DisplayThemePicker(_) => self.choose_theme_state(),

				AppEvent::GoBackToLastState(_) => match self.state {
					crate::tui::app_states::AppState::ChoosingTheme => {
						self.theme_popup.cancel();
						self.normal_state();
					}
					_ => self.normal_state(),
				},

				/* Env */
				AppEvent::EditEnvVariable(_) => {
					if self.env_editor_table.is_selected() {
						self.edit_env_variable_state()
					}
				}
				AppEvent::EnvVariablesMoveUp(_) => self.env_editor_table.up(),
				AppEvent::EnvVariablesMoveDown(_) => self.env_editor_table.down(),
				AppEvent::EnvVariablesMoveLeft(_) | AppEvent::EnvVariablesMoveRight(_) => {
					self.env_editor_table.change_y()
				}
				AppEvent::CreateEnvVariable(_) => self.tui_create_env_variable(),
				AppEvent::DeleteEnvVariable(_) => self.tui_delete_env_variable(),

				AppEvent::ModifyEnvVariable(_) => match self
					.env_editor_table
					.selection_text_input
					.is_in_default_mode()
				{
					true => self.tui_modify_env_variable(),
					false => self
						.env_editor_table
						.selection_text_input
						.key_event(key, None),
				},
				AppEvent::CancelModifyEnvVariable(_) => match self
					.env_editor_table
					.selection_text_input
					.is_in_default_mode()
				{
					true => self.display_env_editor_state(),
					false => self
						.env_editor_table
						.selection_text_input
						.key_event(key, None),
				},
				AppEvent::KeyEventModifyEnvVariable(_) => self
					.env_editor_table
					.selection_text_input
					.key_event(key, None),

				/* Cookies */
				AppEvent::CookiesMoveUp(_) => self.core.cookies_popup.cookies_table.up(),
				AppEvent::CookiesMoveDown(_) => self.core.cookies_popup.cookies_table.down(),
				AppEvent::CookiesMoveLeft(_) => self.core.cookies_popup.cookies_table.left(),
				AppEvent::CookiesMoveRight(_) => self.core.cookies_popup.cookies_table.right(),

				AppEvent::EditCookie(_) => {
					if self.core.cookies_popup.cookies_table.selection.is_some() {
						self.edit_cookie_state()
					}
				}
				AppEvent::CreateCookie(_) => self.tui_create_cookie(),
				AppEvent::DeleteCookie(_) => self.tui_delete_cookie(),

				AppEvent::ModifyCookie(_) => match self
					.core
					.cookies_popup
					.cookies_table
					.selection_text_input
					.is_in_default_mode()
				{
					true => self.tui_modify_cookie(),
					false => self
						.core
						.cookies_popup
						.cookies_table
						.selection_text_input
						.key_event(key, None),
				},
				AppEvent::CancelEditCookie(_) => match self
					.core
					.cookies_popup
					.cookies_table
					.selection_text_input
					.is_in_default_mode()
				{
					true => self.display_cookies_state(),
					false => self
						.core
						.cookies_popup
						.cookies_table
						.selection_text_input
						.key_event(key, None),
				},
				AppEvent::KeyEventEditCookie(_) => self
					.core
					.cookies_popup
					.cookies_table
					.selection_text_input
					.key_event(key, None),

				/* Logs */
				AppEvent::ScrollLogsUp(_) => self.logs_vertical_scrollbar.page_up(),
				AppEvent::ScrollLogsDown(_) => self.logs_vertical_scrollbar.page_down(),
				AppEvent::ScrollLogsLeft(_) => self.logs_horizontal_scrollbar.page_up(),
				AppEvent::ScrollLogsRight(_) => self.logs_horizontal_scrollbar.page_down(),

				/* Collections */
				AppEvent::ChooseElementToCreateMoveCursorLeft(_) => self.creation_popup.previous(),
				AppEvent::ChooseElementToCreateMoveCursorRight(_) => self.creation_popup.next(),
				AppEvent::SelectElementToCreate(_) => self.new_element(),

				AppEvent::CreateNewCollection(_) => {
					match self.new_collection_input.is_in_default_mode() {
						true => self.tui_new_collection(),
						false => self.new_collection_input.key_event(key, None),
					}
				}
				AppEvent::CancelCreateNewCollection(_) => {
					match self.new_collection_input.is_in_default_mode() {
						true => self.normal_state(),
						false => self.new_collection_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventCreateNewCollection(_) => {
					self.new_collection_input.key_event(key, None)
				}

				AppEvent::CreateNewRequest(_) => {
					match self.new_request_popup.text_input.is_in_default_mode() {
						true => self.tui_new_request(),
						false => self.new_request_popup.text_input.key_event(key, None),
					}
				}
				AppEvent::CancelCreateNewRequest(_) => {
					match self.new_request_popup.text_input.is_in_default_mode() {
						true => self.normal_state(),
						false => self.new_request_popup.text_input.key_event(key, None),
					}
				}
				AppEvent::CreatingRequestSelectInputUp(_) => {
					self.new_request_popup.previous_input()
				}
				AppEvent::CreatingRequestSelectInputDown(_) => self.new_request_popup.next_input(),
				AppEvent::CreatingRequestInputLeft(_) => self.new_request_popup.input_left(),
				AppEvent::CreatingRequestInputRight(_) => self.new_request_popup.input_right(),
				AppEvent::KeyEventCreateNewRequest(_) => {
					self.new_request_popup.text_input.key_event(key, None)
				}

				AppEvent::DeletingCollectionMoveCursorLeft(_) => {
					self.delete_collection_popup.change_state()
				}
				AppEvent::DeletingCollectionMoveCursorRight(_) => {
					self.delete_collection_popup.change_state()
				}
				AppEvent::DeleteCollection(_) => match self.delete_collection_popup.state {
					true => self.tui_delete_collection(),
					false => self.normal_state(),
				},

				AppEvent::DeletingRequestMoveCursorLeft(_) => {
					self.delete_request_popup.change_state()
				}
				AppEvent::DeletingRequestMoveCursorRight(_) => {
					self.delete_request_popup.change_state()
				}
				AppEvent::DeleteRequest(_) => match self.delete_request_popup.state {
					true => self.tui_delete_request(),
					false => self.normal_state(),
				},

				AppEvent::RenameCollection(_) => {
					match self.rename_collection_input.is_in_default_mode() {
						true => self.tui_rename_collection(),
						false => self.rename_collection_input.key_event(key, None),
					}
				}
				AppEvent::CancelRenameCollection(_) => {
					match self.rename_collection_input.is_in_default_mode() {
						true => self.normal_state(),
						false => self.rename_collection_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventRenameCollection(_) => {
					self.rename_collection_input.key_event(key, None)
				}

				AppEvent::RenameRequest(_) => {
					match self.rename_request_input.is_in_default_mode() {
						true => self.tui_rename_request(),
						false => self.rename_request_input.key_event(key, None),
					}
				}
				AppEvent::CancelRenameRequest(_) => {
					match self.rename_request_input.is_in_default_mode() {
						true => self.normal_state(),
						false => self.rename_request_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventRenameRequest(_) => {
					self.rename_request_input.key_event(key, None)
				}

				/* Folders */
				AppEvent::CreateNewFolder(_) => {
					match self.new_collection_input.is_in_default_mode() {
						true => self.tui_new_folder(),
						false => self.new_collection_input.key_event(key, None),
					}
				}
				AppEvent::CancelCreateNewFolder(_) => {
					match self.new_collection_input.is_in_default_mode() {
						true => self.normal_state(),
						false => self.new_collection_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventCreateNewFolder(_) => {
					self.new_collection_input.key_event(key, None)
				}

				AppEvent::DeletingFolderMoveCursorLeft(_) => {
					self.delete_collection_popup.change_state()
				}
				AppEvent::DeletingFolderMoveCursorRight(_) => {
					self.delete_collection_popup.change_state()
				}
				AppEvent::DeleteFolder(_) => match self.delete_collection_popup.state {
					true => self.tui_delete_folder(),
					false => self.normal_state(),
				},

				AppEvent::RenameFolder(_) => {
					match self.rename_collection_input.is_in_default_mode() {
						true => self.tui_rename_folder(),
						false => self.rename_collection_input.key_event(key, None),
					}
				}
				AppEvent::CancelRenameFolder(_) => {
					match self.rename_collection_input.is_in_default_mode() {
						true => self.normal_state(),
						false => self.rename_collection_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventRenameFolder(_) => {
					self.rename_collection_input.key_event(key, None)
				}

				/* Selected Request */
				AppEvent::GoBackToRequestMenu(_) => self.select_request_state(),

				AppEvent::EditUrl(_) => self.edit_request_url_state(),
				AppEvent::EditMethod(_) => self.tui_next_request_method(),
				AppEvent::EditSettings(_) => self.edit_request_settings_state(),

				AppEvent::NextView(_) => self.next_request_view(),
				AppEvent::SendRequest(_) => self.tui_send_request().await,

				/* Param tabs */
				AppEvent::NextParamTab(_) => self.tui_next_request_param_tab(),

				AppEvent::ModifyRequestAuthMethod(_) => self.tui_next_request_auth(),
				AppEvent::ModifyRequestBodyContentType(_) => self.tui_next_request_content_type(),
				AppEvent::ModifyRequestMessageType(_) => self.tui_next_request_message_type(),

				AppEvent::EditRequestQueryParam(_) => {
					if self.query_params_table.is_selected() {
						self.edit_request_param_state()
					}
				}
				AppEvent::RequestQueryParamsMoveUp(_) => self.query_params_table.up(),
				AppEvent::RequestQueryParamsMoveDown(_) => self.query_params_table.down(),
				AppEvent::RequestQueryParamsMoveLeft(_)
				| AppEvent::RequestQueryParamsMoveRight(_) => self.query_params_table.change_y(),
				AppEvent::CreateRequestQueryParam(_) => self.tui_create_new_query_param(),
				AppEvent::DeleteRequestQueryParam(_) => self.tui_delete_query_param(),
				AppEvent::ToggleRequestQueryParam(_) => self.tui_toggle_query_param(),
				AppEvent::DuplicateRequestQueryParam(_) => self.tui_duplicate_query_param(),

				AppEvent::EditRequestAuth(_) => {
					if self.auth_text_input_selection.usable {
						self.tui_select_request_auth_input_text()
					}
				}
				AppEvent::RequestAuthMoveUp(_) => {
					if self.auth_text_input_selection.usable {
						self.auth_text_input_selection.previous()
					}
				}
				AppEvent::RequestAuthMoveDown(_) => {
					if self.auth_text_input_selection.usable {
						self.auth_text_input_selection.next()
					}
				}
				AppEvent::RequestAuthMoveLeft(_) => self.tui_request_auth_move_left(),
				AppEvent::RequestAuthMoveRight(_) => self.tui_request_auth_move_right(),

				AppEvent::EditRequestHeader(_) => {
					if self.headers_table.is_selected() {
						self.edit_request_header_state()
					}
				}
				AppEvent::RequestHeadersMoveUp(_) => self.headers_table.up(),
				AppEvent::RequestHeadersMoveDown(_) => self.headers_table.down(),
				AppEvent::RequestHeadersMoveLeft(_) | AppEvent::RequestHeadersMoveRight(_) => {
					self.headers_table.change_y()
				}
				AppEvent::CreateRequestHeader(_) => self.tui_create_new_header(),
				AppEvent::DeleteRequestHeader(_) => self.tui_delete_header(),
				AppEvent::ToggleRequestHeader(_) => self.tui_toggle_header(),
				AppEvent::DuplicateRequestHeader(_) => self.tui_duplicate_header(),

				AppEvent::EditRequestBody(_) => match self.body_form_table.is_selected() {
					true => self.edit_request_body_table_state(),
					false => self.edit_request_body_file_or_string_state(),
				},

				AppEvent::EditRequestMessage(_) => self.edit_request_message_state(),

				AppEvent::RequestBodyTableMoveUp(_) => self.body_form_table.up(),
				AppEvent::RequestBodyTableMoveDown(_) => self.body_form_table.down(),
				AppEvent::RequestBodyTableMoveLeft(_) | AppEvent::RequestBodyTableMoveRight(_) => {
					self.body_form_table.change_y()
				}
				AppEvent::CreateRequestBodyTableElement(_) => self.tui_create_new_form_data(),
				AppEvent::DeleteRequestBodyTableElement(_) => self.tui_delete_form_data(),
				AppEvent::ToggleRequestBodyTableElement(_) => self.tui_toggle_form_data(),
				AppEvent::DuplicateRequestBodyTableElement(_) => self.tui_duplicate_form_data(),

				/* Scripts */
				AppEvent::EditRequestScript(_) => self.edit_request_script_state(),
				AppEvent::RequestScriptMove(_) => self.script_console.change_selection(),

				/* Result tabs */
				AppEvent::NextResultTab(_) => self.tui_next_request_result_tab(),

				AppEvent::ScrollResultUp(_) => self.result_vertical_scrollbar.page_up(),
				AppEvent::ScrollResultDown(_) => self.result_vertical_scrollbar.page_down(),
				AppEvent::ScrollResultLeft(_) => self.result_horizontal_scrollbar.page_up(),
				AppEvent::ScrollResultRight(_) => self.result_horizontal_scrollbar.page_down(),

				/* Others */
				#[cfg(feature = "clipboard")]
				AppEvent::CopyResponsePart(_) => self.copy_response_body_content_to_clipboard(),

				#[cfg(not(feature = "clipboard"))]
				AppEvent::CopyResponsePart(_) => {}

				/* Request Export */
				AppEvent::ExportRequest(_) => self.choose_request_export_format_state(),

				AppEvent::RequestExportFormatMoveCursorLeft(_) => self.export_request.previous(),
				AppEvent::RequestExportFormatMoveCursorRight(_) => self.export_request.next(),

				AppEvent::SelectRequestExportFormat(_) => self.tui_export_request(),

				AppEvent::ScrollRequestExportUp(_) => {
					self.display_request_export.vertical_scrollbar.page_up()
				}
				AppEvent::ScrollRequestExportDown(_) => {
					self.display_request_export.vertical_scrollbar.page_down()
				}
				AppEvent::ScrollRequestExportLeft(_) => {
					self.display_request_export.horizontal_scrollbar.page_up()
				}
				AppEvent::ScrollRequestExportRight(_) => {
					self.display_request_export.horizontal_scrollbar.page_down()
				}

				#[cfg(feature = "clipboard")]
				AppEvent::CopyRequestExport(_) => self.copy_request_export_to_clipboard(),

				#[cfg(not(feature = "clipboard"))]
				AppEvent::CopyRequestExport(_) => {}

				/* Url */
				AppEvent::ModifyRequestUrl(_) => match self.url_text_input.is_in_default_mode() {
					true => self.tui_modify_request_url(),
					false => self.url_text_input.key_event(key, None),
				},
				AppEvent::CancelEditRequestUrl(_) => match self.url_text_input.is_in_default_mode()
				{
					true => self.select_request_state(),
					false => self.url_text_input.key_event(key, None),
				},
				AppEvent::KeyEventEditRequestUrl(_) => self.url_text_input.key_event(key, None),

				/* Query params */
				AppEvent::ModifyRequestQueryParam(_) => match self
					.query_params_table
					.selection_text_input
					.is_in_default_mode()
				{
					true => self.tui_modify_request_query_param(),
					false => self
						.query_params_table
						.selection_text_input
						.key_event(key, None),
				},
				AppEvent::CancelEditRequestQueryParam(_) => match self
					.query_params_table
					.selection_text_input
					.is_in_default_mode()
				{
					true => self.select_request_state(),
					false => self
						.query_params_table
						.selection_text_input
						.key_event(key, None),
				},
				AppEvent::KeyEventEditRequestQueryParam(_) => self
					.query_params_table
					.selection_text_input
					.key_event(key, None),

				/* Auth */
				// self.auth_text_input_selection.usable
				AppEvent::ModifyRequestAuthBasicUsername(_) => {
					match self.auth_basic_username_text_input.is_in_default_mode() {
						true => self.tui_modify_request_auth_basic_username(),
						false => self.auth_basic_username_text_input.key_event(key, None),
					}
				}
				AppEvent::CancelEditRequestAuthBasicUsername(_) => {
					match self.auth_basic_password_text_input.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.auth_basic_password_text_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventEditRequestAuthBasicUsername(_) => {
					self.auth_basic_password_text_input.key_event(key, None)
				}

				AppEvent::ModifyRequestAuthBasicPassword(_) => {
					match self.auth_basic_password_text_input.is_in_default_mode() {
						true => self.tui_modify_request_auth_basic_password(),
						false => self.auth_basic_password_text_input.key_event(key, None),
					}
				}
				AppEvent::CancelEditRequestAuthBasicPassword(_) => {
					match self.auth_basic_password_text_input.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.auth_basic_password_text_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventEditRequestAuthBasicPassword(_) => {
					self.auth_digest_nonce_text_input.key_event(key, None)
				}

				AppEvent::ModifyRequestAuthBearerToken(_) => {
					match self.auth_bearer_token_text_input.is_in_default_mode() {
						true => self.tui_modify_request_auth_bearer_token(),
						false => self.auth_bearer_token_text_input.key_event(key, None),
					}
				}
				AppEvent::CancelEditRequestAuthBearerToken(_) => {
					match self.auth_bearer_token_text_input.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.auth_bearer_token_text_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventEditRequestAuthBearerToken(_) => {
					self.auth_bearer_token_text_input.key_event(key, None)
				}

				AppEvent::ModifyRequestAuthJwtSecret(_) => {
					match self.auth_jwt_secret_text_input.is_in_default_mode() {
						true => self.tui_modify_request_auth_jwt_secret(),
						false => self.auth_jwt_secret_text_input.key_event(key, None),
					}
				}
				AppEvent::CancelEditRequestAuthJwtSecret(_) => {
					match self.auth_jwt_secret_text_input.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.auth_jwt_secret_text_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventEditRequestAuthJwtSecret(_) => {
					self.auth_jwt_secret_text_input.key_event(key, None)
				}

				AppEvent::ModifyRequestAuthJwtPayload(_) => {
					match self.auth_jwt_payload_text_area.is_in_default_mode() {
						true => self.tui_modify_request_auth_jwt_payload(),
						false => self
							.auth_jwt_payload_text_area
							.key_event(key, Some(terminal)),
					}
				}
				AppEvent::CancelEditRequestAuthJwtPayload(_) => {
					match self.auth_jwt_payload_text_area.is_in_default_mode() {
						true => self.select_request_state(),
						false => self
							.auth_jwt_payload_text_area
							.key_event(key, Some(terminal)),
					}
				}
				AppEvent::KeyEventEditRequestAuthJwtPayload(_) => self
					.auth_jwt_payload_text_area
					.key_event(key, Some(terminal)),

				AppEvent::ModifyRequestAuthDigestUsername(_) => {
					match self.auth_digest_username_text_input.is_in_default_mode() {
						true => self.tui_modify_request_auth_digest_username(),
						false => self.auth_digest_username_text_input.key_event(key, None),
					}
				}
				AppEvent::CancelEditRequestAuthDigestUsername(_) => {
					match self.auth_digest_username_text_input.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.auth_digest_username_text_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventEditRequestAuthDigestUsername(_) => {
					self.auth_digest_username_text_input.key_event(key, None)
				}

				AppEvent::ModifyRequestAuthDigestPassword(_) => {
					match self.auth_digest_password_text_input.is_in_default_mode() {
						true => self.tui_modify_request_auth_digest_password(),
						false => self.auth_digest_password_text_input.key_event(key, None),
					}
				}
				AppEvent::CancelEditRequestAuthDigestPassword(_) => {
					match self.auth_digest_password_text_input.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.auth_digest_password_text_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventEditRequestAuthDigestPassword(_) => {
					self.auth_digest_password_text_input.key_event(key, None)
				}

				AppEvent::ModifyRequestAuthDigestDomains(_) => {
					match self.auth_digest_domains_text_input.is_in_default_mode() {
						true => self.tui_modify_request_auth_digest_domains(),
						false => self.auth_digest_domains_text_input.key_event(key, None),
					}
				}
				AppEvent::CancelEditRequestAuthDigestDomains(_) => {
					match self.auth_digest_domains_text_input.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.auth_digest_domains_text_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventEditRequestAuthDigestDomains(_) => {
					self.auth_digest_domains_text_input.key_event(key, None)
				}

				AppEvent::ModifyRequestAuthDigestRealm(_) => {
					match self.auth_digest_realm_text_input.is_in_default_mode() {
						true => self.tui_modify_request_auth_digest_realm(),
						false => self.auth_digest_realm_text_input.key_event(key, None),
					}
				}
				AppEvent::CancelEditRequestAuthDigestRealm(_) => {
					match self.auth_digest_realm_text_input.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.auth_digest_realm_text_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventEditRequestAuthDigestRealm(_) => {
					self.auth_digest_realm_text_input.key_event(key, None)
				}

				AppEvent::ModifyRequestAuthDigestNonce(_) => {
					match self.auth_digest_nonce_text_input.is_in_default_mode() {
						true => self.tui_modify_request_auth_digest_nonce(),
						false => self.auth_digest_nonce_text_input.key_event(key, None),
					}
				}
				AppEvent::CancelEditRequestAuthDigestNonce(_) => {
					match self.auth_digest_nonce_text_input.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.auth_digest_nonce_text_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventEditRequestAuthDigestNonce(_) => {
					self.auth_digest_nonce_text_input.key_event(key, None)
				}

				AppEvent::ModifyRequestAuthDigestOpaque(_) => {
					match self.auth_digest_opaque_text_input.is_in_default_mode() {
						true => self.tui_modify_request_auth_digest_opaque(),
						false => self.auth_digest_opaque_text_input.key_event(key, None),
					}
				}
				AppEvent::CancelEditRequestAuthDigestOpaque(_) => {
					match self.auth_digest_opaque_text_input.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.auth_digest_opaque_text_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventEditRequestAuthDigestOpaque(_) => {
					self.auth_digest_opaque_text_input.key_event(key, None)
				}

				/* Header */
				AppEvent::ModifyRequestHeader(_) => {
					match self.headers_table.selection_text_input.is_in_default_mode() {
						true => self.tui_modify_request_header(),
						false => self.headers_table.selection_text_input.key_event(key, None),
					}
				}
				AppEvent::CancelEditRequestHeader(_) => {
					match self.headers_table.selection_text_input.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.headers_table.selection_text_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventEditRequestHeader(_) => {
					self.headers_table.selection_text_input.key_event(key, None)
				}

				/* Body */
				AppEvent::ModifyRequestBodyTable(_) => match self
					.body_form_table
					.selection_text_input
					.is_in_default_mode()
				{
					true => self.tui_modify_request_form_data(),
					false => self
						.body_form_table
						.selection_text_input
						.key_event(key, None),
				},
				AppEvent::CancelEditRequestBodyTable(_) => match self
					.body_form_table
					.selection_text_input
					.is_in_default_mode()
				{
					true => self.select_request_state(),
					false => self
						.body_form_table
						.selection_text_input
						.key_event(key, None),
				},
				AppEvent::KeyEventEditRequestBodyTable(_) => self
					.body_form_table
					.selection_text_input
					.key_event(key, None),

				AppEvent::ModifyRequestBodyFile(_) => {
					match self.body_file_text_input.is_in_default_mode() {
						true => self.tui_modify_request_body(),
						false => self.body_file_text_input.key_event(key, None),
					}
				}
				AppEvent::CancelEditRequestBodyFile(_) => {
					match self.body_file_text_input.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.body_file_text_input.key_event(key, None),
					}
				}
				AppEvent::KeyEventEditRequestBodyFile(_) => {
					self.body_file_text_input.key_event(key, None)
				}

				AppEvent::ModifyRequestBodyString(_) => {
					match self.body_text_area.is_in_default_mode() {
						true => self.tui_modify_request_body(),
						false => self.body_text_area.key_event(key, Some(terminal)),
					}
				}
				AppEvent::CancelEditRequestBodyString(_) => {
					match self.body_text_area.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.body_text_area.key_event(key, Some(terminal)),
					}
				}
				AppEvent::KeyEventEditRequestBodyString(_) => {
					self.body_text_area.key_event(key, Some(terminal))
				}

				/* Websocket */
				AppEvent::ModifyRequestMessage(_) => {
					match self.message_text_area.is_in_default_mode() {
						true => self.tui_send_request_message().await,
						false => self.message_text_area.key_event(key, Some(terminal)),
					}
				}
				AppEvent::CancelEditRequestMessage(_) => {
					match self.message_text_area.is_in_default_mode() {
						true => self.select_request_state(),
						false => self.message_text_area.key_event(key, Some(terminal)),
					}
				}
				AppEvent::KeyEventEditRequestMessage(_) => {
					self.message_text_area.key_event(key, Some(terminal))
				}

				/* Scripts */
				AppEvent::ModifyRequestPreRequestScript(_) => match self
					.script_console
					.pre_request_text_area
					.is_in_default_mode()
				{
					true => self.tui_modify_pre_request_script(),
					false => self
						.script_console
						.pre_request_text_area
						.key_event(key, Some(terminal)),
				},
				AppEvent::CancelEditRequestPreRequestScript(_) => match self
					.script_console
					.pre_request_text_area
					.is_in_default_mode()
				{
					true => self.select_request_state(),
					false => self
						.script_console
						.pre_request_text_area
						.key_event(key, Some(terminal)),
				},
				AppEvent::KeyEventEditRequestPreRequestScript(_) => self
					.script_console
					.pre_request_text_area
					.key_event(key, Some(terminal)),

				AppEvent::ModifyRequestPostRequestScript(_) => match self
					.script_console
					.post_request_text_area
					.is_in_default_mode()
				{
					true => self.tui_modify_post_request_script(),
					false => self
						.script_console
						.post_request_text_area
						.key_event(key, None),
				},
				AppEvent::CancelEditRequestPostRequestScript(_) => match self
					.script_console
					.post_request_text_area
					.is_in_default_mode()
				{
					true => self.select_request_state(),
					false => self
						.script_console
						.post_request_text_area
						.key_event(key, None),
				},
				AppEvent::KeyEventEditRequestPostRequestScript(_) => self
					.script_console
					.post_request_text_area
					.key_event(key, None),

				/* Settings */
				AppEvent::RequestSettingsMoveUp(_) => self.request_settings_popup.previous(),
				AppEvent::RequestSettingsMoveDown(_) => self.request_settings_popup.next(),
				AppEvent::RequestSettingsToggleSettingLeft(_) => {
					self.request_settings_popup.toggle_setting_left()
				}
				AppEvent::RequestSettingsToggleSettingRight(_) => {
					self.request_settings_popup.toggle_setting_right()
				}
				AppEvent::ModifyRequestSettings(_) => self.tui_modify_request_settings(),

				/* Response Body Selection */
				AppEvent::EnterResponseBodySelection(_) => {
					debug!("EnterResponseBodySelection event triggered");
					self.select_response_body_state();
				}
				AppEvent::ExitResponseBodySelection(_) => {
					match self.response_body_text_area.is_in_default_mode() {
						true => self.exit_response_body_selection_state(),
						false => self.response_body_text_area.key_event(key, Some(terminal)),
					}
				}
				AppEvent::KeyEventSelectResponseBody(_) => {
					self.response_body_text_area.key_event(key, Some(terminal))
				}

				/* Theme Picker */
				AppEvent::ThemePickerMoveUp(_) => {
					debug!("ThemePickerMoveUp event triggered");
					self.theme_popup.previous();
				}
				AppEvent::ThemePickerMoveDown(_) => {
					debug!("ThemePickerMoveDown event triggered");
					self.theme_popup.next();
				}
				AppEvent::ThemePickerConfirm(_) => {
					debug!("ThemePickerConfirm event triggered");
					if let Some(theme_name) = self.theme_popup.confirm() {
						self.save_theme_to_global_config(&theme_name);
					}
					self.go_back_to_last_state();
				}

				/* Others */
				AppEvent::Documentation(_) => {}
			},
		};

		miss_input
	}
}
