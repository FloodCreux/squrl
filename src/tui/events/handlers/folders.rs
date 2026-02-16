use crokey::KeyCombination;

use crate::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) fn handle_folders_event(
		&mut self,
		event: &AppEvent,
		key: KeyCombination,
	) {
		match event {
			AppEvent::CreateNewFolder(_) => match self
				.collection_popups
				.new_collection_input
				.is_in_default_mode()
			{
				true => self.tui_new_folder(),
				false => self
					.collection_popups
					.new_collection_input
					.key_event(key, None),
			},
			AppEvent::CancelCreateNewFolder(_) => {
				match self
					.collection_popups
					.new_collection_input
					.is_in_default_mode()
				{
					true => self.normal_state(),
					false => self
						.collection_popups
						.new_collection_input
						.key_event(key, None),
				}
			}
			AppEvent::KeyEventCreateNewFolder(_) => self
				.collection_popups
				.new_collection_input
				.key_event(key, None),

			AppEvent::DeletingFolderMoveCursorLeft(_) => self
				.collection_popups
				.delete_collection_popup
				.change_state(),
			AppEvent::DeletingFolderMoveCursorRight(_) => self
				.collection_popups
				.delete_collection_popup
				.change_state(),
			AppEvent::DeleteFolder(_) => match self.collection_popups.delete_collection_popup.state
			{
				true => self.tui_delete_folder(),
				false => self.normal_state(),
			},

			AppEvent::RenameFolder(_) => match self
				.collection_popups
				.rename_collection_input
				.is_in_default_mode()
			{
				true => self.tui_rename_folder(),
				false => self
					.collection_popups
					.rename_collection_input
					.key_event(key, None),
			},
			AppEvent::CancelRenameFolder(_) => {
				match self
					.collection_popups
					.rename_collection_input
					.is_in_default_mode()
				{
					true => self.normal_state(),
					false => self
						.collection_popups
						.rename_collection_input
						.key_event(key, None),
				}
			}
			AppEvent::KeyEventRenameFolder(_) => self
				.collection_popups
				.rename_collection_input
				.key_event(key, None),

			_ => unreachable!("handle_folders_event called with non-folders event"),
		}
	}
}
