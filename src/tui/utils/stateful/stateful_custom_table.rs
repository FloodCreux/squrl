use crate::app::files::theme::THEME;
use crate::models::request::KeyValue;
use crate::tui::utils::stateful::table_navigation::TableNavigation;
use crate::tui::utils::stateful::text_input::{SingleLineTextInput, TextInput};
use ratatui::buffer::Buffer;
use ratatui::layout::Direction::{Horizontal, Vertical};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Line, Modifier, StatefulWidget, Style, Stylize, Widget};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

pub struct StatefulCustomTable<'a> {
	pub left_state: ListState,
	pub right_state: ListState,
	/// (x, y)
	pub selection: Option<(usize, usize)>,
	pub rows: Vec<KeyValue>,
	pub selection_text_input: TextInput,

	pub is_editing: bool,
	pub empty_rows_lines: Vec<Line<'a>>,
	pub default_key: &'static str,
	pub default_value: &'static str,
}

impl<'a> StatefulCustomTable<'a> {
	pub fn new(
		empty_rows_lines: Vec<Line<'a>>,
		default_key: &'static str,
		default_value: &'static str,
	) -> Self {
		Self {
			left_state: ListState::default(),
			right_state: ListState::default(),
			selection: None,
			rows: vec![],
			selection_text_input: TextInput::new(None),
			is_editing: false,
			empty_rows_lines,
			default_key,
			default_value,
		}
	}

	pub fn update_selection(&mut self, selection: Option<(usize, usize)>) {
		match selection {
			None => {
				self.selection = None;
				self.left_state.select(None);
				self.right_state.select(None);
			}
			Some(selection) => {
				self.selection = Some(selection);
				self.left_state.select(Some(selection.0));
				self.right_state.select(Some(selection.1));
			}
		}
	}

	pub fn change_y(&mut self) {
		self.left();
		// Sync list states after column toggle
		if let Some((x, _)) = self.selection {
			self.right_state.select(Some(x));
			self.left_state.select(Some(x));
		}
	}

	pub fn is_selected(&self) -> bool {
		self.selection.is_some()
	}
}

impl TableNavigation for StatefulCustomTable<'_> {
	fn rows_len(&self) -> usize { self.rows.len() }
	fn columns_count(&self) -> usize { 2 }
	fn selection(&self) -> Option<(usize, usize)> { self.selection }
	fn set_selection(&mut self, selection: Option<(usize, usize)>) { self.selection = selection; }

	fn select_row_in_all_states(&mut self, row: usize) {
		self.left_state.select(Some(row));
		self.right_state.select(Some(row));
	}

	fn selected_row_in_column(&self, col: usize) -> Option<usize> {
		match col {
			0 => self.left_state.selected(),
			1 => self.right_state.selected(),
			_ => None,
		}
	}
}

impl<'a> StatefulWidget for &'a mut StatefulCustomTable<'_> {
	type State = (Vec<ListItem<'a>>, Vec<ListItem<'a>>);

	fn render(self, area: Rect, buf: &mut Buffer, rows: &mut Self::State)
	where
		Self: Sized,
	{
		match self.selection {
			None => {
				let headers_paragraph = Paragraph::new(self.empty_rows_lines.clone()).centered();

				headers_paragraph.render(area, buf);
			}
			Some(selection) => {
				let layout = Layout::new(Vertical, [Constraint::Fill(1)]).split(area);

				let horizontal_margin = 2;

				let table_layout = Layout::new(
					Horizontal,
					[Constraint::Percentage(25), Constraint::Percentage(75)],
				)
				.horizontal_margin(horizontal_margin)
				.split(layout[0]);

				let mut left_list_style = Style::default();
				let mut right_list_style = Style::default();

				match selection.1 {
					0 => left_list_style = left_list_style.add_modifier(Modifier::BOLD),
					1 => {
						right_list_style = right_list_style
							.add_modifier(Modifier::BOLD)
							.fg(THEME.read().others.selection_highlight_color)
					}
					_ => {}
				}

				let left_list = List::new(rows.0.to_owned())
					.highlight_style(left_list_style)
					.fg(THEME.read().ui.font_color);

				let right_list = List::new(rows.1.to_owned())
					.highlight_style(right_list_style)
					.fg(THEME.read().ui.font_color);

				StatefulWidget::render(
					left_list,
					table_layout[0],
					buf,
					&mut self.left_state.clone(),
				);
				StatefulWidget::render(
					right_list,
					table_layout[1],
					buf,
					&mut self.right_state.clone(),
				);

				// Form input & cursor

				let cell_with = layout[0].width / 2;

				let width_adjustment = match selection.1 {
					0 => 0,
					1 => {
						let even_odd_adjustment = match layout[0].width % 2 {
							1 => 1,
							0 => 2,
							_ => 0,
						};

						cell_with.saturating_sub(even_odd_adjustment)
					}
					_ => 0,
				};

				let height_adjustment =
					(selection.0 - self.left_state.offset()) as u16 % layout[0].height;

				let selection_position_x = layout[0].x + width_adjustment + horizontal_margin;
				let selection_position_y = layout[0].y + height_adjustment;

				let separator_offset = match selection.1 {
					1 => 2u16,
					_ => 0,
				};

				let text_rect = Rect::new(
					selection_position_x + separator_offset,
					selection_position_y,
					cell_with.saturating_sub(horizontal_margin + separator_offset),
					1,
				);

				if self.is_editing {
					self.selection_text_input.display_cursor = true;
					self.selection_text_input.highlight_text = true;
					" ".repeat(text_rect.width as usize).render(text_rect, buf);
					SingleLineTextInput(&mut self.selection_text_input).render(text_rect, buf);
				} else {
					self.selection_text_input.display_cursor = false;
					self.selection_text_input.highlight_text = false;
				}
			}
		}
	}
}
