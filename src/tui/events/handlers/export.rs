use crokey::KeyCombination;

use crate::app::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) fn handle_export_event(
		&mut self,
		event: &AppEvent,
		_key: KeyCombination,
	) {
		match event {
			AppEvent::ExportRequest(_) => self.choose_request_export_format_state(),

			AppEvent::RequestExportFormatMoveCursorLeft(_) => self.export_request.previous(),
			AppEvent::RequestExportFormatMoveCursorRight(_) => self.export_request.next(),

			AppEvent::SelectRequestExportFormat(_) => self.tui_export_request(),

			AppEvent::ScrollRequestExportUp(_) => {
				self.display_request_export.vertical_scrollbar.page_up()
			}
			AppEvent::ScrollRequestExportDown(_) => {
				self.display_request_export.vertical_scrollbar.page_down()
			}
			AppEvent::ScrollRequestExportLeft(_) => {
				self.display_request_export.horizontal_scrollbar.page_up()
			}
			AppEvent::ScrollRequestExportRight(_) => {
				self.display_request_export.horizontal_scrollbar.page_down()
			}

			#[cfg(feature = "clipboard")]
			AppEvent::CopyRequestExport(_) => self.copy_request_export_to_clipboard(),

			#[cfg(not(feature = "clipboard"))]
			AppEvent::CopyRequestExport(_) => {}

			_ => unreachable!("handle_export_event called with non-export event"),
		}
	}
}
