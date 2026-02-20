use crate::app::App;
use crate::app::files::theme::THEME;
use crate::models::protocol::protocol::Protocol;
use crate::models::request::Request;
use crate::tui::app_states::AppState;
use crate::tui::utils::stateful::text_input::SingleLineTextInput;
use ratatui::Frame;
use ratatui::layout::Direction::{Horizontal, Vertical};
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

impl App<'_> {
	pub fn render_http_request(&mut self, frame: &mut Frame, rect: Rect, request: Request) {
		let request_name = request.name.clone();
		let layout = Layout::new(
			Vertical,
			[Constraint::Percentage(30), Constraint::Percentage(70)],
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

		let (method_label, method_color) = match &request.protocol {
			Protocol::HttpRequest(http_request) => (
				http_request.method.to_string(),
				http_request.method.get_color(),
			),
			Protocol::GraphqlRequest(_) => {
				("GQL".to_string(), THEME.read().ui.main_foreground_color)
			}
			_ => ("???".to_string(), Color::default()),
		};

		let method_block = Block::new()
			.borders(Borders::NONE)
			.padding(Padding::horizontal(1))
			.fg(THEME.read().ui.main_foreground_color);

		let method_area = method_block.inner(request_header_layout[0]);

		let method_paragraph = Paragraph::new(format!("[ {} ]", method_label))
			.style(Modifier::BOLD)
			.fg(method_color)
			.centered();

		frame.render_widget(method_block, request_header_layout[0]);
		frame.render_widget(method_paragraph, method_area);

		// REQUEST URL

		self.request_editor.url_input.display_cursor =
			matches!(self.state, AppState::EditingRequestUrl);
		frame.render_widget(
			SingleLineTextInput(&mut self.request_editor.url_input),
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
