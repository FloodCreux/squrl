use crokey::KeyCombination;

use crate::app::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) fn handle_logs_event(
		&mut self,
		event: &AppEvent,
		_key: KeyCombination,
	) {
		match event {
			AppEvent::ScrollLogsUp(_) => self.logs_vertical_scrollbar.page_up(),
			AppEvent::ScrollLogsDown(_) => self.logs_vertical_scrollbar.page_down(),
			AppEvent::ScrollLogsLeft(_) => self.logs_horizontal_scrollbar.page_up(),
			AppEvent::ScrollLogsRight(_) => self.logs_horizontal_scrollbar.page_down(),

			_ => unreachable!("handle_logs_event called with non-logs event"),
		}
	}
}
