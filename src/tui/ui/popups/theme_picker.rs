use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::widgets::{
	Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

use crate::app::App;
use crate::app::files::theme::THEME;
use crate::tui::utils::centered_rect::centered_rect;

impl App<'_> {
	pub fn render_theme_picker_popup(&mut self, frame: &mut Frame) {
		let popup_block = Block::default()
			.title(" Choose Theme ")
			.borders(Borders::ALL)
			.fg(THEME.read().ui.main_foreground_color)
			.bg(THEME.read().ui.main_background_color);

		// Calculate popup size based on content
		let max_theme_name_len = self
			.theme_popup
			.themes
			.iter()
			.map(|t| t.len())
			.max()
			.unwrap_or(10);

		// +4 for padding (2 spaces on each side), +2 for borders
		let popup_width = (max_theme_name_len + 6).clamp(24, 50) as u16;
		let visible_items = 10.min(self.theme_popup.themes.len()) as u16;
		// +2 for top/bottom borders
		let popup_height = visible_items + 2;

		let area = centered_rect(popup_width, popup_height, frame.area());

		frame.render_widget(Clear, area);
		frame.render_widget(popup_block.clone(), area);

		// Get inner area (inside the borders)
		let inner_area = popup_block.inner(area);

		// Calculate which items to show (scrolling)
		let total_items = self.theme_popup.themes.len();
		let max_visible = visible_items as usize;

		let start_idx = if self.theme_popup.selection >= max_visible {
			self.theme_popup.selection - max_visible + 1
		} else {
			0
		};
		let end_idx = (start_idx + max_visible).min(total_items);

		// Create layout for visible items
		let constraints: Vec<Constraint> =
			(0..max_visible).map(|_| Constraint::Length(1)).collect();

		let items_layout = Layout::vertical(constraints).split(inner_area);

		// Render visible theme items
		for (display_idx, theme_idx) in (start_idx..end_idx).enumerate() {
			if let Some(theme_name) = self.theme_popup.themes.get(theme_idx) {
				let mut paragraph =
					Paragraph::new(format!(" {} ", theme_name)).fg(THEME.read().ui.font_color);

				if theme_idx == self.theme_popup.selection {
					paragraph = paragraph
						.fg(THEME.read().others.selection_highlight_color)
						.bold()
						.bg(THEME.read().ui.secondary_background_color);
				}

				if display_idx < items_layout.len() {
					frame.render_widget(paragraph, items_layout[display_idx]);
				}
			}
		}

		// Render scrollbar if needed
		if total_items > max_visible {
			let scrollbar = Scrollbar::default()
				.orientation(ScrollbarOrientation::VerticalRight)
				.begin_symbol(Some("▲"))
				.end_symbol(Some("▼"));

			let mut scrollbar_state =
				ScrollbarState::new(total_items).position(self.theme_popup.selection);

			frame.render_stateful_widget(scrollbar, inner_area, &mut scrollbar_state);
		}
	}
}
