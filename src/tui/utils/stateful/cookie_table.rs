use cookie_store::Cookie;
use ratatui::widgets::ListState;

use crate::tui::utils::stateful::table_navigation::TableNavigation;
use crate::tui::utils::stateful::text_input::TextInput;

pub const COOKIES_COLUMNS_NUMBER: usize = 8;

pub struct StatefulCookieTable {
	pub lists_states: [ListState; COOKIES_COLUMNS_NUMBER],
	/// (x, y)
	pub selection: Option<(usize, usize)>,
	pub rows: Vec<[String; COOKIES_COLUMNS_NUMBER]>,
	pub selection_text_input: TextInput,
}

impl Default for StatefulCookieTable {
	fn default() -> Self {
		Self {
			lists_states: [
				ListState::default(),
				ListState::default(),
				ListState::default(),
				ListState::default(),
				ListState::default(),
				ListState::default(),
				ListState::default(),
				ListState::default(),
			],
			selection: None,
			rows: vec![],
			selection_text_input: TextInput::new(None),
		}
	}
}

impl TableNavigation for StatefulCookieTable {
	fn rows_len(&self) -> usize { self.rows.len() }
	fn columns_count(&self) -> usize { COOKIES_COLUMNS_NUMBER }
	fn selection(&self) -> Option<(usize, usize)> { self.selection }
	fn set_selection(&mut self, selection: Option<(usize, usize)>) { self.selection = selection; }

	fn select_row_in_all_states(&mut self, row: usize) {
		for list_state in self.lists_states.iter_mut() {
			list_state.select(Some(row));
		}
	}

	fn selected_row_in_column(&self, col: usize) -> Option<usize> {
		self.lists_states[col].selected()
	}
}

pub fn cookie_to_row(cookie: &Cookie) -> [String; COOKIES_COLUMNS_NUMBER] {
	[
		match cookie.domain() {
			None => String::new(),
			Some(domain) => domain.to_string(),
		},
		cookie.name().to_string(),
		cookie.value().to_string(),
		match cookie.path() {
			None => String::new(),
			Some(path) => path.to_string(),
		},
		match cookie.expires() {
			None => String::new(),
			Some(expiration) => match expiration.is_datetime() {
				true => expiration.datetime().unwrap().to_string(),
				false => String::from("session"),
			},
		},
		match cookie.http_only() {
			None => String::new(),
			Some(http_only) => http_only.to_string(),
		},
		match cookie.secure() {
			None => String::new(),
			Some(secure) => secure.to_string(),
		},
		match cookie.same_site() {
			None => String::new(),
			Some(same_site) => same_site.to_string(),
		},
	]
}
