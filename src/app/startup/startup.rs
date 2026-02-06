use crate::app::app::App;
use crate::app::startup::startup::AppMode::{CLI, TUI};
use crate::cli::args::{ARGS, Command};

pub enum AppMode {
	TUI,
	CLI(Command),
}

impl<'a> App<'a> {
	pub fn startup(&mut self) -> AppMode {
		match ARGS.command {
			Some(_) => tracing_subscriber::fmt()
				.pretty()
				.with_file(false)
				.with_line_number(false)
				.init(),
			_ => tracing_subscriber::fmt()
				.pretty()
				.with_file(false)
				.with_line_number(false)
				.init(),
		};

		if let Some(command) = &ARGS.command {
			CLI(command.clone())
		} else {
			TUI
		}
	}
}
