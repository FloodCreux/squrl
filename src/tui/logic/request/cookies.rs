use crate::app::app::App;
use crate::tui::utils::stateful::cookie_table::COOKIES_COLUMNS_NUMBER;
use reqwest::Url;

impl App<'_> {
	pub fn tui_update_cookies_table_selection(&mut self) {
		match self.core.cookies_popup.cookies_table.rows.is_empty() {
			false => {
				self.core.cookies_popup.cookies_table.selection = Some((0, 0));

				for table_state in self
					.core
					.cookies_popup
					.cookies_table
					.lists_states
					.iter_mut()
				{
					table_state.select(Some(0));
				}
			}
			true => {
				self.core.cookies_popup.cookies_table.selection = None;

				for table_state in self
					.core
					.cookies_popup
					.cookies_table
					.lists_states
					.iter_mut()
				{
					table_state.select(None);
				}
			}
		}
	}

	pub fn tui_delete_cookie(&mut self) {
		if self.core.cookies_popup.cookies_table.rows.is_empty()
			|| self.core.cookies_popup.cookies_table.selection.is_none()
		{
			return;
		}

		let selection = self.core.cookies_popup.cookies_table.selection.unwrap();
		let cookie_row = self
			.core
			.cookies_popup
			.cookies_table
			.rows
			.remove(selection.0);

		{
			let mut local_cookie_store = self.core.cookies_popup.cookie_store.write().unwrap();

			local_cookie_store.remove(&cookie_row[0], &cookie_row[3], &cookie_row[1]);
		}

		self.tui_update_cookies_table_selection();
	}

	pub fn tui_modify_cookie(&mut self) {
		if self.core.cookies_popup.cookies_table.rows.is_empty()
			|| self.core.cookies_popup.cookies_table.selection.is_none()
		{
			return;
		}

		let selection = self.core.cookies_popup.cookies_table.selection.unwrap();
		let new_text = self
			.core
			.cookies_popup
			.cookies_table
			.selection_text_input
			.to_string();

		let row = &self.core.cookies_popup.cookies_table.rows[selection.0];
		let old_domain = row[0].clone();
		let old_name = row[1].clone();
		let old_path = row[3].clone();

		// Remove old cookie from the store
		{
			let mut local_cookie_store = self.core.cookies_popup.cookie_store.write().unwrap();
			local_cookie_store.remove(&old_domain, &old_path, &old_name);
		}

		// Update the in-memory row with the new text
		self.core.cookies_popup.cookies_table.rows[selection.0][selection.1] = new_text;

		// Re-insert the cookie into the store
		self.insert_cookie_row_into_store(selection.0);

		// Refresh the cookies display (reloads from store and sets state)
		self.display_cookies_state();
	}

	pub fn tui_create_cookie(&mut self) {
		let new_row: [String; COOKIES_COLUMNS_NUMBER] = [
			String::new(),     // URL/Domain
			String::new(),     // Name
			String::new(),     // Value
			String::from("/"), // Path
			String::new(),     // Expires
			String::new(),     // HttpOnly
			String::new(),     // Secure
			String::new(),     // SameSite
		];

		self.core.cookies_popup.cookies_table.rows.push(new_row);

		let new_row_index = self.core.cookies_popup.cookies_table.rows.len() - 1;

		// Select the new row, Name column (index 1)
		self.core.cookies_popup.cookies_table.selection = Some((new_row_index, 1));

		for list_state in self
			.core
			.cookies_popup
			.cookies_table
			.lists_states
			.iter_mut()
		{
			list_state.select(Some(new_row_index));
		}
	}

	/// Build a Set-Cookie string from a cookie row and insert it into the cookie store.
	fn insert_cookie_row_into_store(&mut self, row_index: usize) {
		let row = &self.core.cookies_popup.cookies_table.rows[row_index];

		let domain = &row[0];
		let name = &row[1];
		let value = &row[2];
		let path = &row[3];

		// Need a valid domain to insert into the cookie store
		if domain.is_empty() || name.is_empty() {
			return;
		}

		// Build a Set-Cookie header string
		let mut cookie_str = format!("{}={}; Path={}", name, value, path);

		// Domain
		cookie_str.push_str(&format!("; Domain={}", domain));

		// HttpOnly
		if row[5] == "true" {
			cookie_str.push_str("; HttpOnly");
		}

		// Secure
		if row[6] == "true" {
			cookie_str.push_str("; Secure");
		}

		// SameSite
		if !row[7].is_empty() {
			cookie_str.push_str(&format!("; SameSite={}", row[7]));
		}

		// Expires
		if !row[4].is_empty() && row[4] != "session" {
			cookie_str.push_str(&format!("; Expires={}", row[4]));
		}

		// We need a URL to insert the cookie; construct one from the domain
		let scheme = if row[6] == "true" { "https" } else { "http" };
		let url_string = format!("{}://{}{}", scheme, domain, path);

		if let Ok(url) = Url::parse(&url_string) {
			let mut local_cookie_store = self.core.cookies_popup.cookie_store.write().unwrap();
			let _ = local_cookie_store.parse(&cookie_str, &url);
		}
	}
}
