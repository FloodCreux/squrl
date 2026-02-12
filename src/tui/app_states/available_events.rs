use crokey::key;

use crate::app::files::key_bindings::{CustomTextArea, KEY_BINDINGS, KeyBindings, TextAreaMode};
use crate::models::protocol::protocol::Protocol;
use crate::tui::app_states::AppState;
use crate::tui::app_states::AppState::*;
use crate::tui::event_key_bindings::EventKeyBinding;
use crate::tui::events::AppEvent;
use crate::tui::events::AppEvent::*;
use crate::tui::ui::param_tabs::param_tabs::RequestParamsTabs;
use crate::tui::ui::views::RequestView;

use super::EMPTY_KEY;

/// A tuple of (variant constructor, event name, optional short name) used by
/// helpers that construct events from variant constructors passed as function pointers.
type EventSpec<'a> = (fn(EventKeyBinding) -> AppEvent, &'a str, Option<&'a str>);

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

// ---------------------------------------------------------------------------
// Helper: standard text input states (Confirm / Cancel / KeyEvent triplet)
// ---------------------------------------------------------------------------
// Most text-editing states share the same structure: a confirm action, a cancel
// action, and a catch-all key event, followed by text input documentation.
// The `single_line` flag controls which save key is used and the documentation
// style. The `insert_mode_only` flag is forwarded to the documentation generator.
fn simple_text_input_events(
	key_bindings: &KeyBindings,
	confirm: fn(EventKeyBinding) -> AppEvent,
	cancel: fn(EventKeyBinding) -> AppEvent,
	key_event: fn(EventKeyBinding) -> AppEvent,
	single_line: bool,
	insert_mode_only: bool,
) -> Vec<AppEvent> {
	let save_key = if single_line {
		key_bindings.generic.text_input.save_and_quit_single_line
	} else {
		key_bindings.generic.text_input.save_and_quit_area
	};

	text_input_events(
		vec![
			confirm(EventKeyBinding::new(
				vec![save_key],
				"Confirm",
				Some("Confirm"),
			)),
			cancel(EventKeyBinding::new(
				vec![key_bindings.generic.text_input.quit_without_saving],
				"Cancel",
				Some("Cancel"),
			)),
			key_event(EventKeyBinding::new(vec![], "Any input", None)),
		],
		key_bindings,
		single_line,
		insert_mode_only,
	)
}

// ---------------------------------------------------------------------------
// Helper: text input states (general form)
// ---------------------------------------------------------------------------
// Combines state-specific events (confirm, cancel, key-event, and any extras)
// with the text input documentation entries for the configured text area mode.
fn text_input_events(
	specific_events: Vec<AppEvent>,
	key_bindings: &KeyBindings,
	single_line: bool,
	insert_mode_only: bool,
) -> Vec<AppEvent> {
	[
		specific_events,
		generate_text_input_documentation(
			key_bindings.generic.text_input.mode,
			single_line,
			insert_mode_only,
		),
	]
	.concat()
}

// ---------------------------------------------------------------------------
// Helper: confirmation dialog states (delete collection/request/folder)
// ---------------------------------------------------------------------------
// All confirmation dialogs share: GoBack + move left + move right + select.
// The caller provides variant constructors for the left/right/confirm events.
fn confirmation_dialog_events(
	key_bindings: &KeyBindings,
	move_left: fn(EventKeyBinding) -> AppEvent,
	move_right: fn(EventKeyBinding) -> AppEvent,
	confirm: fn(EventKeyBinding) -> AppEvent,
) -> Vec<AppEvent> {
	vec![
		GoBackToLastState(EventKeyBinding::new(
			vec![key_bindings.generic.navigation.go_back],
			"Cancel",
			Some("Cancel"),
		)),
		move_left(EventKeyBinding::new(
			vec![key_bindings.generic.navigation.move_cursor_left],
			"Move selection left",
			Some("Left"),
		)),
		move_right(EventKeyBinding::new(
			vec![key_bindings.generic.navigation.move_cursor_right],
			"Move selection right",
			Some("Right"),
		)),
		confirm(EventKeyBinding::new(
			vec![key_bindings.generic.navigation.select],
			"Select choice",
			Some("Select"),
		)),
	]
}

