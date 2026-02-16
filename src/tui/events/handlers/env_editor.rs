use crokey::KeyCombination;

use crate::app::App;
use crate::tui::events::AppEvent;
use crate::tui::utils::stateful::table_navigation::TableNavigation;

impl App<'_> {
	pub(in crate::tui::events) fn handle_env_editor_event(
		&mut self,
		event: &AppEvent,
		key: KeyCombination,
	) {
		match event {
			AppEvent::EditEnvVariable(_) => {
				if self.env_editor_table.is_selected() {
					self.edit_env_variable_state()
				}
			}
			AppEvent::EnvVariablesMoveUp(_) => self.env_editor_table.up(),
			AppEvent::EnvVariablesMoveDown(_) => self.env_editor_table.down(),
			AppEvent::EnvVariablesMoveLeft(_) | AppEvent::EnvVariablesMoveRight(_) => {
				self.env_editor_table.change_y()
			}
			AppEvent::CreateEnvVariable(_) => self.tui_create_env_variable(),
			AppEvent::DeleteEnvVariable(_) => self.tui_delete_env_variable(),

			AppEvent::ModifyEnvVariable(_) => match self
				.env_editor_table
				.selection_text_input
				.is_in_default_mode()
			{
				true => self.tui_modify_env_variable(),
				false => self
					.env_editor_table
					.selection_text_input
					.key_event(key, None),
			},
			AppEvent::CancelModifyEnvVariable(_) => match self
				.env_editor_table
				.selection_text_input
				.is_in_default_mode()
			{
				true => self.display_env_editor_state(),
				false => self
					.env_editor_table
					.selection_text_input
					.key_event(key, None),
			},
			AppEvent::KeyEventModifyEnvVariable(_) => self
				.env_editor_table
				.selection_text_input
				.key_event(key, None),

			_ => unreachable!("handle_env_editor_event called with non-env-editor event"),
		}
	}
}
