use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::layout::Direction::{Horizontal, Vertical};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::prelude::Modifier;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Borders};

use crate::app::app::App;
use crate::app::files::theme::THEME;

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
			[Constraint::Percentage(20), Constraint::Percentage(80)],
		)
		.split(main_layout[1]);

		if self.environments.is_empty() {
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
	}

	pub fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), B::Error> {
		terminal.draw(|frame| self.ui(frame))?;
		Ok(())
	}
}
