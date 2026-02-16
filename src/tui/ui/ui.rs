use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::layout::Direction::{Horizontal, Vertical};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::prelude::Modifier;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders};

use crate::app::App;
use crate::app::files::theme::THEME;
use crate::models::protocol::protocol::Protocol;
use crate::tui::app_states::AppState::*;
use crate::tui::app_states::{AVAILABLE_EVENTS, event_available_keys_to_spans};

impl<'a> App<'a> {
	fn ui(&mut self, frame: &mut Frame) {
		if let Some(bg_color) = THEME.read().ui.app_background {
			let test = Block::new().bg(bg_color);
			frame.render_widget(test, frame.area());
		}

		let main_layout = Layout::new(
			Vertical,
			[
				Constraint::Length(1),
				Constraint::Min(1),
				Constraint::Length(1),
			],
		)
		.split(frame.area());

		let header = Block::new()
			.title(" SQURL ")
			.add_modifier(Modifier::BOLD)
			.add_modifier(Modifier::ITALIC)
			.title_alignment(Alignment::Center)
			.borders(Borders::TOP);

		frame.render_widget(header, main_layout[0]);

		let inner_layout = Layout::new(
			Horizontal,
			[
				Constraint::Percentage(20),
				Constraint::Length(1),
				Constraint::Percentage(80),
			],
		)
		.split(main_layout[1]);

		if self.core.environments.is_empty() {
			let env_and_collections_layout =
				Layout::new(Vertical, [Constraint::Fill(1)]).split(inner_layout[0]);

			self.render_collections(frame, env_and_collections_layout[0]);
		} else {
			let env_and_collections_layout =
				Layout::new(Vertical, [Constraint::Length(3), Constraint::Fill(1)])
					.split(inner_layout[0]);

			self.render_environments(frame, env_and_collections_layout[0]);
			self.render_collections(frame, env_and_collections_layout[1]);
		}

		let separator = Block::new()
			.borders(Borders::RIGHT)
			.fg(THEME.read().ui.separator_color);

		frame.render_widget(separator, inner_layout[1]);

		match self.collections_tree.selected {
			None => self.render_homepage(frame, inner_layout[2]),
			Some(selection) => {
				let selected_request = self.get_request_from_selection(&selection).read().clone();

				match selected_request.protocol {
					Protocol::HttpRequest(_) => {
						self.render_http_request(frame, inner_layout[2], selected_request)
					}
					Protocol::WsRequest(_) => {
						self.render_ws_request(frame, inner_layout[2], selected_request)
					}
				}
			}
		}

		let state_line = self.get_state_line();
		let events = &*AVAILABLE_EVENTS.read();
		let available_keys = Line::from(
			event_available_keys_to_spans(
				events,
				THEME.read().ui.main_foreground_color,
				THEME.read().ui.main_background_color,
				true,
			)
			.concat(),
		);

		let footer_left = Block::new()
			.title(available_keys)
			.title_alignment(Alignment::Left);

		let footer_right = Block::new()
			.title(state_line)
			.title_alignment(Alignment::Right);

		frame.render_widget(footer_left, main_layout[2]);
		frame.render_widget(footer_right, main_layout[2]);

		// POPUPS

		match self.state {
			ChoosingElementToCreate => self.render_creating_element_popup(frame),
			DisplayingCookies | EditingCookies => self.render_cookies_popup(frame),
			CreatingNewCollection => self.render_creating_new_collection_popup(frame),
			CreatingNewRequest => self.render_creating_new_request_popup(frame),
			CreatingNewFolder => self.render_creating_new_folder_popup(frame),
			DeletingCollection => self.render_deleting_collection_popup(frame),
			DeletingRequest => self.render_deleting_request_popup(frame),
			DeletingFolder => self.render_deleting_folder_popup(frame),
			RenamingCollection => self.render_renaming_collection_popup(frame),
			RenamingRequest => self.render_renaming_request_popup(frame),
			RenamingFolder => self.render_renaming_folder_popup(frame),
			ChoosingTheme => self.render_theme_picker_popup(frame),
			_ => {}
		}

		if self.should_display_help {
			self.render_help_popup(frame);
		}
	}

	pub fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), B::Error> {
		terminal.draw(|frame| self.ui(frame))?;
		Ok(())
	}
}
