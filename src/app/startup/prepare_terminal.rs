use crate::app::app::App;
use ratatui::crossterm::ExecutableCommand;
use ratatui::crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use std::io::stdout;
use tracing::trace;

impl App {
	pub fn prepare_terminal(&mut self) -> &mut Self {
		trace!("Preparing terminal...");

		enable_raw_mode().unwrap();
		stdout().execute(EnterAlternateScreen).unwrap();

		trace!("Terminal OK");
		self
	}
}
