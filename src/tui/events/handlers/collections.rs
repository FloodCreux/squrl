use crokey::KeyCombination;

use crate::app::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) fn handle_collections_event(
		&mut self,
		event: &AppEvent,
		key: KeyCombination,
	) {
		match event {
			AppEvent::ChooseElementToCreateMoveCursorLeft(_) => self.creation_popup.previous(),
			AppEvent::ChooseElementToCreateMoveCursorRight(_) => self.creation_popup.next(),
			AppEvent::SelectElementToCreate(_) => self.new_element(),

			AppEvent::CreateNewCollection(_) => {
				match self.new_collection_input.is_in_default_mode() {
					true => self.tui_new_collection(),
					false => self.new_collection_input.key_event(key, None),
				}
			}
			AppEvent::CancelCreateNewCollection(_) => {
				match self.new_collection_input.is_in_default_mode() {
					true => self.normal_state(),
					false => self.new_collection_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventCreateNewCollection(_) => {
				self.new_collection_input.key_event(key, None)
			}

			AppEvent::CreateNewRequest(_) => {
				match self.new_request_popup.text_input.is_in_default_mode() {
					true => self.tui_new_request(),
					false => self.new_request_popup.text_input.key_event(key, None),
				}
			}
			AppEvent::CancelCreateNewRequest(_) => {
				match self.new_request_popup.text_input.is_in_default_mode() {
					true => self.normal_state(),
					false => self.new_request_popup.text_input.key_event(key, None),
				}
			}
			AppEvent::CreatingRequestSelectInputUp(_) => self.new_request_popup.previous_input(),
			AppEvent::CreatingRequestSelectInputDown(_) => self.new_request_popup.next_input(),
			AppEvent::CreatingRequestInputLeft(_) => self.new_request_popup.input_left(),
			AppEvent::CreatingRequestInputRight(_) => self.new_request_popup.input_right(),
			AppEvent::KeyEventCreateNewRequest(_) => {
				self.new_request_popup.text_input.key_event(key, None)
			}

			AppEvent::DeletingCollectionMoveCursorLeft(_) => {
				self.delete_collection_popup.change_state()
			}
			AppEvent::DeletingCollectionMoveCursorRight(_) => {
				self.delete_collection_popup.change_state()
			}
			AppEvent::DeleteCollection(_) => match self.delete_collection_popup.state {
				true => self.tui_delete_collection(),
				false => self.normal_state(),
			},

			AppEvent::DeletingRequestMoveCursorLeft(_) => self.delete_request_popup.change_state(),
			AppEvent::DeletingRequestMoveCursorRight(_) => self.delete_request_popup.change_state(),
			AppEvent::DeleteRequest(_) => match self.delete_request_popup.state {
				true => self.tui_delete_request(),
				false => self.normal_state(),
			},

			AppEvent::RenameCollection(_) => {
				match self.rename_collection_input.is_in_default_mode() {
					true => self.tui_rename_collection(),
					false => self.rename_collection_input.key_event(key, None),
				}
			}
			AppEvent::CancelRenameCollection(_) => {
				match self.rename_collection_input.is_in_default_mode() {
					true => self.normal_state(),
					false => self.rename_collection_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventRenameCollection(_) => {
				self.rename_collection_input.key_event(key, None)
			}

			AppEvent::RenameRequest(_) => match self.rename_request_input.is_in_default_mode() {
				true => self.tui_rename_request(),
				false => self.rename_request_input.key_event(key, None),
			},
			AppEvent::CancelRenameRequest(_) => {
				match self.rename_request_input.is_in_default_mode() {
					true => self.normal_state(),
					false => self.rename_request_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventRenameRequest(_) => self.rename_request_input.key_event(key, None),

			_ => unreachable!("handle_collections_event called with non-collections event"),
		}
	}
}
