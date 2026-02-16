use crate::app::files::key_bindings::KeyBindings;
use crate::tui::event_key_bindings::EventKeyBinding;
use crate::tui::events::AppEvent;
use crate::tui::events::AppEvent::*;

use super::dispatch::EventSpec;
use super::text_input_docs::generate_text_input_documentation;

// ---------------------------------------------------------------------------
// Helper: standard text input states (Confirm / Cancel / KeyEvent triplet)
// ---------------------------------------------------------------------------
// Most text-editing states share the same structure: a confirm action, a cancel
// action, and a catch-all key event, followed by text input documentation.
// The `single_line` flag controls which save key is used and the documentation
// style. The `insert_mode_only` flag is forwarded to the documentation generator.
pub(super) fn simple_text_input_events(
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
pub(super) fn text_input_events(
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
pub(super) fn confirmation_dialog_events(
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
pub(super) fn list_view_events(
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
pub(super) fn scroll_view_events(
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
