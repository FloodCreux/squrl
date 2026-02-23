use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::app::files::theme::THEME;

impl<'a> App<'a> {
	pub(super) fn render_environments(&mut self, frame: &mut Frame, rect: Rect) {
		// Check if the selected collection has its own environments
		let (env_name, is_collection_env) = {
			let selected = self.collections_tree.state.selected();
			if !selected.is_empty() {
				let ci = selected[0];
				if let Some(collection) = self.core.collections.get(ci) {
					if !collection.environments.is_empty() {
						let name = collection
							.selected_environment
							.clone()
							.unwrap_or_else(|| "(none)".to_string());
						(Some(name), true)
					} else {
						(None, false)
					}
				} else {
					(None, false)
				}
			} else {
				(None, false)
			}
		};

		let current_environment = match env_name {
			Some(name) => name,
			None => match self.get_selected_env_as_local() {
				Some(local_env) => {
					let env = local_env.read();
					env.name.clone()
				}
				None => "(no environment)".to_string(),
			},
		};

		let title = if is_collection_env {
			"Collection Env"
		} else {
			"Environment"
		};

		let current_environment_paragraph = Paragraph::new(current_environment).block(
			Block::default()
				.title(title)
				.title_alignment(Alignment::Center)
				.borders(Borders::ALL)
				.border_set(border::Set {
					vertical_left: " ",
					vertical_right: " ",
					..border::PLAIN
				})
				.style(Style::new().fg(THEME.read().ui.secondary_foreground_color)),
		);

		frame.render_widget(current_environment_paragraph, rect)
	}
}
