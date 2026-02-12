use crokey::KeyCombination;

use crate::app::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) fn handle_result_tabs_event(
		&mut self,
		event: &AppEvent,
		_key: KeyCombination,
	) {
		match event {
			AppEvent::NextResultTab(_) => self.tui_next_request_result_tab(),

			AppEvent::ScrollResultUp(_) => self.result_vertical_scrollbar.page_up(),
			AppEvent::ScrollResultDown(_) => self.result_vertical_scrollbar.page_down(),
			AppEvent::ScrollResultLeft(_) => self.result_horizontal_scrollbar.page_up(),
			AppEvent::ScrollResultRight(_) => self.result_horizontal_scrollbar.page_down(),

			#[cfg(feature = "clipboard")]
			AppEvent::CopyResponsePart(_) => self.copy_response_body_content_to_clipboard(),

			#[cfg(not(feature = "clipboard"))]
			AppEvent::CopyResponsePart(_) => {}

			_ => unreachable!("handle_result_tabs_event called with non-result-tabs event"),
		}
	}
}
