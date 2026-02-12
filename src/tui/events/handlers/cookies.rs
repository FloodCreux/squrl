use crokey::KeyCombination;

use crate::app::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) fn handle_cookies_event(
		&mut self,
		event: &AppEvent,
		key: KeyCombination,
	) {
		match event {
			AppEvent::CookiesMoveUp(_) => self.core.cookies_popup.cookies_table.up(),
			AppEvent::CookiesMoveDown(_) => self.core.cookies_popup.cookies_table.down(),
			AppEvent::CookiesMoveLeft(_) => self.core.cookies_popup.cookies_table.left(),
			AppEvent::CookiesMoveRight(_) => self.core.cookies_popup.cookies_table.right(),

			AppEvent::EditCookie(_) => {
				if self.core.cookies_popup.cookies_table.selection.is_some() {
					self.edit_cookie_state()
				}
			}
			AppEvent::CreateCookie(_) => self.tui_create_cookie(),
			AppEvent::DeleteCookie(_) => self.tui_delete_cookie(),

			AppEvent::ModifyCookie(_) => match self
				.core
				.cookies_popup
				.cookies_table
				.selection_text_input
				.is_in_default_mode()
			{
				true => self.tui_modify_cookie(),
				false => self
					.core
					.cookies_popup
					.cookies_table
					.selection_text_input
					.key_event(key, None),
			},
			AppEvent::CancelEditCookie(_) => match self
				.core
				.cookies_popup
				.cookies_table
				.selection_text_input
				.is_in_default_mode()
			{
				true => self.display_cookies_state(),
				false => self
					.core
					.cookies_popup
					.cookies_table
					.selection_text_input
					.key_event(key, None),
			},
			AppEvent::KeyEventEditCookie(_) => self
				.core
				.cookies_popup
				.cookies_table
				.selection_text_input
				.key_event(key, None),

			_ => unreachable!("handle_cookies_event called with non-cookies event"),
		}
	}
}
