use crate::app::App;
use crate::models::protocol::ws::message_type::{MessageType, next_message_type};
use crate::models::protocol::ws::ws::{Message, Sender};
use chrono::Local;
use futures_util::SinkExt;
use reqwest_websocket::{Bytes, CloseCode};
use std::sync::Arc;
use textwrap::wrap;
use tracing::info;

impl App<'_> {
	pub async fn tui_send_request_message(&mut self) {
		let Some(selected) = self.collections_tree.selected else {
			return;
		};
		let local_selected_request = self.get_request_from_selection(&selected);

		// Build the message and clone the tx Arc while holding the write guard,
		// then drop the guard before awaiting the async send.
		let send_info = {
			let mut selected_request = local_selected_request.write();
			let selected_ws_request = selected_request
				.get_ws_request_mut()
				.expect("request should be WebSocket");

			info!("Sending message");

			if selected_ws_request.is_connected {
				if let Some(websocket) = &selected_ws_request.websocket {
					let lines = self.message_text_area.to_lines();

					selected_ws_request.message_type = match selected_ws_request.message_type {
						MessageType::Text(_) => MessageType::Text(lines.join("\n")),
						MessageType::Binary(_) => MessageType::Binary(
							lines.join("").as_bytes().to_vec().into_boxed_slice(),
						),
						MessageType::Ping(_) => {
							MessageType::Ping(lines.join("").as_bytes().to_vec().into_boxed_slice())
						}
						MessageType::Pong(_) => {
							MessageType::Pong(lines.join("").as_bytes().to_vec().into_boxed_slice())
						}
						MessageType::Close(_) => MessageType::Close(lines.join("\n")),
					};

					let message = match &selected_ws_request.message_type {
						MessageType::Text(text) => reqwest_websocket::Message::Text(text.clone()),
						MessageType::Binary(binary) => {
							reqwest_websocket::Message::Binary(Bytes::from(binary.clone()))
						}
						MessageType::Ping(ping) => {
							reqwest_websocket::Message::Ping(Bytes::from(ping.clone()))
						}
						MessageType::Pong(pong) => {
							reqwest_websocket::Message::Pong(Bytes::from(pong.clone()))
						}
						MessageType::Close(close) => reqwest_websocket::Message::Close {
							code: CloseCode::Normal,
							reason: close.clone(),
						},
					};

					let tx = Arc::clone(&websocket.tx);
					let message_type = selected_ws_request.message_type.clone();
					Some((tx, message, message_type))
				} else {
					None
				}
			} else {
				info!("Websocket is not connected");
				None
			}
		};
		// Write guard is dropped here — safe to await

		if let Some((tx, message, message_type)) = send_info {
			tx.lock().await.send(message).await.ok();

			// Re-acquire write guard to record the sent message.
			{
				let mut selected_request = local_selected_request.write();
				let selected_ws_request = selected_request
					.get_ws_request_mut()
					.expect("request should be WebSocket");
				selected_ws_request.messages.push(Message {
					timestamp: Local::now(),
					content: message_type,
					sender: Sender::You,
				});
			}

			info!("Message sent");

			*self.core.received_response.lock() = true;
		}

		self.tui_load_request_message_param_tab();
		self.select_request_state();
	}

	pub fn tui_next_request_message_type(&mut self) {
		let Some(selected) = self.collections_tree.selected else {
			return;
		};
		let local_selected_request = self.get_request_from_selection(&selected);

		{
			let mut selected_request = local_selected_request.write();
			let selected_ws_request = selected_request
				.get_ws_request_mut()
				.expect("request should be WebSocket");

			let next_message_type = next_message_type(&selected_ws_request.message_type);

			info!("Message type set to \"{}\"", next_message_type);

			selected_ws_request.message_type = next_message_type;
		}

		self.save_collection_to_file(selected.collection_index());
	}

	pub fn get_messages_lines_count(&self) -> usize {
		let Some(local_selected_request) = self.get_selected_request_as_local() else {
			return 0;
		};
		let selected_request = local_selected_request.read();
		let ws_request = selected_request
			.get_ws_request()
			.expect("request should be WebSocket");

		let mut line_count = 0;
		let mut last_sender = None;

		for message in &ws_request.messages {
			let content = message.content.to_content();
			let max_length = self.get_max_line_length(&content);
			let lines = wrap(&content, max_length);

			match message.sender {
				Sender::You => line_count += lines.len() + 1,
				Sender::Server => match last_sender == Some(&message.sender) {
					true => line_count += lines.len() + 1,
					false => line_count += lines.len() + 2,
				},
			}

			last_sender = Some(&message.sender);
		}

		line_count
	}
}
