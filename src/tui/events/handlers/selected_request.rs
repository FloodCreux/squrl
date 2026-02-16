use crokey::KeyCombination;

use crate::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) async fn handle_selected_request_event(
		&mut self,
		event: &AppEvent,
		_key: KeyCombination,
	) {
		match event {
			AppEvent::GoBackToRequestMenu(_) => self.select_request_state(),

			AppEvent::EditUrl(_) => self.edit_request_url_state(),
			AppEvent::EditMethod(_) => self.tui_next_request_method(),
			AppEvent::EditSettings(_) => self.edit_request_settings_state(),

			AppEvent::NextView(_) => self.next_request_view(),
			AppEvent::SendRequest(_) => self.tui_send_request().await,

			_ => {
				unreachable!("handle_selected_request_event called with non-selected-request event")
			}
		}
	}
}
