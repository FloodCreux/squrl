use crate::app::app::App;
use crate::app::files::theme::THEME;
use crate::models::request::Request;
use crate::tui::app_states::AppState;
use crate::tui::ui::views::RequestView;
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

		frame.render_widget(request_block, rect);

		let inner = rect.inner(Margin::new(1, 1));
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
		let method = http_request.method.clone();

		let method_block = Block::new()
			.borders(Borders::NONE)
			.padding(Padding::horizontal(1))
			.fg(THEME.read().ui.main_foreground_color);

		let method_area = method_block.inner(request_header_layout[0]);

		let method_paragraph = Paragraph::new(format!("[ {} ]", method.to_string()))
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

		let params_block = Block::new()
			.borders(Borders::RIGHT)
			.fg(THEME.read().ui.main_foreground_color);

		let request_params_area = params_block.inner(request_layout[2]);

		frame.render_widget(params_block, request_layout[2]);
		self.render_request_params(frame, request_params_area, &request);

		// // REQUEST MAIN LAYOUT
		//
		// let request_main_layout_constraints = match self.request_view {
		// 	RequestView::Normal => [Constraint::Percentage(50), Constraint::Percentage(50)],
		// 	RequestView::OnlyResult => [Constraint::Percentage(0), Constraint::Percentage(100)],
		// 	RequestView::OnlyParams => [Constraint::Percentage(100), Constraint::Percentage(0)],
		// };
		//
		// let request_main_layout =
		// 	Layout::new(Horizontal, request_main_layout_constraints).split(request_layout[2]);
		//
		// // REQUEST RESULT LAYOUT
		//
		// if should_render_result {
		// 	let result_block = Block::new().fg(THEME.read().ui.main_foreground_color);
		// 	let result_block_area = result_block.inner(request_main_layout[1]);
		//
		// 	frame.render_widget(result_block, request_main_layout[1]);
		// 	self.render_request_result(frame, result_block_area, &request);
		// }
	}
}
