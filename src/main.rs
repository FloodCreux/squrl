use std::io::stdout;

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::ExecutableCommand;
use ratatui::crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};

use squrl::app::app::App;
use squrl::app::startup::startup::AppMode;

extern crate core;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let mut app = App::new()?;
	let app_mode = app.startup();

	let should_run_tui = match app_mode {
		AppMode::CLI(command) => {
			app.handle_command(command).await;
			false
		}
		AppMode::TUI => true,
	};

	match should_run_tui {
		true => run_tui(&mut app).await,
		false => Ok(()),
	}
}

async fn run_tui<'a>(app: &mut App<'a>) -> anyhow::Result<()> {
	let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

	app.prepare_terminal().chain_hook().run(terminal).await?;

	stdout().execute(LeaveAlternateScreen)?;
	disable_raw_mode()?;

	Ok(())
}
