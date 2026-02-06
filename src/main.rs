use crate::app::app::App;
use crate::app::startup::startup::AppMode;

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

	Ok(())
}
