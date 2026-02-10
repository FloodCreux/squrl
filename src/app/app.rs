use std::io::Stdout;
use std::sync::Arc;

#[cfg(feature = "clipboard")]
use arboard::Clipboard;
use parking_lot::RwLock;
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};

use crate::app::files::config::Config;
use crate::models::collection::Collection;
use crate::models::environment::Environment;
use crate::tui::utils::stateful::cookies_popup::CookiesPopup;
use crate::tui::utils::stateful::display_popup::DisplayPopup;
use crate::tui::utils::stateful::stateful_tree::StatefulTree;

pub struct App<'a> {
	pub should_quit: bool,
	pub environments: Vec<Arc<RwLock<Environment>>>,
	pub selected_environment: usize,
	pub config: Config,

	pub collections: Vec<Collection>,
	pub collections_tree: StatefulTree<'a>,

	pub cookies_popup: CookiesPopup,
	pub display_request_export: DisplayPopup,

	#[cfg(feature = "clipboard")]
	pub clipboard: Option<Clipboard>,
}

impl App<'_> {
	pub fn new<'a>() -> anyhow::Result<App<'a>> {
		Ok(App {
			config: Config::default(),
			collections: vec![],
			collections_tree: StatefulTree::default(),
			should_quit: false,
			environments: vec![],
			selected_environment: 0,
			cookies_popup: CookiesPopup::default(),
			display_request_export: DisplayPopup::default(),
			clipboard: Clipboard::new().ok(),
		})
	}

	pub async fn run(
		&mut self,
		mut terminal: Terminal<CrosstermBackend<Stdout>>,
	) -> Result<(), <CrosstermBackend<Stdout> as Backend>::Error> {
		let _ = terminal.clear();

		while !self.should_quit {}

		Ok(())
	}
}
