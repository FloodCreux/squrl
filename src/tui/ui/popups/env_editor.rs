use ratatui::Frame;
use ratatui::layout::Direction::Vertical;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Borders, Clear};

use crate::app::App;
use crate::app::files::theme::THEME;
use crate::tui::app_states::AppState;
use crate::tui::logic::utils::key_value_vec_to_items_list;
use crate::tui::utils::centered_rect::centered_rect;

impl App<'_> {
	pub fn render_env_editor_popup(&mut self, frame: &mut Frame) {
		let local_env = self.get_selected_env_as_local();

		// Determine the title based on whether we're showing collection or global env
		let title = {
			let selected = self.collections_tree.state.selected();
			if !selected.is_empty() {
				let ci = selected[0];
				if let Some(collection) = self.core.collections.get(ci) {
					if !collection.environments.is_empty() {
						let env_name = collection
							.selected_environment
							.as_deref()
							.unwrap_or("(none)");
						format!(
							" Environment: {} (collection: {}) ",
							env_name, collection.name
						)
					} else {
						match &local_env {
							Some(env) => format!(" Environment: {} ", env.read().name),
							None => " Environment Editor ".to_string(),
						}
					}
				} else {
					" Environment Editor ".to_string()
				}
			} else {
				match &local_env {
					Some(env) => format!(" Environment: {} ", env.read().name),
					None => " Environment Editor ".to_string(),
				}
			}
		};

		let popup_block = Block::default()
			.title(title)
			.borders(Borders::ALL)
			.fg(THEME.read().ui.font_color)
			.bg(THEME.read().ui.main_background_color);

		let area = centered_rect(100, 20, frame.area());

		frame.render_widget(Clear, area);
		frame.render_widget(popup_block, area);

		let inner_layout = Layout::new(Vertical, [Constraint::Fill(1)])
			.vertical_margin(1)
			.horizontal_margin(1)
			.split(area);

		self.env_editor_table.is_editing = matches!(self.state, AppState::EditingEnvVariable);

		let (keys, values) = key_value_vec_to_items_list(&local_env, &self.env_editor_table.rows);

		frame.render_stateful_widget(
			&mut self.env_editor_table,
			inner_layout[0],
			&mut (keys, values),
		);
	}
}
