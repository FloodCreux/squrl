use crokey::key;

use crate::app::files::key_bindings::KeyBindings;
use crate::models::protocol::protocol::Protocol;
use crate::tui::event_key_bindings::EventKeyBinding;
use crate::tui::events::AppEvent;
use crate::tui::events::AppEvent::*;
use crate::tui::ui::param_tabs::param_tabs::RequestParamsTabs;
use crate::tui::ui::views::RequestView;

// ---------------------------------------------------------------------------
// Helper: SelectedRequest -- the most complex state
// ---------------------------------------------------------------------------
// Broken into sub-helpers for readability.
pub(super) fn selected_request_events(
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
