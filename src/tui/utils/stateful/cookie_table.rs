use crate::tui::utils::stateful::text_input::TextInput;
use cookie_store::Cookie;
use ratatui::layout::Constraint;
use ratatui::widgets::ListState;
use strum::{Display, FromRepr};

#[allow(clippy::upper_case_acronyms)]
#[derive(Display, FromRepr)]
pub enum CookieColumns {
	#[strum(to_string = "URL")]
	URL,
	#[strum(to_string = "Name")]
	Name,
	#[strum(to_string = "Value")]
	Value,
	#[strum(to_string = "Path")]
	Path,
	#[strum(to_string = "Expires")]
	Expires,
	#[strum(to_string = "Http\nonly")]
	HttpOnly,
	#[strum(to_string = "Secure")]
	Secure,
	#[strum(to_string = "Same\nsite")]
	SameSite,
}

impl CookieColumns {
	pub fn constraints() -> [Constraint; 8] {
		[
			Constraint::Percentage(10),
			Constraint::Percentage(15),
			Constraint::Percentage(37),
			Constraint::Percentage(10),
			Constraint::Percentage(10),
			Constraint::Percentage(6),
			Constraint::Percentage(6),
			Constraint::Percentage(6),
		]
	}
}

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

impl StatefulCookieTable {
	/// Returns true if the given column index is editable (Name = 1, Value = 2)
	pub fn is_editable_column(col: usize) -> bool {
		col == 1 || col == 2
	}

	fn decrement_x(&self, i: usize) -> usize {
		if i == 0 { self.rows.len() - 1 } else { i - 1 }
	}

	fn increment_x(&self, i: usize) -> usize {
		if i >= self.rows.len() - 1 { 0 } else { i + 1 }
	}

	fn decrement_y(&self, i: usize) -> usize {
		if i == 0 {
			COOKIES_COLUMNS_NUMBER - 1
		} else {
			i - 1
		}
	}

	pub fn increment_y(&mut self, i: usize) -> usize {
		if i >= COOKIES_COLUMNS_NUMBER - 1 {
			0
		} else {
			i + 1
		}
	}

	pub fn up(&mut self) {
		let Some((_, y)) = self.selection else { return };
		if self.rows.is_empty() {
			return;
		}

		let x = match self.lists_states[y].selected() {
			None => 0,
			Some(i) => self.decrement_x(i),
		};

		for list_state in self.lists_states.iter_mut() {
			list_state.select(Some(x));
		}

		self.selection = Some((x, y))
	}

	pub fn down(&mut self) {
		let Some((_, y)) = self.selection else { return };
		if self.rows.is_empty() {
			return;
		}

		let x = match self.lists_states[y].selected() {
			None => 0,
			Some(i) => self.increment_x(i),
		};

		for list_state in self.lists_states.iter_mut() {
			list_state.select(Some(x));
		}

		self.selection = Some((x, y))
	}

	pub fn left(&mut self) {
		let Some((x, col)) = self.selection else {
			return;
		};
		if self.rows.is_empty() {
			return;
		}

		let y = self.decrement_y(col);

		self.selection = Some((x, y))
	}

	pub fn right(&mut self) {
		let Some((x, col)) = self.selection else {
			return;
		};
		if self.rows.is_empty() {
			return;
		}

		let y = self.increment_y(col);

		self.selection = Some((x, y))
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
				true => expiration
					.datetime()
					.expect("datetime should exist when is_datetime is true")
					.to_string(),
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
