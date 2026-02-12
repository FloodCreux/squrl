use crokey::KeyCombination;
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;
use std::io::Stdout;

use crate::app::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) fn handle_response_body_event(
		&mut self,
		event: &AppEvent,
		key: KeyCombination,
		terminal: &mut Terminal<CrosstermBackend<Stdout>>,
	) {
		match event {
			AppEvent::EnterResponseBodySelection(_) => {
				tracing::debug!("EnterResponseBodySelection event triggered");
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

			_ => unreachable!("handle_response_body_event called with non-response-body event"),
		}
	}
}
