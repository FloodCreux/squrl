use ratatui::Frame;
use ratatui::layout::Direction::{Horizontal, Vertical};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Line, Modifier, Style};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::app::App;
use crate::app::files::theme::THEME;
use crate::tui::app_states::AppState;
use crate::tui::utils::centered_rect::centered_rect;
use crate::tui::utils::stateful::cookie_table::{COOKIES_COLUMNS_NUMBER, CookieColumns};
use crate::tui::utils::stateful::text_input::SingleLineTextInput;

impl App<'_> {
	pub fn render_cookies_popup(&mut self, frame: &mut Frame) {
		let popup_block = Block::default()
			.title("Cookies")
			.borders(Borders::ALL)
			.fg(THEME.read().ui.font_color)
			.bg(THEME.read().ui.main_background_color);

		let area = centered_rect(120, 25, frame.area());

		frame.render_widget(Clear, area);
		frame.render_widget(popup_block, area);

		let horizontal_margin = 1;

		let cookies_layout = Layout::new(Vertical, [Constraint::Length(2), Constraint::Fill(1)])
			.vertical_margin(1)
			.horizontal_margin(horizontal_margin)
			.split(area);

		let inner_cookies_layout =
			Layout::new(Horizontal, CookieColumns::constraints()).split(cookies_layout[0]);

		let header_names = [
			CookieColumns::URL.to_string(),
			CookieColumns::Name.to_string(),
			CookieColumns::Value.to_string(),
			CookieColumns::Path.to_string(),
			CookieColumns::Expires.to_string(),
			CookieColumns::HttpOnly.to_string(),
			CookieColumns::Secure.to_string(),
			CookieColumns::SameSite.to_string(),
		];

		for (index, header_name) in header_names.iter().enumerate() {
			let paragraph = Paragraph::new(header_name.as_str())
				.centered()
				.block(Block::new().borders(Borders::BOTTOM | Borders::RIGHT))
				.fg(THEME.read().ui.font_color);

			frame.render_widget(paragraph, inner_cookies_layout[index]);
		}

		match self.core.cookies_popup.cookies_table.selection {
			None => {
				let cookies_lines = vec![
					Line::default(),
					Line::from("No cookies"),
					Line::from(
						"(Add one by sending a request or pressing 'n')"
							.fg(THEME.read().ui.font_color),
					),
				];

				let cookies_paragraph = Paragraph::new(cookies_lines).centered();

				frame.render_widget(cookies_paragraph, cookies_layout[1]);
			}
			Some(selection) => {
				self.render_cookie_list(selection, frame, cookies_layout[1]);
				self.render_cookie_cursor(selection, frame, cookies_layout[1]);
			}
		}
	}

	fn render_cookie_list(&mut self, selection: (usize, usize), frame: &mut Frame, area: Rect) {
		let table_layout = Layout::new(Horizontal, CookieColumns::constraints())
			.horizontal_margin(2)
			.split(area);

		let mut cookies: [Vec<ListItem>; COOKIES_COLUMNS_NUMBER] = [
			vec![],
			vec![],
			vec![],
			vec![],
			vec![],
			vec![],
			vec![],
			vec![],
		];

		for cookie in &self.core.cookies_popup.cookies_table.rows {
			for (index, value) in cookie.iter().enumerate() {
				let value = ListItem::from(value.clone());

				cookies[index].push(value);
			}
		}

		let mut list_styles = [
			Style::default(),
			Style::default(),
			Style::default(),
			Style::default(),
			Style::default(),
			Style::default(),
			Style::default(),
			Style::default(),
		];

		list_styles[selection.1] = list_styles[selection.1]
			.fg(THEME.read().others.selection_highlight_color)
			.add_modifier(Modifier::BOLD);

		for (index, cookie) in cookies.iter().enumerate() {
			let list = List::new(cookie.clone()).highlight_style(list_styles[index]);

			frame.render_stateful_widget(
				list,
				table_layout[index],
				&mut self.core.cookies_popup.cookies_table.lists_states[index].clone(),
			);
		}
	}

	fn render_cookie_cursor(&mut self, selection: (usize, usize), frame: &mut Frame, area: Rect) {
		if self.state != AppState::EditingCookies {
			return;
		}

		// Use the same layout as render_cookie_list to find exact cell positions
		let table_layout = Layout::new(Horizontal, CookieColumns::constraints())
			.horizontal_margin(2)
			.split(area);

		let col_area = table_layout[selection.1];

		let row_offset = self.core.cookies_popup.cookies_table.lists_states[0].offset();
		let visible_row = (selection.0 - row_offset) as u16;

		if visible_row >= col_area.height {
			return;
		}

		let text_rect = Rect::new(col_area.x, col_area.y + visible_row, col_area.width, 1);

		// Clear the cell area and render the text input
		let blank = " ".repeat(text_rect.width as usize);
		frame.render_widget(Paragraph::new(blank), text_rect);

		self.core
			.cookies_popup
			.cookies_table
			.selection_text_input
			.display_cursor = true;
		self.core
			.cookies_popup
			.cookies_table
			.selection_text_input
			.highlight_text = true;

		frame.render_widget(
			SingleLineTextInput(&mut self.core.cookies_popup.cookies_table.selection_text_input),
			text_rect,
		);
	}
}
