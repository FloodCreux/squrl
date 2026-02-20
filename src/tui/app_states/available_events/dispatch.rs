use crokey::key;

use crate::app::files::key_bindings::KEY_BINDINGS;
use crate::models::protocol::protocol::Protocol;
use crate::tui::app_states::AppState;
use crate::tui::app_states::AppState::*;
use crate::tui::event_key_bindings::EventKeyBinding;
use crate::tui::events::AppEvent;
use crate::tui::events::AppEvent::*;
use crate::tui::ui::param_tabs::param_tabs::RequestParamsTabs;
use crate::tui::ui::views::RequestView;

use super::helpers::*;
use super::selected_request::selected_request_events;

/// A tuple of (variant constructor, event name, optional short name) used by
/// helpers that construct events from variant constructors passed as function pointers.
pub(super) type EventSpec<'a> = (fn(EventKeyBinding) -> AppEvent, &'a str, Option<&'a str>);

impl AppState {
	pub fn get_available_events(
		&self,
		request_view: RequestView,
		request_param_tab: RequestParamsTabs,
		protocol: Option<Protocol>,
		is_there_any_env: bool,
	) -> Vec<AppEvent> {
		let key_bindings = KEY_BINDINGS.read();

		match self {
			Normal => {
				let mut base_events = vec![
					ExitApp(EventKeyBinding::new(
						vec![key_bindings.main_menu.exit, key!(ctrl - c)],
						"Exit",
						Some("Exit"),
					)),
					Documentation(EventKeyBinding::new(
						vec![key_bindings.generic.display_help],
						"Display help",
						Some("Help"),
					)),
					MoveCollectionCursorUp(EventKeyBinding::new(
						vec![key_bindings.generic.navigation.move_cursor_up],
						"Move up",
						Some("Up"),
					)),
					MoveCollectionCursorDown(EventKeyBinding::new(
						vec![key_bindings.generic.navigation.move_cursor_down],
						"Move down",
						Some("Down"),
					)),
					SelectRequestOrExpandCollection(EventKeyBinding::new(
						vec![key_bindings.generic.navigation.select],
						"Select",
						Some("Select"),
					)),
					UnselectRequest(EventKeyBinding::new(
						vec![key_bindings.main_menu.unselect_request],
						"Unselect",
						None,
					)),
					ExpandCollection(EventKeyBinding::new(
						vec![key_bindings.main_menu.expand_collection],
						"Expand",
						None,
					)),
					CreateElement(EventKeyBinding::new(
						vec![key_bindings.generic.list_and_table_actions.create_element],
						"Create element",
						Some("Create"),
					)),
					DeleteElement(EventKeyBinding::new(
						vec![key_bindings.generic.list_and_table_actions.delete_element],
						"Delete element",
						None,
					)),
					RenameElement(EventKeyBinding::new(
						vec![key_bindings.generic.list_and_table_actions.rename_element],
						"Rename element",
						None,
					)),
					DuplicateElement(EventKeyBinding::new(
						vec![
							key_bindings
								.generic
								.list_and_table_actions
								.duplicate_element,
						],
						"Duplicate element",
						None,
					)),
					MoveElementUp(EventKeyBinding::new(
						vec![key_bindings.main_menu.move_request_up],
						"Move request up",
						None,
					)),
					MoveElementDown(EventKeyBinding::new(
						vec![key_bindings.main_menu.move_request_down],
						"Move request down",
						None,
					)),
				];

				if is_there_any_env {
					let env_events = vec![
						NextEnvironment(EventKeyBinding::new(
							vec![key_bindings.main_menu.next_environment],
							"Next environment",
							None,
						)),
						DisplayEnvEditor(EventKeyBinding::new(
							vec![key_bindings.main_menu.display_env_editor],
							"Environment editor",
							None,
						)),
					];

					base_events.extend(env_events);
				}

				let other_events = vec![
					DisplayCookies(EventKeyBinding::new(
						vec![key_bindings.main_menu.display_cookies],
						"Display cookies",
						None,
					)),
					DisplayLogs(EventKeyBinding::new(
						vec![key_bindings.main_menu.display_logs],
						"Display logs",
						None,
					)),
					DisplayThemePicker(EventKeyBinding::new(
						vec![key_bindings.main_menu.display_theme_picker],
						"Theme picker",
						None,
					)),
				];

				base_events.extend(other_events);

				base_events
			}

			DisplayingEnvEditor => list_view_events(
				&key_bindings,
				EditEnvVariable,
				"Edit env variable",
				EnvVariablesMoveUp,
				EnvVariablesMoveDown,
				EnvVariablesMoveLeft,
				EnvVariablesMoveRight,
				Some((
					CreateEnvVariable,
					"Create env variable",
					Some("Create variable"),
				)),
				Some((
					DeleteEnvVariable,
					"Delete env variable",
					Some("Delete variable"),
				)),
			),

			EditingEnvVariable => simple_text_input_events(
				&key_bindings,
				ModifyEnvVariable,
				CancelModifyEnvVariable,
				KeyEventModifyEnvVariable,
				true,
				true,
			),

			DisplayingCookies => list_view_events(
				&key_bindings,
				EditCookie,
				"Edit cookie",
				CookiesMoveUp,
				CookiesMoveDown,
				CookiesMoveLeft,
				CookiesMoveRight,
				Some((CreateCookie, "Create cookie", Some("Create cookie"))),
				Some((DeleteCookie, "Delete cookie", Some("Delete"))),
			),

			EditingCookies => simple_text_input_events(
				&key_bindings,
				ModifyCookie,
				CancelEditCookie,
				KeyEventEditCookie,
				true,
				true,
			),

			DisplayingLogs => scroll_view_events(
				&key_bindings,
				GoBackToLastState,
				ScrollLogsUp,
				"Scroll logs up",
				ScrollLogsDown,
				"Scroll logs down",
				ScrollLogsLeft,
				"Scroll logs left",
				ScrollLogsRight,
				"Scroll logs right",
				None,
			),

			ChoosingElementToCreate => vec![
				GoBackToLastState(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.go_back],
					"Quit",
					Some("Quit"),
				)),
				ChooseElementToCreateMoveCursorLeft(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.move_cursor_left],
					"Move selection left",
					Some("Left"),
				)),
				ChooseElementToCreateMoveCursorRight(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.move_cursor_right],
					"Move selection right",
					Some("Right"),
				)),
				SelectElementToCreate(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.select],
					"Select element to create",
					Some("Select"),
				)),
			],

			CreatingNewCollection => simple_text_input_events(
				&key_bindings,
				CreateNewCollection,
				CancelCreateNewCollection,
				KeyEventCreateNewCollection,
				true,
				false,
			),

			CreatingNewRequest => text_input_events(
				vec![
					CreateNewRequest(EventKeyBinding::new(
						vec![key_bindings.generic.text_input.save_and_quit_single_line],
						"Confirm",
						Some("Confirm"),
					)),
					CancelCreateNewRequest(EventKeyBinding::new(
						vec![key_bindings.generic.text_input.quit_without_saving],
						"Cancel",
						Some("Cancel"),
					)),
					CreatingRequestSelectInputUp(EventKeyBinding::new(
						vec![key_bindings.generic.navigation.alt_move_cursor_up],
						"Input selection up",
						Some("Up"),
					)),
					CreatingRequestSelectInputDown(EventKeyBinding::new(
						vec![key_bindings.generic.navigation.alt_move_cursor_down],
						"Input selection down",
						Some("Down"),
					)),
					CreatingRequestInputLeft(EventKeyBinding::new(
						vec![key_bindings.generic.navigation.move_cursor_left],
						"Previous",
						Some("Left"),
					)),
					CreatingRequestInputRight(EventKeyBinding::new(
						vec![key_bindings.generic.navigation.move_cursor_right],
						"Next",
						Some("Right"),
					)),
					KeyEventCreateNewRequest(EventKeyBinding::new(vec![], "Any input", None)),
				],
				&key_bindings,
				true,
				false,
			),

			DeletingCollection => confirmation_dialog_events(
				&key_bindings,
				DeletingCollectionMoveCursorLeft,
				DeletingCollectionMoveCursorRight,
				DeleteCollection,
			),

			DeletingRequest => confirmation_dialog_events(
				&key_bindings,
				DeletingRequestMoveCursorLeft,
				DeletingRequestMoveCursorRight,
				DeleteRequest,
			),

			RenamingCollection => simple_text_input_events(
				&key_bindings,
				RenameCollection,
				CancelRenameCollection,
				KeyEventRenameCollection,
				true,
				false,
			),

			RenamingRequest => simple_text_input_events(
				&key_bindings,
				RenameRequest,
				CancelRenameRequest,
				KeyEventRenameRequest,
				true,
				false,
			),

			CreatingNewFolder => simple_text_input_events(
				&key_bindings,
				CreateNewFolder,
				CancelCreateNewFolder,
				KeyEventCreateNewFolder,
				true,
				false,
			),

			DeletingFolder => confirmation_dialog_events(
				&key_bindings,
				DeletingFolderMoveCursorLeft,
				DeletingFolderMoveCursorRight,
				DeleteFolder,
			),

			RenamingFolder => simple_text_input_events(
				&key_bindings,
				RenameFolder,
				CancelRenameFolder,
				KeyEventRenameFolder,
				true,
				false,
			),

			SelectedRequest => selected_request_events(
				&key_bindings,
				request_view,
				request_param_tab,
				protocol,
				is_there_any_env,
			),

			EditingRequestUrl => simple_text_input_events(
				&key_bindings,
				ModifyRequestUrl,
				CancelEditRequestUrl,
				KeyEventEditRequestUrl,
				true,
				false,
			),

			EditingRequestParam => simple_text_input_events(
				&key_bindings,
				ModifyRequestQueryParam,
				CancelEditRequestQueryParam,
				KeyEventEditRequestQueryParam,
				true,
				true,
			),

			EditingRequestAuthBasicUsername => simple_text_input_events(
				&key_bindings,
				ModifyRequestAuthBasicUsername,
				CancelEditRequestAuthBasicUsername,
				KeyEventEditRequestAuthBasicUsername,
				true,
				false,
			),

			EditingRequestAuthBasicPassword => simple_text_input_events(
				&key_bindings,
				ModifyRequestAuthBasicPassword,
				CancelEditRequestAuthBasicPassword,
				KeyEventEditRequestAuthBasicPassword,
				true,
				false,
			),

			EditingRequestAuthBearerToken => simple_text_input_events(
				&key_bindings,
				ModifyRequestAuthBearerToken,
				CancelEditRequestAuthBearerToken,
				KeyEventEditRequestAuthBearerToken,
				true,
				false,
			),

			EditingRequestAuthJwtSecret => simple_text_input_events(
				&key_bindings,
				ModifyRequestAuthJwtSecret,
				CancelEditRequestAuthJwtSecret,
				KeyEventEditRequestAuthJwtSecret,
				true,
				false,
			),

			// JWT payload is a multi-line text area
			EditingRequestAuthJwtPayload => simple_text_input_events(
				&key_bindings,
				ModifyRequestAuthJwtPayload,
				CancelEditRequestAuthJwtPayload,
				KeyEventEditRequestAuthJwtPayload,
				false,
				false,
			),

			EditingRequestAuthDigestUsername => simple_text_input_events(
				&key_bindings,
				ModifyRequestAuthDigestUsername,
				CancelEditRequestAuthDigestUsername,
				KeyEventEditRequestAuthDigestUsername,
				true,
				false,
			),

			EditingRequestAuthDigestPassword => simple_text_input_events(
				&key_bindings,
				ModifyRequestAuthDigestPassword,
				CancelEditRequestAuthDigestPassword,
				KeyEventEditRequestAuthDigestPassword,
				true,
				false,
			),

			EditingRequestAuthDigestDomains => simple_text_input_events(
				&key_bindings,
				ModifyRequestAuthDigestDomains,
				CancelEditRequestAuthDigestDomains,
				KeyEventEditRequestAuthDigestDomains,
				true,
				false,
			),

			EditingRequestAuthDigestRealm => simple_text_input_events(
				&key_bindings,
				ModifyRequestAuthDigestRealm,
				CancelEditRequestAuthDigestRealm,
				KeyEventEditRequestAuthDigestRealm,
				true,
				false,
			),

			EditingRequestAuthDigestNonce => simple_text_input_events(
				&key_bindings,
				ModifyRequestAuthDigestNonce,
				CancelEditRequestAuthDigestNonce,
				KeyEventEditRequestAuthDigestNonce,
				true,
				false,
			),

			EditingRequestAuthDigestOpaque => simple_text_input_events(
				&key_bindings,
				ModifyRequestAuthDigestOpaque,
				CancelEditRequestAuthDigestOpaque,
				KeyEventEditRequestAuthDigestOpaque,
				true,
				false,
			),

			EditingRequestHeader => simple_text_input_events(
				&key_bindings,
				ModifyRequestHeader,
				CancelEditRequestHeader,
				KeyEventEditRequestHeader,
				true,
				true,
			),

			EditingRequestBodyTable => simple_text_input_events(
				&key_bindings,
				ModifyRequestBodyTable,
				CancelEditRequestBodyTable,
				KeyEventEditRequestBodyTable,
				true,
				true,
			),

			EditingRequestBodyFile => simple_text_input_events(
				&key_bindings,
				ModifyRequestBodyFile,
				CancelEditRequestBodyFile,
				KeyEventEditRequestBodyFile,
				true,
				false,
			),

			// Multi-line text areas
			EditingRequestBodyString => simple_text_input_events(
				&key_bindings,
				ModifyRequestBodyString,
				CancelEditRequestBodyString,
				KeyEventEditRequestBodyString,
				false,
				false,
			),

			EditingRequestMessage => simple_text_input_events(
				&key_bindings,
				ModifyRequestMessage,
				CancelEditRequestMessage,
				KeyEventEditRequestMessage,
				false,
				false,
			),

			EditingGraphqlQuery => simple_text_input_events(
				&key_bindings,
				ModifyGraphqlQuery,
				CancelEditGraphqlQuery,
				KeyEventEditGraphqlQuery,
				false,
				false,
			),

			EditingGraphqlVariables => simple_text_input_events(
				&key_bindings,
				ModifyGraphqlVariables,
				CancelEditGraphqlVariables,
				KeyEventEditGraphqlVariables,
				false,
				false,
			),

			EditingPreRequestScript => simple_text_input_events(
				&key_bindings,
				ModifyRequestPreRequestScript,
				CancelEditRequestPreRequestScript,
				KeyEventEditRequestPreRequestScript,
				false,
				false,
			),

			EditingPostRequestScript => simple_text_input_events(
				&key_bindings,
				ModifyRequestPostRequestScript,
				CancelEditRequestPostRequestScript,
				KeyEventEditRequestPostRequestScript,
				false,
				false,
			),

			EditingRequestSettings => vec![
				GoBackToRequestMenu(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.go_back],
					"Cancel",
					Some("Cancel"),
				)),
				RequestSettingsMoveUp(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.move_cursor_up],
					"Move up",
					Some("Up"),
				)),
				RequestSettingsMoveDown(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.move_cursor_down],
					"Move down",
					Some("Down"),
				)),
				RequestSettingsToggleSettingLeft(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.move_cursor_left],
					"Toggle setting",
					Some("Toggle left"),
				)),
				RequestSettingsToggleSettingRight(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.move_cursor_right],
					"Toggle setting",
					Some("Toggle right"),
				)),
				ModifyRequestSettings(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.select],
					"Confirm",
					Some("Confirm"),
				)),
			],

			ChoosingRequestExportFormat => vec![
				GoBackToRequestMenu(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.go_back],
					"Quit",
					Some("Quit"),
				)),
				RequestExportFormatMoveCursorLeft(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.move_cursor_left],
					"Move selection left",
					Some("Left"),
				)),
				RequestExportFormatMoveCursorRight(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.move_cursor_right],
					"Move selection right",
					Some("Right"),
				)),
				SelectRequestExportFormat(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.select],
					"Select export format",
					Some("Select"),
				)),
			],

			DisplayingRequestExport => scroll_view_events(
				&key_bindings,
				GoBackToRequestMenu,
				ScrollRequestExportUp,
				"Scroll request export up",
				ScrollRequestExportDown,
				"Scroll request export down",
				ScrollRequestExportLeft,
				"Scroll request export left",
				ScrollRequestExportRight,
				"Scroll request export right",
				Some((CopyRequestExport, "Yank request export", Some("Yank"))),
			),

			SelectingResponseBody => text_input_events(
				vec![
					ExitResponseBodySelection(EventKeyBinding::new(
						vec![key_bindings.generic.navigation.go_back],
						"Exit selection",
						Some("Exit"),
					)),
					KeyEventSelectResponseBody(EventKeyBinding::new(vec![], "Any input", None)),
				],
				&key_bindings,
				false,
				false,
			),

			ChoosingTheme => vec![
				GoBackToLastState(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.go_back],
					"Cancel",
					Some("Cancel"),
				)),
				ThemePickerMoveUp(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.move_cursor_up],
					"Previous theme",
					Some("Up"),
				)),
				ThemePickerMoveDown(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.move_cursor_down],
					"Next theme",
					Some("Down"),
				)),
				ThemePickerConfirm(EventKeyBinding::new(
					vec![key_bindings.generic.navigation.select],
					"Confirm theme",
					Some("Confirm"),
				)),
			],
		}
	}
}
