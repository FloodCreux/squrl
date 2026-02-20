use crate::models::protocol::graphql::graphql::GraphqlRequest;
use crate::models::protocol::http::http::HttpRequest;
use crate::models::protocol::protocol::Protocol;
use crate::models::protocol::ws::ws::WsRequest;
use crate::tui::utils::stateful::text_input::TextInput;

pub struct NewRequestPopup {
	pub selection: usize,

	pub selected_collection: usize,
	pub max_collection_selection: usize,

	pub selected_folder: Option<usize>,
	pub folder_count: usize,

	pub protocol: Protocol,

	pub text_input: TextInput,
}

impl Default for NewRequestPopup {
	fn default() -> Self {
		Self {
			selection: 0,
			selected_collection: 0,
			max_collection_selection: 0,
			selected_folder: None,
			folder_count: 0,
			protocol: Protocol::default(),
			text_input: TextInput::new(Some(String::from("Request name"))),
		}
	}
}

impl NewRequestPopup {
	pub fn next_input(&mut self) {
		self.selection = match self.selection {
			0 => 1,
			1 => 2,
			2 => 3,
			3 => 0,
			_ => unreachable!(),
		}
	}

	pub fn previous_input(&mut self) {
		self.selection = match self.selection {
			0 => 3,
			1 => 0,
			2 => 1,
			3 => 2,
			_ => unreachable!(),
		}
	}

	pub fn input_left(&mut self) {
		match self.selection {
			0 => self.previous_collection(),
			1 => self.previous_folder(),
			2 => self.previous_protocol(),
			3 => self.text_input.move_cursor_left(),
			_ => unreachable!(),
		}
	}

	pub fn input_right(&mut self) {
		match self.selection {
			0 => self.next_collection(),
			1 => self.next_folder(),
			2 => self.next_protocol(),
			3 => self.text_input.move_cursor_right(),
			_ => unreachable!(),
		}
	}

	pub fn next_collection(&mut self) {
		if self.selected_collection + 1 < self.max_collection_selection {
			self.selected_collection += 1;
		} else {
			self.selected_collection = 0;
		}
		// Reset folder selection when collection changes — folder_count will be
		// updated by the App before the next render
		self.selected_folder = None;
		self.folder_count = 0;
	}

	pub fn previous_collection(&mut self) {
		if self.selected_collection as isize > 0 {
			self.selected_collection -= 1;
		} else {
			self.selected_collection = self.max_collection_selection - 1;
		}
		// Reset folder selection when collection changes
		self.selected_folder = None;
		self.folder_count = 0;
	}

	pub fn next_folder(&mut self) {
		if self.folder_count == 0 {
			return;
		}

		self.selected_folder = match self.selected_folder {
			None => Some(0),
			Some(i) if i + 1 < self.folder_count => Some(i + 1),
			Some(_) => None,
		};
	}

	pub fn previous_folder(&mut self) {
		if self.folder_count == 0 {
			return;
		}

		self.selected_folder = match self.selected_folder {
			None => Some(self.folder_count - 1),
			Some(0) => None,
			Some(i) => Some(i - 1),
		};
	}

	pub fn next_protocol(&mut self) {
		self.protocol = match self.protocol {
			Protocol::HttpRequest(_) => Protocol::WsRequest(WsRequest::default()),
			Protocol::WsRequest(_) => Protocol::GraphqlRequest(GraphqlRequest::default()),
			Protocol::GraphqlRequest(_) => Protocol::HttpRequest(HttpRequest::default()),
		}
	}

	pub fn previous_protocol(&mut self) {
		self.protocol = match self.protocol {
			Protocol::HttpRequest(_) => Protocol::GraphqlRequest(GraphqlRequest::default()),
			Protocol::GraphqlRequest(_) => Protocol::WsRequest(WsRequest::default()),
			Protocol::WsRequest(_) => Protocol::HttpRequest(HttpRequest::default()),
		}
	}
}
