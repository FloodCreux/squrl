use crokey::KeyCombination;

use crate::app::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) fn handle_folders_event(
		&mut self,
		event: &AppEvent,
		key: KeyCombination,
	) {
		match event {
			AppEvent::CreateNewFolder(_) => match self.new_collection_input.is_in_default_mode() {
				true => self.tui_new_folder(),
				false => self.new_collection_input.key_event(key, None),
			},
			AppEvent::CancelCreateNewFolder(_) => {
				match self.new_collection_input.is_in_default_mode() {
					true => self.normal_state(),
					false => self.new_collection_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventCreateNewFolder(_) => self.new_collection_input.key_event(key, None),

			AppEvent::DeletingFolderMoveCursorLeft(_) => {
				self.delete_collection_popup.change_state()
			}
			AppEvent::DeletingFolderMoveCursorRight(_) => {
				self.delete_collection_popup.change_state()
			}
			AppEvent::DeleteFolder(_) => match self.delete_collection_popup.state {
				true => self.tui_delete_folder(),
				false => self.normal_state(),
			},

			AppEvent::RenameFolder(_) => match self.rename_collection_input.is_in_default_mode() {
				true => self.tui_rename_folder(),
				false => self.rename_collection_input.key_event(key, None),
			},
			AppEvent::CancelRenameFolder(_) => {
				match self.rename_collection_input.is_in_default_mode() {
					true => self.normal_state(),
					false => self.rename_collection_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventRenameFolder(_) => self.rename_collection_input.key_event(key, None),

			_ => unreachable!("handle_folders_event called with non-folders event"),
		}
	}
}
