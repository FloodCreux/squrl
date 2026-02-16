use chrono::Local;
use ratatui::Frame;
use ratatui::layout::Direction::Vertical;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::prelude::{Alignment, Style};
use ratatui::style::{Color, Modifier, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, Tabs, Wrap};
use ratatui_image::StatefulImage;
use ratatui_image::picker::Picker;
use rayon::prelude::*;
use strum::{Display, EnumIter, FromRepr};
use textwrap::wrap;
use throbber_widgets_tui::{BRAILLE_DOUBLE, Throbber, WhichUse};

use crate::app::App;
use crate::app::files::theme::THEME;
use crate::models::protocol::protocol::Protocol;
use crate::models::protocol::ws::ws::Sender;
use crate::models::request::Request;
use crate::models::response::ResponseContent;
use crate::tui::app_states::AppState;
use crate::tui::utils::centered_rect::centered_rect;
use crate::tui::utils::stateful::text_input::MultiLineTextInput;
use crate::tui::utils::syntax_highlighting::SYNTAX_SET;

#[derive(Default, Clone, Copy, Debug, PartialOrd, PartialEq, Display, FromRepr, EnumIter)]
pub enum RequestResultTabs {
	#[default]
	#[strum(to_string = "RESULT BODY")]
	Body,
	#[strum(to_string = "MESSAGES")]
	Messages,
	#[strum(to_string = "COOKIES")]
	Cookies,
	#[strum(to_string = "HEADERS")]
	Headers,
	#[strum(to_string = "CONSOLE")]
	Console,
}

