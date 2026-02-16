use crokey::KeyCombination;

use crate::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) fn handle_collections_event(
		&mut self,
		event: &AppEvent,
		key: KeyCombination,
	) {
		match event {
			AppEvent::ChooseElementToCreateMoveCursorLeft(_) => {
				self.collection_popups.creation_popup.previous()
			}
			AppEvent::ChooseElementToCreateMoveCursorRight(_) => {
				self.collection_popups.creation_popup.next()
			}
			AppEvent::SelectElementToCreate(_) => self.new_element(),

			AppEvent::CreateNewCollection(_) => {
				match self
					.collection_popups
					.new_collection_input
					.is_in_default_mode()
				{
					true => self.tui_new_collection(),
					false => self
						.collection_popups
						.new_collection_input
						.key_event(key, None),
				}
			}
			AppEvent::CancelCreateNewCollection(_) => {
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
			AppEvent::KeyEventCreateNewCollection(_) => self
				.collection_popups
				.new_collection_input
				.key_event(key, None),

			AppEvent::CreateNewRequest(_) => {
				match self
					.collection_popups
					.new_request_popup
					.text_input
					.is_in_default_mode()
				{
					true => self.tui_new_request(),
					false => self
						.collection_popups
						.new_request_popup
						.text_input
						.key_event(key, None),
				}
			}
			AppEvent::CancelCreateNewRequest(_) => {
				match self
					.collection_popups
					.new_request_popup
					.text_input
					.is_in_default_mode()
				{
					true => self.normal_state(),
					false => self
						.collection_popups
						.new_request_popup
						.text_input
						.key_event(key, None),
				}
			}
			AppEvent::CreatingRequestSelectInputUp(_) => {
				self.collection_popups.new_request_popup.previous_input()
			}
			AppEvent::CreatingRequestSelectInputDown(_) => {
				self.collection_popups.new_request_popup.next_input()
			}
			AppEvent::CreatingRequestInputLeft(_) => {
				self.collection_popups.new_request_popup.input_left()
			}
			AppEvent::CreatingRequestInputRight(_) => {
				self.collection_popups.new_request_popup.input_right()
			}
			AppEvent::KeyEventCreateNewRequest(_) => self
				.collection_popups
				.new_request_popup
				.text_input
				.key_event(key, None),

			AppEvent::DeletingCollectionMoveCursorLeft(_) => self
				.collection_popups
				.delete_collection_popup
				.change_state(),
			AppEvent::DeletingCollectionMoveCursorRight(_) => self
				.collection_popups
				.delete_collection_popup
				.change_state(),
			AppEvent::DeleteCollection(_) => {
				match self.collection_popups.delete_collection_popup.state {
					true => self.tui_delete_collection(),
					false => self.normal_state(),
				}
			}

			AppEvent::DeletingRequestMoveCursorLeft(_) => {
				self.collection_popups.delete_request_popup.change_state()
			}
			AppEvent::DeletingRequestMoveCursorRight(_) => {
				self.collection_popups.delete_request_popup.change_state()
			}
			AppEvent::DeleteRequest(_) => match self.collection_popups.delete_request_popup.state {
				true => self.tui_delete_request(),
				false => self.normal_state(),
			},

			AppEvent::RenameCollection(_) => {
				match self
					.collection_popups
					.rename_collection_input
					.is_in_default_mode()
				{
					true => self.tui_rename_collection(),
					false => self
						.collection_popups
						.rename_collection_input
						.key_event(key, None),
				}
			}
			AppEvent::CancelRenameCollection(_) => {
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
			AppEvent::KeyEventRenameCollection(_) => self
				.collection_popups
				.rename_collection_input
				.key_event(key, None),

			AppEvent::RenameRequest(_) => match self
				.collection_popups
				.rename_request_input
				.is_in_default_mode()
			{
				true => self.tui_rename_request(),
				false => self
					.collection_popups
					.rename_request_input
					.key_event(key, None),
			},
			AppEvent::CancelRenameRequest(_) => {
				match self
					.collection_popups
					.rename_request_input
					.is_in_default_mode()
				{
					true => self.normal_state(),
					false => self
						.collection_popups
						.rename_request_input
						.key_event(key, None),
				}
			}
			AppEvent::KeyEventRenameRequest(_) => self
				.collection_popups
				.rename_request_input
				.key_event(key, None),

			_ => unreachable!("handle_collections_event called with non-collections event"),
		}
	}
}
