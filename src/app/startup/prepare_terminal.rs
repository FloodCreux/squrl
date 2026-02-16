use crate::app::App;
use crate::errors::panic_error;
use ratatui::crossterm::ExecutableCommand;
use ratatui::crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use std::io::stdout;
use tracing::trace;

impl App<'_> {
	pub fn prepare_terminal(&mut self) -> &mut Self {
		trace!("Preparing terminal...");

		if let Err(e) = enable_raw_mode() {
			panic_error(format!("Failed to enable raw mode for terminal: {e}"));
		}
		if let Err(e) = stdout().execute(EnterAlternateScreen) {
			panic_error(format!("Failed to enter alternate screen: {e}"));
		}

		trace!("Terminal OK");
		self
	}
}