// ---------------------------------------------------------------------------
// Helper: list/table view states (env editor, cookies)
// ---------------------------------------------------------------------------
// Pattern: GoBack + edit + up/down/left/right + optional create + optional delete.
#[allow(clippy::too_many_arguments)]
fn list_view_events(
	key_bindings: &KeyBindings,
	edit: fn(EventKeyBinding) -> AppEvent,
	edit_name: &str,
	move_up: fn(EventKeyBinding) -> AppEvent,
	move_down: fn(EventKeyBinding) -> AppEvent,
	move_left: fn(EventKeyBinding) -> AppEvent,
	move_right: fn(EventKeyBinding) -> AppEvent,
	create: Option<EventSpec>,
	delete: Option<EventSpec>,
) -> Vec<AppEvent> {
	let mut events = vec![
		GoBackToLastState(EventKeyBinding::new(
			vec![key_bindings.generic.navigation.go_back],
			"Quit",
			Some("Quit"),
		)),
		edit(EventKeyBinding::new(
			vec![key_bindings.generic.list_and_table_actions.edit_element],
			edit_name,
			None,
		)),
		move_up(EventKeyBinding::new(
			vec![key_bindings.generic.navigation.move_cursor_up],
			"Move up",
			Some("Up"),
		)),
		move_down(EventKeyBinding::new(
			vec![key_bindings.generic.navigation.move_cursor_down],
			"Move down",
			Some("Down"),
		)),
		move_left(EventKeyBinding::new(
			vec![key_bindings.generic.navigation.move_cursor_left],
			"Move left",
			Some("Left"),
		)),
		move_right(EventKeyBinding::new(
			vec![key_bindings.generic.navigation.move_cursor_right],
			"Move right",
			Some("Right"),
		)),
	];

	if let Some((create_fn, create_name, create_short)) = create {
		events.push(create_fn(EventKeyBinding::new(
			vec![key_bindings.generic.list_and_table_actions.create_element],
			create_name,
			create_short,
		)));
	}

	if let Some((delete_fn, delete_name, delete_short)) = delete {
		events.push(delete_fn(EventKeyBinding::new(
			vec![key_bindings.generic.list_and_table_actions.delete_element],
			delete_name,
			delete_short,
		)));
	}

	events
}

// ---------------------------------------------------------------------------
// Helper: scroll view states (logs, request export)
// ---------------------------------------------------------------------------
// Pattern: GoBack + scroll up/down/left/right + optional extra action.
#[allow(clippy::too_many_arguments)]
fn scroll_view_events(
	key_bindings: &KeyBindings,
	go_back: fn(EventKeyBinding) -> AppEvent,
	scroll_up: fn(EventKeyBinding) -> AppEvent,
	scroll_up_name: &str,
	scroll_down: fn(EventKeyBinding) -> AppEvent,
	scroll_down_name: &str,
	scroll_left: fn(EventKeyBinding) -> AppEvent,
	scroll_left_name: &str,
	scroll_right: fn(EventKeyBinding) -> AppEvent,
	scroll_right_name: &str,
	extra: Option<EventSpec>,
) -> Vec<AppEvent> {
	let mut events = vec![
		go_back(EventKeyBinding::new(
			vec![key_bindings.generic.navigation.go_back],
			"Quit",
			Some("Quit"),
		)),
		scroll_up(EventKeyBinding::new(
			vec![key_bindings.request_selected.result_tabs.scroll_up],
			scroll_up_name,
			Some("Up"),
		)),
		scroll_down(EventKeyBinding::new(
			vec![key_bindings.request_selected.result_tabs.scroll_down],
			scroll_down_name,
			Some("Down"),
		)),
		scroll_left(EventKeyBinding::new(
			vec![key_bindings.request_selected.result_tabs.scroll_left],
			scroll_left_name,
			Some("Left"),
		)),
		scroll_right(EventKeyBinding::new(
			vec![key_bindings.request_selected.result_tabs.scroll_right],
			scroll_right_name,
			Some("Right"),
		)),
	];

	if let Some((extra_fn, extra_name, extra_short)) = extra {
		events.push(extra_fn(EventKeyBinding::new(
			vec![key_bindings.request_selected.result_tabs.yank_response_part],
			extra_name,
			extra_short,
		)));
	}

	events
}

