use ratatui::Frame;
use ratatui::layout::Direction::Vertical;
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Color, Stylize};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::app::App;
use crate::app::files::theme::THEME;
use crate::tui::utils::centered_rect::centered_rect;
use crate::tui::utils::stateful::text_input::SingleLineTextInput;

impl App<'_> {
	pub fn render_creating_new_request_popup(&mut self, frame: &mut Frame) {
		let popup_block = Block::default()
			.borders(Borders::ALL)
			.fg(THEME.read().ui.main_foreground_color)
			.bg(THEME.read().ui.main_background_color);

		let area = centered_rect(50, 12, frame.area());

		let new_request_layout = Layout::new(
			Vertical,
			vec![
				Constraint::Length(3),
				Constraint::Length(3),
				Constraint::Length(3),
				Constraint::Length(3),
			],
		)
		.split(area);

		// Sync folder_count from the actual collection data (needed after collection changes)
		let collection = &self.core.collections[self.new_request_popup.selected_collection];
		self.new_request_popup.folder_count = collection.folders.len();

		// Collection selector (row 0)
		let selected_collection_name = collection.name.clone();
		let selection_collection_block_color = match self.new_request_popup.selection == 0 {
			true => Color::Yellow,
			false => THEME.read().ui.main_foreground_color,
		};
		let selected_collection_paragraph = Paragraph::new(selected_collection_name)
			.fg(THEME.read().ui.font_color)
			.block(
				Block::new()
					.title("Collection ← →")
					.borders(Borders::ALL)
					.fg(selection_collection_block_color),
			);

		// Folder selector (row 1)
		let selected_folder_name = match self.new_request_popup.selected_folder {
			None => "None (root)".to_string(),
			Some(folder_index) if folder_index < self.new_request_popup.folder_count => {
				self.core.collections[self.new_request_popup.selected_collection].folders
					[folder_index]
					.name
					.clone()
			}
			Some(_) => "None (root)".to_string(),
		};
		let selection_folder_block_color = match self.new_request_popup.selection == 1 {
			true => Color::Yellow,
			false => THEME.read().ui.main_foreground_color,
		};
		let selected_folder_paragraph = Paragraph::new(selected_folder_name)
			.fg(THEME.read().ui.font_color)
			.block(
				Block::new()
					.title("Folder ← →")
					.borders(Borders::ALL)
					.fg(selection_folder_block_color),
			);

		// Protocol selector (row 2)
		let selected_protocol_name = self.new_request_popup.protocol.to_string();
		let selected_protocol_block_color = match self.new_request_popup.selection == 2 {
			true => Color::Yellow,
			false => THEME.read().ui.main_foreground_color,
		};
		let selected_protocol_paragraph = Paragraph::new(selected_protocol_name)
			.fg(THEME.read().ui.font_color)
			.block(
				Block::new()
					.title("Protocol ← →")
					.borders(Borders::ALL)
					.fg(selected_protocol_block_color),
			);

		// Name input (row 3)
		let highlight_and_display_cursor = self.new_request_popup.selection == 3;

		frame.render_widget(Clear, area);
		frame.render_widget(popup_block, area);
		frame.render_widget(selected_collection_paragraph, new_request_layout[0]);
		frame.render_widget(selected_folder_paragraph, new_request_layout[1]);
		frame.render_widget(selected_protocol_paragraph, new_request_layout[2]);

		self.new_request_popup.text_input.highlight_text = highlight_and_display_cursor;
		self.new_request_popup.text_input.highlight_block = highlight_and_display_cursor;
		self.new_request_popup.text_input.display_cursor = highlight_and_display_cursor;

		frame.render_widget(
			SingleLineTextInput(&mut self.new_request_popup.text_input),
			new_request_layout[3],
		);
	}
}
