use ratatui::Frame;
use ratatui::layout::Direction::{Horizontal, Vertical};
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::app::App;
use crate::app::files::theme::THEME;
use crate::models::request::Request;
use crate::tui::app_states::AppState;
use crate::tui::utils::stateful::text_input::SingleLineTextInput;

impl App<'_> {
	pub fn render_ws_request(&mut self, frame: &mut Frame, rect: Rect, request: Request) {
		let request_name = request.name.clone();
		let layout = Layout::new(
			Vertical,
			[Constraint::Percentage(30), Constraint::Percentage(70)],
		)
		.split(rect);

		// REQUEST BLOCK (top 30%)

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

		// REQUEST NAME

		let request_name_paragraph = Paragraph::new(request_name)
			.centered()
			.fg(THEME.read().ui.font_color);

		frame.render_widget(request_name_paragraph, request_layout[0]);

		// REQUEST HEADER LAYOUT

		let request_header_layout = Layout::new(
			Horizontal,
			[Constraint::Percentage(10), Constraint::Percentage(90)],
		)
		.split(request_layout[1]);

		// REQUEST CONNECTION STATUS

		let ws_request = request
			.get_ws_request()
			.expect("request should be WebSocket");

		let connection_status_block = Block::new()
			.borders(Borders::NONE)
			.padding(Padding::horizontal(1))
			.fg(THEME.read().ui.main_foreground_color);

		let connection_status_area = connection_status_block.inner(request_header_layout[0]);

		let connection_status_paragraph = match ws_request.is_connected {
			true => Paragraph::new("[ CONNECTED ]")
				.style(Modifier::BOLD)
				.fg(THEME.read().websocket.connection_status.connected)
				// .fg(THEME.read().ui.font_color)
				.centered(),
			false => Paragraph::new("[ DISCONNECTED ]")
				.fg(THEME.read().websocket.connection_status.disconnected)
				// .fg(THEME.read().ui.font_color)
				.centered(),
		};

		frame.render_widget(connection_status_block, request_header_layout[0]);
		frame.render_widget(connection_status_paragraph, connection_status_area);

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

		// RESPONSE BLOCK (bottom 70%)

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