// ---------------------------------------------------------------------------
// Helper: SelectedRequest — the most complex state
// ---------------------------------------------------------------------------
// Broken into sub-helpers for readability.
fn selected_request_events(
	key_bindings: &KeyBindings,
	request_view: RequestView,
	request_param_tab: RequestParamsTabs,
	protocol: Option<Protocol>,
	is_there_any_env: bool,
) -> Vec<AppEvent> {
	let (params_events_allowed, result_events_allowed) = match request_view {
		RequestView::Normal => (true, true),
		RequestView::OnlyResult => (false, true),
		RequestView::OnlyParams => (true, false),
	};

	let mut events = selected_request_base_events(key_bindings, is_there_any_env);

	if params_events_allowed {
		events.extend(selected_request_param_tab_events(
			key_bindings,
			request_param_tab,
			protocol,
		));
	} else {
		events.push(NextResultTab(EventKeyBinding::new(
			vec![key_bindings.request_selected.param_next_tab],
			"Next result tab",
			Some("Next tab"),
		)));
	}

	if result_events_allowed {
		events.extend(selected_request_result_tab_events(key_bindings));

		if params_events_allowed {
			events.push(NextResultTab(EventKeyBinding::new(
				vec![key_bindings.request_selected.result_tabs.result_next_tab],
				"Next result tab",
				None,
			)));
		}
	}

	events
}

fn selected_request_base_events(
	key_bindings: &KeyBindings,
	is_there_any_env: bool,
) -> Vec<AppEvent> {
	let mut events = vec![
		ExitApp(EventKeyBinding::new(vec![key!(ctrl - c)], "Exit app", None)),
		GoBackToLastState(EventKeyBinding::new(
			vec![key_bindings.generic.navigation.go_back],
			"Quit to main menu",
			Some("Quit"),
		)),
		Documentation(EventKeyBinding::new(
			vec![key_bindings.generic.display_help],
			"Display help",
			Some("Help"),
		)),
		EditUrl(EventKeyBinding::new(
			vec![key_bindings.request_selected.change_url],
			"Edit URL",
			Some("URL"),
		)),
		EditSettings(EventKeyBinding::new(
			vec![key_bindings.request_selected.request_settings],
			"Request settings",
			None,
		)),
		NextView(EventKeyBinding::new(
			vec![key_bindings.request_selected.next_view],
			"Next view",
			None,
		)),
		SendRequest(EventKeyBinding::new(
			vec![
				key_bindings.request_selected.send_request,
				key_bindings.request_selected.alt_send_request,
			],
			"Send/cancel request",
			Some("Send/Cancel"),
		)),
	];

	if is_there_any_env {
		events.extend(vec![
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
		]);
	}

	events.extend(vec![
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
		ExportRequest(EventKeyBinding::new(
			vec![key_bindings.request_selected.export_request],
			"Export request",
			None,
		)),
	]);

	events
}

