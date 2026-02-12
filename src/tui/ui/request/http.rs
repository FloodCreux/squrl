use crate::app::app::App;
use crate::app::files::theme::THEME;
use crate::models::request::Request;
use crate::tui::app_states::AppState;
use crate::tui::utils::stateful::text_input::SingleLineTextInput;
use ratatui::Frame;
use ratatui::layout::Direction::{Horizontal, Vertical};
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::Modifier;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

impl App<'_> {
	pub fn render_http_request(&mut self, frame: &mut Frame, rect: Rect, request: Request) {
		let request_name = request.name.clone();
		let layout = Layout::new(
			Vertical,
			[Constraint::Percentage(25), Constraint::Percentage(75)],
		)
		.split(rect);

		let request_block = Block::default()
			.title(format!(" REQUEST: {} ", request_name))
			.title_alignment(Alignment::Left)
			.borders(Borders::ALL)
			.border_set(border::Set {
				vertical_left: " ",
				vertical_right: " ",
				..border::PLAIN
			})
			.fg(THEME.read().ui.font_color);

		frame.render_widget(request_block, layout[0]);

		let inner = layout[0].inner(Margin::new(1, 1));
		let request_layout = Layout::new(
			Vertical,
			[
				Constraint::Length(1),
				Constraint::Length(2),
				Constraint::Fill(1),
			],
		)
		.split(inner);

		// REQUEST HEADER LAYOUT

		let request_header_layout = Layout::new(
			Horizontal,
			[Constraint::Percentage(10), Constraint::Percentage(90)],
		)
		.split(request_layout[1]);

		// REQUEST METHOD

		let http_request = request.get_http_request().unwrap();
		let method = http_request.method;

		let method_block = Block::new()
			.borders(Borders::NONE)
			.padding(Padding::horizontal(1))
			.fg(THEME.read().ui.main_foreground_color);

		let method_area = method_block.inner(request_header_layout[0]);

		let method_paragraph = Paragraph::new(format!("[ {} ]", method))
			.style(Modifier::BOLD)
			.fg(method.get_color())
			.centered();

		frame.render_widget(method_block, request_header_layout[0]);
		frame.render_widget(method_paragraph, method_area);

		// REQUEST URL

		self.url_text_input.display_cursor = matches!(self.state, AppState::EditingRequestUrl);
		frame.render_widget(
			SingleLineTextInput(&mut self.url_text_input),
			request_header_layout[1],
		);

		// REQUEST PARAMS

		let params_block = Block::new().fg(THEME.read().ui.main_foreground_color);

		let request_params_area = params_block.inner(request_layout[2]);

		frame.render_widget(params_block, request_layout[2]);
		self.render_request_params(frame, request_params_area, &request);

		let result_block = Block::default()
			.title(" RESPONSE ")
			.title_alignment(Alignment::Left)
			.borders(Borders::ALL)
			.border_set(border::Set {
				vertical_left: " ",
				vertical_right: " ",
				..border::PLAIN
			})
			.fg(THEME.read().ui.font_color);

		let result_block_area = result_block.inner(layout[1]);

		frame.render_widget(result_block, layout[1]);
		self.render_request_result(frame, result_block_area, &request);
	}
}
