pub fn wrapping_decrement(i: usize, len: usize) -> usize {
	if i == 0 { len - 1 } else { i - 1 }
}

pub fn wrapping_increment(i: usize, len: usize) -> usize {
	if i >= len - 1 { 0 } else { i + 1 }
}

pub trait TableNavigation {
	fn rows_len(&self) -> usize;
	fn columns_count(&self) -> usize;
	fn selection(&self) -> Option<(usize, usize)>;
	fn set_selection(&mut self, selection: Option<(usize, usize)>);
	fn select_row_in_all_states(&mut self, row: usize);
	fn selected_row_in_column(&self, col: usize) -> Option<usize>;

	fn up(&mut self) {
		if self.rows_len() == 0 || self.selection().is_none() {
			return;
		}

		let sel = self
			.selection()
			.expect("selection should exist after guard");

		let x = match self.selected_row_in_column(sel.1) {
			None => 0,
			Some(i) => wrapping_decrement(i, self.rows_len()),
		};

		self.select_row_in_all_states(x);
		self.set_selection(Some((x, sel.1)));
	}

	fn down(&mut self) {
		if self.rows_len() == 0 || self.selection().is_none() {
			return;
		}

		let sel = self
			.selection()
			.expect("selection should exist after guard");

		let x = match self.selected_row_in_column(sel.1) {
			None => 0,
			Some(i) => wrapping_increment(i, self.rows_len()),
		};

		self.select_row_in_all_states(x);
		self.set_selection(Some((x, sel.1)));
	}

	fn left(&mut self) {
		if self.rows_len() == 0 || self.selection().is_none() {
			return;
		}

		let sel = self
			.selection()
			.expect("selection should exist after guard");
		let y = wrapping_decrement(sel.1, self.columns_count());
		self.set_selection(Some((sel.0, y)));
	}

	// fn right(&mut self) {
	// 	if self.rows_len() == 0 || self.selection().is_none() {
	// 		return;
	// 	}
	//
	// 	let sel = self.selection().unwrap();
	// 	let y = wrapping_increment(sel.1, self.columns_count());
	// 	self.set_selection(Some((sel.0, y)));
	// }
}
