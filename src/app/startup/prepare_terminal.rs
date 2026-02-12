use crate::app::app::App;
use ratatui::crossterm::ExecutableCommand;
use ratatui::crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use std::io::stdout;
use tracing::trace;

impl App<'_> {
	pub fn prepare_terminal(&mut self) -> &mut Self {
		trace!("Preparing terminal...");

		enable_raw_mode().expect("failed to enable raw mode for terminal");
		stdout()
			.execute(EnterAlternateScreen)
			.expect("failed to enter alternate screen");

		trace!("Terminal OK");
		self
	}
}
