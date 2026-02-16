use crokey::KeyCombination;

use crate::app::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) fn handle_settings_event(
		&mut self,
		event: &AppEvent,
		_key: KeyCombination,
	) {
		match event {
			AppEvent::RequestSettingsMoveUp(_) => self.request_editor.settings_popup.previous(),
			AppEvent::RequestSettingsMoveDown(_) => self.request_editor.settings_popup.next(),
			AppEvent::RequestSettingsToggleSettingLeft(_) => {
				self.request_editor.settings_popup.toggle_setting_left()
			}
			AppEvent::RequestSettingsToggleSettingRight(_) => {
				self.request_editor.settings_popup.toggle_setting_right()
			}
			AppEvent::ModifyRequestSettings(_) => self.tui_modify_request_settings(),

			_ => unreachable!("handle_settings_event called with non-settings event"),
		}
	}
}
