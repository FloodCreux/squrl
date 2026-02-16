use crate::app::App;
use crate::app::files::theme::THEME;
use crate::tui::app_states::AppState::{
	EditingRequestAuthJwtPayload, EditingRequestAuthJwtSecret, SelectedRequest,
};
use crate::tui::utils::stateful::text_input::{MultiLineTextInput, SingleLineTextInput};
use crate::tui::utils::syntax_highlighting::JSON_SYNTAX_REF;
use ratatui::Frame;
use ratatui::layout::Direction::Vertical;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Borders, Paragraph};

impl App<'_> {
	pub(super) fn render_jwt_token_tab(&mut self, frame: &mut Frame, area: Rect) {
		let jwt_token_auth_layout = Layout::new(
			Vertical,
			[
				Constraint::Length(4),
				Constraint::Length(4),
				Constraint::Length(2),
				Constraint::Min(2),
			],
		)
		.vertical_margin(1)
		.horizontal_margin(4)
		.split(area);

		let (algorithm, secret_type) = {
			let Some(local_selected_request) = self.get_selected_request_as_local() else {
				return;
			};
			let selected_request = local_selected_request.read();
			let jwt_token = selected_request.auth.get_jwt();

			(jwt_token.algorithm.clone(), jwt_token.secret_type.clone())
		};

		let mut algorithm_block = Block::new()
			.title("Algorithm ← →")
			.borders(Borders::ALL)
			.fg(THEME.read().ui.main_foreground_color);

		let mut secret_type_block = Block::new()
			.title("Secret type ← →")
			.borders(Borders::ALL)
			.fg(THEME.read().ui.main_foreground_color);

		let mut should_color_blocks = false;
		let mut should_display_cursor = false;

		// Prevent from rendering the cursor while no input text has been selected
		match self.state {
			SelectedRequest => {
				should_color_blocks = true;
			}
			EditingRequestAuthJwtSecret | EditingRequestAuthJwtPayload => {
				should_color_blocks = true;
				should_display_cursor = true;
			}
			_ => {}
		};

		let mut algorithm_paragraph =
			Paragraph::new(algorithm.to_string()).fg(THEME.read().ui.font_color);
		let mut secret_type_paragraph =
			Paragraph::new(secret_type.to_string()).fg(THEME.read().ui.font_color);

		let mut highlight_secret = false;
		let mut display_secret_cursor = false;
		let mut highlight_payload = false;
		let mut display_payload_cursor = false;

		let input_selected = self.request_editor.auth.text_input_selection.selected;

		match input_selected {
			0 if should_color_blocks => {
				algorithm_block = algorithm_block.fg(THEME.read().others.selection_highlight_color);
				algorithm_paragraph =
					algorithm_paragraph.fg(THEME.read().others.selection_highlight_color);
			}
			1 if should_color_blocks => {
				secret_type_block =
					secret_type_block.fg(THEME.read().others.selection_highlight_color);
				secret_type_paragraph =
					secret_type_paragraph.fg(THEME.read().others.selection_highlight_color);
			}
			2 if should_color_blocks => {
				highlight_secret = true;
				display_secret_cursor = should_display_cursor;
			}
			3 if should_color_blocks => {
				highlight_payload = true;
				display_payload_cursor = should_display_cursor;
			}
			_ => {}
		}

		algorithm_paragraph = algorithm_paragraph.block(algorithm_block);
		secret_type_paragraph = secret_type_paragraph.block(secret_type_block);

		self.request_editor.auth.jwt_secret.block_title =
			Some(format!("Secret ({})", algorithm.get_helper()));
		self.request_editor.auth.jwt_secret.highlight_text = highlight_secret;
		self.request_editor.auth.jwt_secret.highlight_block = highlight_secret;
		self.request_editor.auth.jwt_secret.display_cursor = display_secret_cursor;

		self.request_editor.auth.jwt_payload.highlight_text = highlight_payload;
		self.request_editor.auth.jwt_payload.highlight_block = highlight_payload;
		self.request_editor.auth.jwt_payload.display_cursor = display_payload_cursor;

		frame.render_widget(algorithm_paragraph, jwt_token_auth_layout[0]);
		frame.render_widget(secret_type_paragraph, jwt_token_auth_layout[1]);
		frame.render_widget(
			SingleLineTextInput(&mut self.request_editor.auth.jwt_secret),
			jwt_token_auth_layout[2],
		);
		frame.render_widget(
			MultiLineTextInput(
				&mut self.request_editor.auth.jwt_payload,
				JSON_SYNTAX_REF.clone(),
			),
			jwt_token_auth_layout[3],
		);
	}
}