fn selected_request_param_tab_events(
	key_bindings: &KeyBindings,
	request_param_tab: RequestParamsTabs,
	protocol: Option<Protocol>,
) -> Vec<AppEvent> {
	let mut events = vec![
		NextParamTab(EventKeyBinding::new(
			vec![key_bindings.request_selected.param_next_tab],
			"Next param tab",
			Some("Next tab"),
		)),
		ModifyRequestAuthMethod(EventKeyBinding::new(
			vec![key_bindings.request_selected.param_tabs.change_auth_method],
			"Modify auth method",
			None,
		)),
	];

	if let Some(protocol) = protocol {
		let protocol_specific = match protocol {
			Protocol::HttpRequest(_) => vec![
				EditMethod(EventKeyBinding::new(
					vec![key_bindings.request_selected.change_method],
					"Change method",
					Some("Method"),
				)),
				ModifyRequestBodyContentType(EventKeyBinding::new(
					vec![
						key_bindings
							.request_selected
							.param_tabs
							.change_body_content_type,
					],
					"Modify body content-type",
					None,
				)),
			],
			Protocol::WsRequest(_) => {
				vec![ModifyRequestMessageType(EventKeyBinding::new(
					vec![key_bindings.request_selected.param_tabs.change_message_type],
					"Modify message type",
					None,
				))]
			}
		};

		events.extend(protocol_specific);
	}

	let param_tabs_events = match request_param_tab {
		RequestParamsTabs::QueryParams => vec![
			EditRequestQueryParam(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.edit_element],
				"Edit query param",
				None,
			)),
			RequestQueryParamsMoveUp(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_up],
				"Move up",
				None,
			)),
			RequestQueryParamsMoveDown(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_down],
				"Move down",
				None,
			)),
			RequestQueryParamsMoveLeft(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_left],
				"Move left",
				None,
			)),
			RequestQueryParamsMoveRight(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_right],
				"Move right",
				None,
			)),
			CreateRequestQueryParam(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.create_element],
				"Create query param",
				None,
			)),
			DeleteRequestQueryParam(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.delete_element],
				"Delete query param",
				None,
			)),
			ToggleRequestQueryParam(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.toggle_element],
				"Toggle query param",
				None,
			)),
			DuplicateRequestQueryParam(EventKeyBinding::new(
				vec![
					key_bindings
						.generic
						.list_and_table_actions
						.duplicate_element,
				],
				"Duplicate query param",
				None,
			)),
		],
		RequestParamsTabs::Auth => vec![
			EditRequestAuth(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.edit_element],
				"Edit auth element",
				None,
			)),
			RequestAuthMoveUp(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_up],
				"Move up",
				None,
			)),
			RequestAuthMoveDown(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_down],
				"Move down",
				None,
			)),
			RequestAuthMoveLeft(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_left],
				"Move left",
				None,
			)),
			RequestAuthMoveRight(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_right],
				"Move right",
				None,
			)),
		],
		RequestParamsTabs::Headers => vec![
			EditRequestHeader(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.edit_element],
				"Edit header",
				None,
			)),
			RequestHeadersMoveUp(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_up],
				"Move up",
				None,
			)),
			RequestHeadersMoveDown(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_down],
				"Move down",
				None,
			)),
			RequestHeadersMoveLeft(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_left],
				"Move left",
				None,
			)),
			RequestHeadersMoveRight(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_right],
				"Move right",
				None,
			)),
			CreateRequestHeader(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.create_element],
				"Create header",
				None,
			)),
			DeleteRequestHeader(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.delete_element],
				"Delete header",
				None,
			)),
			ToggleRequestHeader(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.toggle_element],
				"Toggle header",
				None,
			)),
			DuplicateRequestHeader(EventKeyBinding::new(
				vec![
					key_bindings
						.generic
						.list_and_table_actions
						.duplicate_element,
				],
				"Duplicate header",
				None,
			)),
		],
		RequestParamsTabs::Body => vec![
			EditRequestBody(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.edit_element],
				"Edit body",
				None,
			)),
			RequestBodyTableMoveUp(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_up],
				"Move up",
				None,
			)),
			RequestBodyTableMoveDown(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_down],
				"Move down",
				None,
			)),
			RequestBodyTableMoveLeft(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_left],
				"Move left",
				None,
			)),
			RequestBodyTableMoveRight(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_right],
				"Move right",
				None,
			)),
			CreateRequestBodyTableElement(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.create_element],
				"Create form element",
				None,
			)),
			DeleteRequestBodyTableElement(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.delete_element],
				"Delete form element",
				None,
			)),
			ToggleRequestBodyTableElement(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.toggle_element],
				"Toggle form element",
				None,
			)),
			DuplicateRequestBodyTableElement(EventKeyBinding::new(
				vec![
					key_bindings
						.generic
						.list_and_table_actions
						.duplicate_element,
				],
				"Duplicate form element",
				None,
			)),
		],
		RequestParamsTabs::Message => {
			vec![EditRequestMessage(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.edit_element],
				"Edit message",
				None,
			))]
		}
		RequestParamsTabs::Scripts => vec![
			EditRequestScript(EventKeyBinding::new(
				vec![key_bindings.generic.list_and_table_actions.edit_element],
				"Edit request script",
				Some("Edit"),
			)),
			RequestScriptMove(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_up],
				"Move up",
				Some("Up"),
			)),
			RequestScriptMove(EventKeyBinding::new(
				vec![key_bindings.generic.navigation.move_cursor_down],
				"Move down",
				Some("Down"),
			)),
		],
	};

	events.extend(param_tabs_events);
	events
}

