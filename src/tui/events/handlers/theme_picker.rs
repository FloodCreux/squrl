use crokey::KeyCombination;

use crate::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) fn handle_theme_picker_event(
		&mut self,
		event: &AppEvent,
		_key: KeyCombination,
	) {
		match event {
			AppEvent::ThemePickerMoveUp(_) => {
				tracing::debug!("ThemePickerMoveUp event triggered");
				self.theme_popup.previous();
			}
			AppEvent::ThemePickerMoveDown(_) => {
				tracing::debug!("ThemePickerMoveDown event triggered");
				self.theme_popup.next();
			}
			AppEvent::ThemePickerConfirm(_) => {
				tracing::debug!("ThemePickerConfirm event triggered");
				if let Some(theme_name) = self.theme_popup.confirm() {
					self.save_theme_to_global_config(&theme_name);
				}
				self.go_back_to_last_state();
			}

			_ => unreachable!("handle_theme_picker_event called with non-theme-picker event"),
		}
	}
}
