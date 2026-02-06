use std::io::Stdout;
use std::sync::Arc;

use parking_lot::RwLock;
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};

use crate::models::environment::Environment;

pub struct App<'a> {
	pub should_quit: bool,
	pub environments: Vec<Arc<RwLock<Environment>>>,
	pub selected_environment: usize,
}

impl App<'_> {
	pub fn new<'a>() -> anyhow::Result<App<'a>> {
		Ok(App {
			should_quit: false,
			environments: vec![],
			selected_environment: 0,
		})
	}

	pub async fn run(
		&mut self,
		mut terminal: Terminal<CrosstermBackend<Stdout>>,
	) -> Result<(), <CrosstermBackend<Stdout> as Backend>::Error> {
		terminal.clear();

		while !self.should_quit {}

		Ok(())
	}
}