fn selected_request_result_tab_events(key_bindings: &KeyBindings) -> Vec<AppEvent> {
	vec![
		ScrollResultUp(EventKeyBinding::new(
			vec![key_bindings.request_selected.result_tabs.scroll_up],
			"Scroll result up",
			None,
		)),
		ScrollResultDown(EventKeyBinding::new(
			vec![key_bindings.request_selected.result_tabs.scroll_down],
			"Scroll result down",
			None,
		)),
		ScrollResultLeft(EventKeyBinding::new(
			vec![key_bindings.request_selected.result_tabs.scroll_left],
			"Scroll result left",
			None,
		)),
		ScrollResultRight(EventKeyBinding::new(
			vec![key_bindings.request_selected.result_tabs.scroll_right],
			"Scroll result right",
			None,
		)),
		CopyResponsePart(EventKeyBinding::new(
			vec![key_bindings.request_selected.result_tabs.yank_response_part],
			"Yank response part",
			Some("Yank"),
		)),
		EnterResponseBodySelection(EventKeyBinding::new(
			vec![
				key_bindings
					.request_selected
					.result_tabs
					.select_response_body,
			],
			"Select response body",
			Some("Select"),
		)),
	]
}

// ---------------------------------------------------------------------------
// Text input documentation generator
// ---------------------------------------------------------------------------
fn generate_text_input_documentation(
	text_input_mode: TextAreaMode,
	single_line: bool,
	insert_mode_only: bool,
) -> Vec<AppEvent> {
	let mut initial = Vec::new();

	match text_input_mode {
		TextAreaMode::Vim => {
			if !single_line {
				initial.push(Documentation(EventKeyBinding::new(
					vec![key!(ctrl - e)],
					"System editor",
					None,
				)));
			}

			if !insert_mode_only {
				initial.extend(vec![
					Documentation(EventKeyBinding::new(
						vec![key!(esc)],
						"Normal mode",
						Some("Esc"),
					)),
					Documentation(EventKeyBinding::new(
						vec![key!(i)],
						"Enter insert mode",
						None,
					)),
					Documentation(EventKeyBinding::new(
						vec![key!(v)],
						"Enter visual mode",
						None,
					)),
					Documentation(EventKeyBinding::new(
						vec![key!('/')],
						"Start search",
						Some("Search"),
					)),
				]);
			}

			initial.extend(vec![
				Documentation(EventKeyBinding::new(vec![key!(y)], "Copy selection", None)),
				Documentation(EventKeyBinding::new(
					vec![key!(y), key!(y)],
					"Copy line",
					None,
				)),
				Documentation(EventKeyBinding::new(vec![key!(p)], "Paste", None)),
				Documentation(EventKeyBinding::new(vec![key!(u)], "Undo", Some("Undo"))),
				Documentation(EventKeyBinding::new(
					vec![key!(ctrl - r)],
					"Redo",
					Some("Redo"),
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(w)],
					"Move to next word",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(e)],
					"Move to end of word",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(b)],
					"Move to previous word",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(0)],
					"Move to start of line",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!('$')],
					"Move to end of line",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(g), key!(g)],
					"Move to first line",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(G)],
					"Move to last line",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(a)],
					"Append after cursor",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(o)],
					"Insert line below",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(O)],
					"Insert line above",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(enter)],
					"Insert line break",
					None,
				)),
				Documentation(EventKeyBinding::new(vec![key!(x)], "Delete char", None)),
				Documentation(EventKeyBinding::new(
					vec![key!(d), key!(d)],
					"Delete line",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(D)],
					"Delete to end of line",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![*EMPTY_KEY],
					"Many other vim commands...",
					None,
				)),
			]);
		}
		TextAreaMode::Emacs => {
			if !single_line {
				initial.push(Documentation(EventKeyBinding::new(
					vec![key!(alt - e)],
					"System editor",
					None,
				)));
			}

			initial.extend(vec![
				Documentation(EventKeyBinding::new(
					vec![key!(ctrl - u)],
					"Undo",
					Some("Undo"),
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(ctrl - r)],
					"Redo",
					Some("Redo"),
				)),
				Documentation(EventKeyBinding::new(vec![key!(ctrl - y)], "Paste", None)),
				Documentation(EventKeyBinding::new(
					vec![key!(backspace)],
					"Remove char from search",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(ctrl - k)],
					"Delete to end of line",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(ctrl - o)],
					"Insert line break above",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(enter)],
					"Insert line break",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(ctrl - j)],
					"Insert line break",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(backspace)],
					"Delete previous char",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(ctrl - h)],
					"Delete previous char",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(backspace)],
					"Delete next char",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(ctrl - d)],
					"Delete next char",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(alt - d)],
					"Delete next word",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![key!(alt - backspace)],
					"Delete previous word",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![*EMPTY_KEY],
					"Many other emacs shortcuts...",
					None,
				)),
			]);

			if !single_line {
				initial.extend(vec![
					Documentation(EventKeyBinding::new(
						vec![key!(ctrl - s)],
						"Start search",
						Some("Search"),
					)),
					Documentation(EventKeyBinding::new(
						vec![key!(ctrl - s)],
						"Find next match",
						None,
					)),
					Documentation(EventKeyBinding::new(
						vec![key!(ctrl - r)],
						"Find previous match",
						None,
					)),
					Documentation(EventKeyBinding::new(
						vec![key!(enter)],
						"Select current search result",
						None,
					)),
					Documentation(EventKeyBinding::new(
						vec![key!(ctrl - g)],
						"Stop search",
						None,
					)),
				]);
			}
		}
		_ => {
			let custom_text_area_bindings = match text_input_mode {
				TextAreaMode::Default => CustomTextArea::default(),
				TextAreaMode::Custom(custom_text_area_bindings) => custom_text_area_bindings,
				_ => unreachable!(),
			};

			initial.extend(vec![
				Documentation(EventKeyBinding::new(
					vec![custom_text_area_bindings.delete_backward],
					"Delete char backward",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![custom_text_area_bindings.delete_forward],
					"Delete char forward",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![custom_text_area_bindings.move_cursor_left],
					"Move cursor left",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![custom_text_area_bindings.move_cursor_right],
					"Move cursor right",
					None,
				)),
			]);

			if !single_line {
				initial.extend(vec![
					Documentation(EventKeyBinding::new(
						vec![custom_text_area_bindings.move_cursor_up],
						"Move cursor up",
						None,
					)),
					Documentation(EventKeyBinding::new(
						vec![custom_text_area_bindings.move_cursor_down],
						"Move cursor down",
						None,
					)),
				]);
			}

			initial.extend(vec![
				Documentation(EventKeyBinding::new(
					vec![custom_text_area_bindings.move_cursor_line_start],
					"Move cursor line start",
					Some("Home"),
				)),
				Documentation(EventKeyBinding::new(
					vec![custom_text_area_bindings.move_cursor_line_end],
					"Move cursor line end",
					Some("End"),
				)),
				Documentation(EventKeyBinding::new(
					vec![custom_text_area_bindings.skip_word_left],
					"Skip word left",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![custom_text_area_bindings.skip_word_right],
					"Skip word right",
					None,
				)),
				Documentation(EventKeyBinding::new(
					vec![custom_text_area_bindings.undo],
					"Undo",
					Some("Undo"),
				)),
				Documentation(EventKeyBinding::new(
					vec![custom_text_area_bindings.redo],
					"Redo",
					None,
				)),
			]);

			if !insert_mode_only {
				initial.push(Documentation(EventKeyBinding::new(
					vec![custom_text_area_bindings.search],
					"Search",
					Some("Search"),
				)));
			}
		}
	}

	initial
}
