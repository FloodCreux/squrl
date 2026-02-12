use crokey::KeyCombination;

use crate::app::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) fn handle_main_menu_event(
		&mut self,
		event: &AppEvent,
		_key: KeyCombination,
	) {
		match event {
			AppEvent::ExitApp(_) => self.should_quit = true,

			AppEvent::MoveCollectionCursorUp(_) => self.collections_tree.up(),
			AppEvent::MoveCollectionCursorDown(_) => self.collections_tree.down(),

			AppEvent::SelectRequestOrExpandCollection(_) => {
				self.select_request_or_expand_collection()
			}
			AppEvent::ExpandCollection(_) => {
				self.collections_tree.state.toggle_selected();
			}
			AppEvent::UnselectRequest(_) => self.unselect_request(),

			AppEvent::CreateElement(_) => self.choose_element_to_create_state(),
			AppEvent::DeleteElement(_) => self.delete_element(),
			AppEvent::RenameElement(_) => self.rename_element(),
			AppEvent::DuplicateElement(_) => self.duplicate_element(),

			AppEvent::MoveElementUp(_) => self.tui_move_element_up(),
			AppEvent::MoveElementDown(_) => self.tui_move_element_down(),

			AppEvent::NextEnvironment(_) => self.tui_next_environment(),
			AppEvent::DisplayEnvEditor(_) => self.display_env_editor_state(),
			AppEvent::DisplayCookies(_) => self.display_cookies_state(),
			AppEvent::DisplayLogs(_) => self.display_logs_state(),
			AppEvent::DisplayThemePicker(_) => self.choose_theme_state(),

			AppEvent::GoBackToLastState(_) => match self.state {
				crate::tui::app_states::AppState::ChoosingTheme => {
					self.theme_popup.cancel();
					self.normal_state();
				}
				_ => self.normal_state(),
			},

			_ => unreachable!("handle_main_menu_event called with non-main-menu event"),
		}
	}
}
