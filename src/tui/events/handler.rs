use crokey::KeyCombination;
use ratatui::Terminal;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyEventKind};
use ratatui::prelude::CrosstermBackend;
use std::io::Stdout;
use tracing::debug;

use crate::app::App;
use crate::app::files::key_bindings::KEY_BINDINGS;
use crate::tui::app_states::AVAILABLE_EVENTS;
use crate::tui::events::AppEvent;

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
				AppEvent::ExitApp(_)
				| AppEvent::MoveCollectionCursorUp(_)
				| AppEvent::MoveCollectionCursorDown(_)
				| AppEvent::SelectRequestOrExpandCollection(_)
				| AppEvent::ExpandCollection(_)
				| AppEvent::UnselectRequest(_)
				| AppEvent::CreateElement(_)
				| AppEvent::DeleteElement(_)
				| AppEvent::RenameElement(_)
				| AppEvent::DuplicateElement(_)
				| AppEvent::MoveElementUp(_)
				| AppEvent::MoveElementDown(_)
				| AppEvent::NextEnvironment(_)
				| AppEvent::DisplayEnvEditor(_)
				| AppEvent::DisplayCookies(_)
				| AppEvent::DisplayLogs(_)
				| AppEvent::DisplayThemePicker(_)
				| AppEvent::GoBackToLastState(_) => self.handle_main_menu_event(event, key),

				/* Env editor */
				AppEvent::EditEnvVariable(_)
				| AppEvent::EnvVariablesMoveUp(_)
				| AppEvent::EnvVariablesMoveDown(_)
				| AppEvent::EnvVariablesMoveLeft(_)
				| AppEvent::EnvVariablesMoveRight(_)
				| AppEvent::CreateEnvVariable(_)
				| AppEvent::DeleteEnvVariable(_)
				| AppEvent::ModifyEnvVariable(_)
				| AppEvent::CancelModifyEnvVariable(_)
				| AppEvent::KeyEventModifyEnvVariable(_) => self.handle_env_editor_event(event, key),

				/* Cookies */
				AppEvent::CookiesMoveUp(_)
				| AppEvent::CookiesMoveDown(_)
				| AppEvent::CookiesMoveLeft(_)
				| AppEvent::CookiesMoveRight(_)
				| AppEvent::EditCookie(_)
				| AppEvent::CreateCookie(_)
				| AppEvent::DeleteCookie(_)
				| AppEvent::ModifyCookie(_)
				| AppEvent::CancelEditCookie(_)
				| AppEvent::KeyEventEditCookie(_) => self.handle_cookies_event(event, key),

				/* Logs */
				AppEvent::ScrollLogsUp(_)
				| AppEvent::ScrollLogsDown(_)
				| AppEvent::ScrollLogsLeft(_)
				| AppEvent::ScrollLogsRight(_) => self.handle_logs_event(event, key),

				/* Collections */
				AppEvent::ChooseElementToCreateMoveCursorLeft(_)
				| AppEvent::ChooseElementToCreateMoveCursorRight(_)
				| AppEvent::SelectElementToCreate(_)
				| AppEvent::CreateNewCollection(_)
				| AppEvent::CancelCreateNewCollection(_)
				| AppEvent::KeyEventCreateNewCollection(_)
				| AppEvent::CreateNewRequest(_)
				| AppEvent::CancelCreateNewRequest(_)
				| AppEvent::CreatingRequestSelectInputUp(_)
				| AppEvent::CreatingRequestSelectInputDown(_)
				| AppEvent::CreatingRequestInputLeft(_)
				| AppEvent::CreatingRequestInputRight(_)
				| AppEvent::KeyEventCreateNewRequest(_)
				| AppEvent::DeletingCollectionMoveCursorLeft(_)
				| AppEvent::DeletingCollectionMoveCursorRight(_)
				| AppEvent::DeleteCollection(_)
				| AppEvent::DeletingRequestMoveCursorLeft(_)
				| AppEvent::DeletingRequestMoveCursorRight(_)
				| AppEvent::DeleteRequest(_)
				| AppEvent::RenameCollection(_)
				| AppEvent::CancelRenameCollection(_)
				| AppEvent::KeyEventRenameCollection(_)
				| AppEvent::RenameRequest(_)
				| AppEvent::CancelRenameRequest(_)
				| AppEvent::KeyEventRenameRequest(_) => self.handle_collections_event(event, key),

				/* Folders */
				AppEvent::CreateNewFolder(_)
				| AppEvent::CancelCreateNewFolder(_)
				| AppEvent::KeyEventCreateNewFolder(_)
				| AppEvent::DeletingFolderMoveCursorLeft(_)
				| AppEvent::DeletingFolderMoveCursorRight(_)
				| AppEvent::DeleteFolder(_)
				| AppEvent::RenameFolder(_)
				| AppEvent::CancelRenameFolder(_)
				| AppEvent::KeyEventRenameFolder(_) => self.handle_folders_event(event, key),

				/* Selected request (async — contains tui_send_request) */
				AppEvent::GoBackToRequestMenu(_)
				| AppEvent::EditUrl(_)
				| AppEvent::EditMethod(_)
				| AppEvent::EditSettings(_)
				| AppEvent::NextView(_)
				| AppEvent::SendRequest(_) => self.handle_selected_request_event(event, key).await,

				/* Param tabs */
				AppEvent::NextParamTab(_)
				| AppEvent::ModifyRequestAuthMethod(_)
				| AppEvent::ModifyRequestBodyContentType(_)
				| AppEvent::ModifyRequestMessageType(_)
				| AppEvent::EditRequestQueryParam(_)
				| AppEvent::RequestQueryParamsMoveUp(_)
				| AppEvent::RequestQueryParamsMoveDown(_)
				| AppEvent::RequestQueryParamsMoveLeft(_)
				| AppEvent::RequestQueryParamsMoveRight(_)
				| AppEvent::CreateRequestQueryParam(_)
				| AppEvent::DeleteRequestQueryParam(_)
				| AppEvent::ToggleRequestQueryParam(_)
				| AppEvent::DuplicateRequestQueryParam(_)
				| AppEvent::EditRequestAuth(_)
				| AppEvent::RequestAuthMoveUp(_)
				| AppEvent::RequestAuthMoveDown(_)
				| AppEvent::RequestAuthMoveLeft(_)
				| AppEvent::RequestAuthMoveRight(_)
				| AppEvent::EditRequestHeader(_)
				| AppEvent::RequestHeadersMoveUp(_)
				| AppEvent::RequestHeadersMoveDown(_)
				| AppEvent::RequestHeadersMoveLeft(_)
				| AppEvent::RequestHeadersMoveRight(_)
				| AppEvent::CreateRequestHeader(_)
				| AppEvent::DeleteRequestHeader(_)
				| AppEvent::ToggleRequestHeader(_)
				| AppEvent::DuplicateRequestHeader(_)
				| AppEvent::EditRequestBody(_)
				| AppEvent::EditRequestMessage(_)
				| AppEvent::EditGraphqlQuery(_)
				| AppEvent::EditGraphqlVariables(_)
				| AppEvent::RequestBodyTableMoveUp(_)
				| AppEvent::RequestBodyTableMoveDown(_)
				| AppEvent::RequestBodyTableMoveLeft(_)
				| AppEvent::RequestBodyTableMoveRight(_)
				| AppEvent::CreateRequestBodyTableElement(_)
				| AppEvent::DeleteRequestBodyTableElement(_)
				| AppEvent::ToggleRequestBodyTableElement(_)
				| AppEvent::DuplicateRequestBodyTableElement(_)
				| AppEvent::EditRequestScript(_)
				| AppEvent::RequestScriptMove(_) => self.handle_param_tabs_event(event, key),

				/* Result tabs */
				AppEvent::NextResultTab(_)
				| AppEvent::ScrollResultUp(_)
				| AppEvent::ScrollResultDown(_)
				| AppEvent::ScrollResultLeft(_)
				| AppEvent::ScrollResultRight(_)
				| AppEvent::CopyResponsePart(_) => self.handle_result_tabs_event(event, key),

				/* Text input editing (async — contains tui_send_request_message) */
				AppEvent::ModifyRequestUrl(_)
				| AppEvent::CancelEditRequestUrl(_)
				| AppEvent::KeyEventEditRequestUrl(_)
				| AppEvent::ModifyRequestQueryParam(_)
				| AppEvent::CancelEditRequestQueryParam(_)
				| AppEvent::KeyEventEditRequestQueryParam(_)
				| AppEvent::ModifyRequestAuthBasicUsername(_)
				| AppEvent::CancelEditRequestAuthBasicUsername(_)
				| AppEvent::KeyEventEditRequestAuthBasicUsername(_)
				| AppEvent::ModifyRequestAuthBasicPassword(_)
				| AppEvent::CancelEditRequestAuthBasicPassword(_)
				| AppEvent::KeyEventEditRequestAuthBasicPassword(_)
				| AppEvent::ModifyRequestAuthBearerToken(_)
				| AppEvent::CancelEditRequestAuthBearerToken(_)
				| AppEvent::KeyEventEditRequestAuthBearerToken(_)
				| AppEvent::ModifyRequestAuthJwtSecret(_)
				| AppEvent::CancelEditRequestAuthJwtSecret(_)
				| AppEvent::KeyEventEditRequestAuthJwtSecret(_)
				| AppEvent::ModifyRequestAuthJwtPayload(_)
				| AppEvent::CancelEditRequestAuthJwtPayload(_)
				| AppEvent::KeyEventEditRequestAuthJwtPayload(_)
				| AppEvent::ModifyRequestAuthDigestUsername(_)
				| AppEvent::CancelEditRequestAuthDigestUsername(_)
				| AppEvent::KeyEventEditRequestAuthDigestUsername(_)
				| AppEvent::ModifyRequestAuthDigestPassword(_)
				| AppEvent::CancelEditRequestAuthDigestPassword(_)
				| AppEvent::KeyEventEditRequestAuthDigestPassword(_)
				| AppEvent::ModifyRequestAuthDigestDomains(_)
				| AppEvent::CancelEditRequestAuthDigestDomains(_)
				| AppEvent::KeyEventEditRequestAuthDigestDomains(_)
				| AppEvent::ModifyRequestAuthDigestRealm(_)
				| AppEvent::CancelEditRequestAuthDigestRealm(_)
				| AppEvent::KeyEventEditRequestAuthDigestRealm(_)
				| AppEvent::ModifyRequestAuthDigestNonce(_)
				| AppEvent::CancelEditRequestAuthDigestNonce(_)
				| AppEvent::KeyEventEditRequestAuthDigestNonce(_)
				| AppEvent::ModifyRequestAuthDigestOpaque(_)
				| AppEvent::CancelEditRequestAuthDigestOpaque(_)
				| AppEvent::KeyEventEditRequestAuthDigestOpaque(_)
				| AppEvent::ModifyRequestHeader(_)
				| AppEvent::CancelEditRequestHeader(_)
				| AppEvent::KeyEventEditRequestHeader(_)
				| AppEvent::ModifyRequestBodyTable(_)
				| AppEvent::CancelEditRequestBodyTable(_)
				| AppEvent::KeyEventEditRequestBodyTable(_)
				| AppEvent::ModifyRequestBodyFile(_)
				| AppEvent::CancelEditRequestBodyFile(_)
				| AppEvent::KeyEventEditRequestBodyFile(_)
				| AppEvent::ModifyRequestBodyString(_)
				| AppEvent::CancelEditRequestBodyString(_)
				| AppEvent::KeyEventEditRequestBodyString(_)
				| AppEvent::ModifyRequestMessage(_)
				| AppEvent::CancelEditRequestMessage(_)
				| AppEvent::KeyEventEditRequestMessage(_)
				| AppEvent::ModifyGraphqlQuery(_)
				| AppEvent::CancelEditGraphqlQuery(_)
				| AppEvent::KeyEventEditGraphqlQuery(_)
				| AppEvent::ModifyGraphqlVariables(_)
				| AppEvent::CancelEditGraphqlVariables(_)
				| AppEvent::KeyEventEditGraphqlVariables(_)
				| AppEvent::ModifyRequestPreRequestScript(_)
				| AppEvent::CancelEditRequestPreRequestScript(_)
				| AppEvent::KeyEventEditRequestPreRequestScript(_)
				| AppEvent::ModifyRequestPostRequestScript(_)
				| AppEvent::CancelEditRequestPostRequestScript(_)
				| AppEvent::KeyEventEditRequestPostRequestScript(_) => {
					self.handle_text_input_event(event, key, terminal).await
				}

				/* Export */
				AppEvent::ExportRequest(_)
				| AppEvent::RequestExportFormatMoveCursorLeft(_)
				| AppEvent::RequestExportFormatMoveCursorRight(_)
				| AppEvent::SelectRequestExportFormat(_)
				| AppEvent::ScrollRequestExportUp(_)
				| AppEvent::ScrollRequestExportDown(_)
				| AppEvent::ScrollRequestExportLeft(_)
				| AppEvent::ScrollRequestExportRight(_)
				| AppEvent::CopyRequestExport(_) => self.handle_export_event(event, key),

				/* Response body selection */
				AppEvent::EnterResponseBodySelection(_)
				| AppEvent::ExitResponseBodySelection(_)
				| AppEvent::KeyEventSelectResponseBody(_) => {
					self.handle_response_body_event(event, key, terminal)
				}

				/* Theme picker */
				AppEvent::ThemePickerMoveUp(_)
				| AppEvent::ThemePickerMoveDown(_)
				| AppEvent::ThemePickerConfirm(_) => self.handle_theme_picker_event(event, key),

				/* Settings */
				AppEvent::RequestSettingsMoveUp(_)
				| AppEvent::RequestSettingsMoveDown(_)
				| AppEvent::RequestSettingsToggleSettingLeft(_)
				| AppEvent::RequestSettingsToggleSettingRight(_)
				| AppEvent::ModifyRequestSettings(_) => self.handle_settings_event(event, key),

				/* Documentation (no-op) */
				AppEvent::Documentation(_) => {}
			},
		};

		miss_input
	}
}
