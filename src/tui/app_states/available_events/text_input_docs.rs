use crokey::key;

use crate::app::files::key_bindings::{CustomTextArea, TextAreaMode};
use crate::tui::event_key_bindings::EventKeyBinding;
use crate::tui::events::AppEvent;
use crate::tui::events::AppEvent::*;

use super::super::EMPTY_KEY;

// ---------------------------------------------------------------------------
// Text input documentation generator
// ---------------------------------------------------------------------------
pub(super) fn generate_text_input_documentation(
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
