use crate::app::App;
use crate::app::files::theme::THEME;
use ratatui::Frame;
use ratatui::layout::Direction::{Horizontal, Vertical};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph};
use tui_big_text::{BigTextBuilder, PixelSize};

impl App<'_> {
	pub(super) fn render_homepage(&mut self, frame: &mut Frame, rect: Rect) {
		let block = Block::new();

		let inner_block_area = block.inner(rect);

		let description_string = r#"
       xxxxxxxxxx
   xxxxxxxxxxxxxxxxxx
 +xxxxxxxxxxxxxxxxxxxxx               x
xxxxxxxxxxxxxxxxxxxxxxxxx             xxx
xxxxxxxxxxxXxxxxxxxxxxxxxx            xxx
xxxxxxxxxxxxxxxxxxxxxxxxxx            +xxx
xxxxxxxxxxxxxXxxxxxxxxxxxxx       ;;;;;;;;;;
xxxxxxxxxxxxxxxxxxxxxxxxxxx      ;;;;;.   ;;;;
 xxxxxxxxxxxXxxxxxxxxxxxxxx.    ;;;;;      ;;;;
     xxxxxxXxxxxxxxxxxxxxxxx   ;;;;;;   &&  ;;;;
          Xxxxxxxxxxx;;;;;;;;;;;;;;;;;;      ;;;
         Xxxxxxxxxx;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
        Xxxxxxxxx;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;x
       XXxxxxxxx;;;;;;;;;;;;;+++++++++  :;;.
       Xxxxxxxxx;;;;;;;;;;+++++++++++++++.
       Xxxxxxxxx;;;;;;;;;+++++++++++++++++++++
       Xxxxxxxxx;;;;;;;;+++++++++++ +++++
       xxxxxxxxx;;;;;;;++++++++++++
        Xxxxxxxxx;;;;;+++++++++++++
         Xxxxxxxxx;;;;++++++++++++
           xxxxxxxx;;++++++++++++++
               xxxxxx+++++++++++++++
			"#;

		let inner_layout = Layout::new(
			Vertical,
			[
				Constraint::Percentage(50),
				Constraint::Length(1),
				Constraint::Length(1),
				Constraint::Length(4),
				Constraint::Length(description_string.lines().count() as u16),
				Constraint::Percentage(50),
			],
		)
		.split(inner_block_area);

		let title_length = 20;
		let description_width = description_string
			.lines()
			.map(|line| line.len())
			.max()
			.unwrap_or(0) as u16;

		let title_layout = Layout::new(
			Horizontal,
			[
				Constraint::Fill(1),
				Constraint::Length(title_length),
				Constraint::Fill(1),
			],
		)
		.split(inner_layout[3]);

		let title = BigTextBuilder::default()
			.pixel_size(PixelSize::Quadrant)
			.lines(["SQURL".into()])
			.style(Style::new().fg(THEME.read().ui.font_color))
			.centered()
			.build();

		let welcome_to = Paragraph::new("Welcome to")
			.centered()
			.fg(THEME.read().ui.secondary_foreground_color);

		let description_layout = Layout::new(
			Horizontal,
			[
				Constraint::Fill(1),
				Constraint::Length(description_width),
				Constraint::Fill(1),
			],
		)
		.split(inner_layout[4]);

		let description =
			Paragraph::new(description_string).fg(THEME.read().ui.main_foreground_color);

		frame.render_widget(block, rect);
		frame.render_widget(welcome_to, inner_layout[1]);
		frame.render_widget(title, title_layout[1]);
		frame.render_widget(description, description_layout[1]);
	}
}