impl App<'_> {
	pub(super) fn render_request_result(
		&mut self,
		frame: &mut Frame,
		rect: Rect,
		request: &Request,
	) {
		let request_result_layout = Layout::new(
			Vertical,
			[
				Constraint::Length(2),
				Constraint::Length(1),
				Constraint::Fill(1),
			],
		)
		.split(rect);

		// REQUEST RESULT TABS

		let allowed_tabs = match &request.protocol {
			Protocol::HttpRequest(_) => vec![
				RequestResultTabs::Body,
				RequestResultTabs::Cookies,
				RequestResultTabs::Headers,
				RequestResultTabs::Console,
			],
			Protocol::WsRequest(_) => vec![
				RequestResultTabs::Messages,
				RequestResultTabs::Cookies,
				RequestResultTabs::Headers,
				RequestResultTabs::Console,
			],
		};

		let selected_request_tab_index = match &request.protocol {
			Protocol::HttpRequest(_) => match self.request_result_tab {
				RequestResultTabs::Body => 0,
				RequestResultTabs::Cookies => 1,
				RequestResultTabs::Headers => 2,
				RequestResultTabs::Console => 3,
				_ => unreachable!(),
			},
			Protocol::WsRequest(_) => match self.request_result_tab {
				RequestResultTabs::Messages => 0,
				RequestResultTabs::Cookies => 1,
				RequestResultTabs::Headers => 2,
				RequestResultTabs::Console => 3,
				_ => unreachable!(),
			},
		};

		let tab_texts: Vec<String> = allowed_tabs
			.iter()
			.map(|tab| match tab {
				RequestResultTabs::Body => tab.to_string().to_uppercase(),
				RequestResultTabs::Messages => tab.to_string().to_uppercase(),
				RequestResultTabs::Cookies | RequestResultTabs::Headers => {
					tab.to_string().to_uppercase()
				}
				RequestResultTabs::Console => tab.to_string().to_uppercase(),
			})
			.collect();

		let max_tab_width = tab_texts.iter().map(|t| t.len()).max().unwrap_or(0) + 4;

		let result_tabs = tab_texts.iter().enumerate().map(|(index, text)| {
			if index == selected_request_tab_index {
				let inner_width = max_tab_width - 2; // total width minus "[" and "]"
				let padded = format!("{:^inner_width$}", text);

				Line::from(vec![
					Span::raw("[")
						.style(Modifier::BOLD)
						.fg(THEME.read().ui.font_color)
						.bg(Color::Reset),
					Span::raw(padded)
						.fg(THEME.read().ui.main_background_color)
						.bg(THEME.read().ui.font_color),
					Span::raw("]")
						.style(Modifier::BOLD)
						.fg(THEME.read().ui.font_color)
						.bg(Color::Reset),
				])
			} else {
				let padded = format!("{:^max_tab_width$}", text);
				Line::from(padded).fg(THEME.read().ui.font_color)
			}
		});

		let result_tabs = Tabs::new(result_tabs).select(None::<usize>).block(
			Block::new()
				.borders(Borders::BOTTOM)
				.fg(THEME.read().ui.main_foreground_color),
		);

		frame.render_widget(result_tabs, request_result_layout[0]);

		// If the selected request is currently pending
		if request.is_pending {
			let area = centered_rect(9, 1, request_result_layout[2]);

			self.response_view.throbber_state.calc_next();

			let throbber = Throbber::default()
				.label("Pending")
				.style(Style::new().fg(THEME.read().ui.secondary_foreground_color))
				.throbber_set(BRAILLE_DOUBLE)
				.use_type(WhichUse::Spin);

			frame.render_stateful_widget(throbber, area, &mut self.response_view.throbber_state);
		}
		// If the selected request is not pending
		else {
			// REQUEST RESULT STATUS LINE

			let status_code = match &request.response.status_code {
				None => "",
				Some(status_code) => status_code,
			};

			let request_duration = match &request.response.duration {
				None => "",
				Some(duration) => duration,
			};

			let request_size = match &request.response.content {
				None => "0 KB".to_string(),
				Some(content) => match content {
					ResponseContent::Body(body) => {
						let size_in_kb = body.len() as f64 / 1024.0;
						format!("{} KB", size_in_kb)
					}
					ResponseContent::Image(img) => {
						let size_in_kb = img.data.len() as f64 / 1024.0;
						format!("{:.2} KB", size_in_kb)
					}
				},
			};

			let status_chunks =
				Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
					.split(request_result_layout[1]);

			let status_line = Line::from(vec![
				Span::styled("STATUS: ", Style::default().fg(Color::White)),
				Span::styled(
					status_code,
					Style::default().fg(status_code_color(status_code)),
				),
				Span::raw("   "),
				Span::styled("TIME: ", Style::default().fg(Color::White)),
				Span::styled(request_duration, Style::default().fg(Color::Yellow)),
				Span::raw("   "),
				Span::styled("SIZE: ", Style::default().fg(Color::White)),
				Span::styled(request_size, Style::default().fg(Color::Blue)),
			])
			.bg(Color::Gray)
			.style(Style::default().add_modifier(Modifier::BOLD));
			frame.render_widget(status_line, status_chunks[0]);

			// REQUEST RESULT CONTENT

			match self.request_result_tab {
				RequestResultTabs::Body => match &request.response.content {
					None => {}
					Some(content) => match content {
						ResponseContent::Body(body) => {
							// If in selection mode, render the TextInput instead
							if self.state == AppState::SelectingResponseBody {
								let syntax = SYNTAX_SET.find_syntax_plain_text().clone();
								frame.render_widget(
									MultiLineTextInput(
										&mut self.response_view.body_text_area,
										syntax,
									),
									request_result_layout[2],
								);
							} else {
								let lines: Vec<Line> =
									if !self.core.config.is_syntax_highlighting_disabled()
										&& self.syntax_highlighting.highlighted_body.is_some()
									{
										self.syntax_highlighting
											.highlighted_body
											.clone()
											.expect("highlighted body should exist")
									} else {
										body.lines().map(Line::raw).collect()
									};

								let mut body_paragraph = Paragraph::new(lines);

								if self.core.config.should_wrap_body() {
									body_paragraph = body_paragraph
										.wrap(Wrap::default())
										.scroll((self.response_view.vertical_scrollbar.scroll, 0));
								} else {
									body_paragraph = body_paragraph.scroll((
										self.response_view.vertical_scrollbar.scroll,
										self.response_view.horizontal_scrollbar.scroll,
									));
								}

								frame.render_widget(body_paragraph, request_result_layout[2]);
							}
						}
						ResponseContent::Image(image_response) => match &image_response.image {
							_ if self.core.config.is_image_preview_disabled() => {
								let image_disabled_paragraph =
									Paragraph::new("\nImage preview disabled").centered();
								frame.render_widget(
									image_disabled_paragraph,
									request_result_layout[2],
								);
							}
							Some(image) => {
								let picker =
									match self.core.config.is_graphical_protocol_disabled() {
										true => Picker::halfblocks(),
										false => Picker::from_query_stdio()
											.unwrap_or(Picker::halfblocks()),
									};

								let mut image_static = picker.new_resize_protocol(image.clone());

								frame.render_stateful_widget(
									StatefulImage::default(),
									request_result_layout[2],
									&mut image_static,
								);
							}
							None => {
								let image_error_paragraph =
									Paragraph::new("\nCould not decode image")
										.centered()
										.fg(THEME.read().ui.font_color);
								frame
									.render_widget(image_error_paragraph, request_result_layout[2]);
							}
						},
					},
				},
				RequestResultTabs::Messages => {
					let ws_request = request
						.get_ws_request()
						.expect("request should be WebSocket");

					let mut messages = vec![];
					let mut last_sender: Option<&Sender> = None;

					for message in &ws_request.messages {
						let mut alignment = Alignment::Right;

						let content = message.content.to_content();
						let max_length = self.get_max_line_length(&content);
						let lines = wrap(&content, max_length);

						match message.sender {
							Sender::You => {
								for line in lines {
									let line = match line.is_empty() {
										true => " ".repeat(max_length),
										false => format!("{line:max_length$}"),
									};

									messages.push(
										Line::raw(line)
											.fg(THEME.read().ui.font_color)
											.bg(THEME
												.read()
												.websocket
												.messages
												.you_background_color)
											.alignment(alignment),
									);
								}
							}
							Sender::Server => {
								alignment = Alignment::Left;

								if last_sender != Some(&message.sender) {
									messages.push(
										Line::raw(message.sender.to_string())
											.bold()
											.fg(THEME
												.read()
												.websocket
												.messages
												.server_foreground_color)
											.alignment(alignment),
									);
								}

								for line in lines {
									let line = match line.is_empty() {
										true => " ".repeat(max_length),
										false => format!("{line:max_length$}"),
									};

									messages.push(
										Line::raw(line)
											.fg(THEME.read().ui.font_color)
											.bg(THEME
												.read()
												.websocket
												.messages
												.server_background_color)
											.alignment(alignment),
									);
								}
							}
						}

						let timestamp_format =
							match Local::now().date_naive() == message.timestamp.date_naive() {
								true => "%H:%M:%S",
								false => "%H:%M:%S %d/%m/%Y",
							};

						let timestamp = message.timestamp.format(timestamp_format).to_string();

						messages.push(
							Line::raw(format!("{} {}", message.content, timestamp))
								.fg(THEME.read().websocket.messages.details_color)
								.alignment(alignment),
						);

						last_sender = Some(&message.sender);
					}

					let messages_paragraph = Paragraph::new(messages).scroll((
						self.response_view.vertical_scrollbar.scroll,
						self.response_view.horizontal_scrollbar.scroll,
					));

					let inner_area = Rect {
						x: request_result_layout[2].x,
						y: request_result_layout[2].y,
						width: request_result_layout[2].width.saturating_sub(2),
						height: request_result_layout[2].height,
					};

					frame.render_widget(messages_paragraph, inner_area);
				}
				RequestResultTabs::Cookies => {
					let result_cookies = match &request.response.cookies {
						None => "",
						Some(cookies) => cookies,
					};

					let cookies_paragraph = Paragraph::new(result_cookies)
						.fg(THEME.read().ui.font_color)
						.scroll((
							self.response_view.vertical_scrollbar.scroll,
							self.response_view.horizontal_scrollbar.scroll,
						));

					frame.render_widget(cookies_paragraph, request_result_layout[2]);
				}
				RequestResultTabs::Headers => {
					let result_headers: Vec<Line> = request
						.response
						.headers
						.par_iter()
						.map(|(header, value)| {
							Line::from(vec![
								Span::raw(header)
									.bold()
									.fg(THEME.read().ui.secondary_foreground_color),
								Span::raw(": ").fg(THEME.read().ui.secondary_foreground_color),
								Span::raw(value).fg(THEME.read().ui.font_color),
							])
						})
						.collect();

					let headers_paragraph = Paragraph::new(result_headers).scroll((
						self.response_view.vertical_scrollbar.scroll,
						self.response_view.horizontal_scrollbar.scroll,
					));

					frame.render_widget(headers_paragraph, request_result_layout[2]);
				}
				RequestResultTabs::Console => {
					let console_paragraph =
						Paragraph::new(self.syntax_highlighting.highlighted_console_output.clone())
							.scroll((
								self.response_view.vertical_scrollbar.scroll,
								self.response_view.horizontal_scrollbar.scroll,
							));

					frame.render_widget(console_paragraph, request_result_layout[2]);
				}
			};
		}

		let result_vertical_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
			.style(Style::new().fg(THEME.read().ui.font_color));
		let result_horizontal_scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
			.style(Style::new().fg(THEME.read().ui.font_color))
			.thumb_symbol("■"); // Better than the default full block

		frame.render_stateful_widget(
			result_vertical_scrollbar,
			rect.inner(Margin {
				// using an inner vertical margin of 1 unit makes the scrollbar inside the block
				vertical: 1,
				horizontal: 0,
			}),
			&mut self.response_view.vertical_scrollbar.state,
		);

		if !(self.core.config.should_wrap_body()
			&& self.request_result_tab == RequestResultTabs::Body)
		{
			frame.render_stateful_widget(
				result_horizontal_scrollbar,
				rect.inner(Margin {
					// using an inner vertical margin of 1 unit makes the scrollbar inside the block
					vertical: 0,
					horizontal: 1,
				}),
				&mut self.response_view.horizontal_scrollbar.state,
			);
		}

		self.last_messages_area_size.0 = request_result_layout[2].width.saturating_sub(1);
		self.last_messages_area_size.1 = request_result_layout[2].height.saturating_sub(1);
	}
}

fn status_code_color(code: &str) -> Color {
	match code.as_bytes().first() {
		Some(b'2') => Color::Green,
		Some(b'3') => Color::Cyan,
		Some(b'4') => Color::Yellow,
		Some(b'5') => Color::Red,
		_ => Color::DarkGray,
	}
}
