use crate::app::app::App;
use crate::app::request::http::send::send_http_request;
use crate::app::request::ws::send::send_ws_request;
use crate::cli::commands::request_commands::send::SendCommand;
use crate::models::protocol::protocol::Protocol;
use crate::models::protocol::ws::message_type::MessageType;
use crate::models::protocol::ws::ws::{Message, Sender};
use crate::models::request::Request;
use crate::models::response::ResponseContent;
use anyhow::anyhow;
use chrono::Local;
use futures_util::SinkExt;
use parking_lot::RwLock;
use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};
use ratatui_image::picker::Picker;
use ratatui_image::{Resize, ResizeEncodeRender};
use std::io::stdout;
use std::sync::Arc;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::info;

impl App<'_> {
	pub async fn cli_send_request(
		&mut self,
		collection_index: usize,
		request_index: usize,
		send_command: &SendCommand,
	) -> anyhow::Result<()> {
		let local_request =
			self.get_request_as_local_from_indexes(&(collection_index, request_index));

		self.local_send_request(send_command, local_request).await?;

		if self.core.config.should_save_requests_response() {
			self.save_collection_to_file(collection_index);
		}

		Ok(())
	}

	pub async fn cli_send_collection(
		&mut self,
		collection_name: &str,
		send_command: &SendCommand,
	) -> anyhow::Result<()> {
		let collection_index = self.find_collection(collection_name)?;
		let collection = &self.core.collections[collection_index];

		let mut requests: Vec<Arc<RwLock<Request>>> = vec![];

		for request in &collection.requests {
			let local_request = request.clone();
			requests.push(local_request);
		}

		for request in requests {
			self.local_send_request(send_command, request).await?;

			if self.core.config.should_save_requests_response() {
				self.save_collection_to_file(collection_index);
			}
		}

		Ok(())
	}

	pub async fn local_send_request(
		&mut self,
		send_command: &SendCommand,
		local_request: Arc<RwLock<Request>>,
	) -> anyhow::Result<()> {
		// Synchronous phase: prepare the request while holding the write guard.
		let (prepared, protocol) = {
			let mut request = local_request.write();

			if let Some(env_name) = &send_command.env {
				let env_index = self.find_environment(env_name)?;
				self.core.selected_environment = env_index;
			};

			if send_command.request_name {
				println!("{}", request.name);
			}

			let prepared = match self.prepare_request(&mut request) {
				Ok(prepared) => prepared,
				Err(error) => {
					if send_command.console
						&& let Some(pre_request_output) = &request.console_output.pre_request_output
					{
						println!("{}", pre_request_output);
					}

					return Err(anyhow!(error));
				}
			};

			let protocol = request.protocol.clone();
			(prepared, protocol)
		};
		// Guard is dropped here — safe to await for file body finalization

		let prepared_request = App::finalize_prepared_request(prepared).await?;

		let local_env = self.get_selected_env_as_local();
		let response = match protocol {
			Protocol::HttpRequest(_) => {
				send_http_request(prepared_request, local_request.clone(), &local_env).await?
			}
			Protocol::WsRequest(_) => {
				send_ws_request(
					prepared_request,
					local_request.clone(),
					&local_env,
					self.core.received_response.clone(),
				)
				.await?
			}
		};

		let request = local_request.read();

		if send_command.status_code {
			println!(
				"{}",
				response
					.status_code
					.as_ref()
					.expect("response should have a status code")
			);
		}

		if send_command.duration {
			println!(
				"{}",
				response.duration.expect("response should have a duration")
			);
		}

		if send_command.cookies {
			println!(
				"{}",
				response.cookies.expect("response should have cookies")
			);
		}

		if send_command.headers {
			println!("{:?}", response.headers);
		}

		if send_command.console {
			let console_output = match (
				&request.console_output.pre_request_output,
				&request.console_output.post_request_output,
			) {
				(None, None) => &String::new(),
				(Some(pre_request_console_output), None) => pre_request_console_output,
				(None, Some(post_request_console_output)) => post_request_console_output,
				(Some(pre_request_console_output), Some(post_request_console_output)) => {
					&format!("{pre_request_console_output}\n{post_request_console_output}")
				}
			};

			println!("{}", console_output);
		}

		if !send_command.hide_content {
			match response.content {
				None => {}
				Some(content) => match content {
					ResponseContent::Body(body) => println!("{}", body),
					ResponseContent::Image(image) => match image.image {
						None => {
							println!("{:?}", image.data)
						}
						Some(dynamic_image) => {
							let image_width = dynamic_image.width() as f32;
							let image_height = dynamic_image.height() as f32;

							let backend = CrosstermBackend::new(stdout());
							let terminal_size = backend.size()?;

							let width_ratio = terminal_size.width as f32 / image_width;
							let height_ratio = terminal_size.height as f32 / image_height;

							let ratio = width_ratio.min(height_ratio);

							let mut terminal = Terminal::with_options(
								backend,
								TerminalOptions {
									viewport: Viewport::Inline((image_height * ratio) as u16),
								},
							)?;

							let picker = match self.core.config.is_graphical_protocol_disabled() {
								true => Picker::halfblocks(),
								false => Picker::from_query_stdio().unwrap_or(Picker::halfblocks()),
							};

							let mut stateful_protocol = picker.new_resize_protocol(dynamic_image);

							terminal.draw(|frame| {
								stateful_protocol.resize_encode_render(
									&Resize::Fit(None),
									Rect {
										x: 0,
										y: 0,
										width: (image_width * ratio) as u16,
										height: (image_height * ratio) as u16,
									},
									frame.buffer_mut(),
								)
							})?;
						}
					},
				},
			};
		}
		drop(request);

		if let Protocol::WsRequest(_) = &protocol {
			let mut last_length = 0;
			let local_local_request = local_request.clone();

			tokio::spawn(async move {
				let stdin = io::stdin();
				let reader = BufReader::new(stdin);
				let mut lines = reader.lines();
				let mut buffer = String::new();

				loop {
					if let Ok(Some(line)) = lines.next_line().await {
						if line.ends_with("\u{1b}") {
							let line = &line[..line.len() - 1];
							buffer.push_str(line);
							buffer.push('\n');
						} else {
							buffer.push_str(&line);
							let text = buffer.clone();
							buffer.clear();

							let tx_and_connected = {
								let request = local_local_request.read();
								let ws_request = request
									.get_ws_request()
									.expect("request should be WebSocket");
								if ws_request.is_connected {
									ws_request.websocket.as_ref().map(|ws| Arc::clone(&ws.tx))
								} else {
									None
								}
							};

							if let Some(tx) = tx_and_connected {
								info!("Sending message");

								tx.lock()
									.await
									.send(reqwest_websocket::Message::Text(text.clone()))
									.await
									.expect("sending WebSocket message should succeed");

								let mut request = local_local_request.write();
								let ws_request = request
									.get_ws_request_mut()
									.expect("request should be WebSocket");
								ws_request.messages.push(Message {
									timestamp: Local::now(),
									sender: Sender::You,
									content: MessageType::Text(text),
								});
							}
						}
					}
				}
			});

			loop {
				if let Some(request) = local_request.try_read() {
					let ws_request = request.get_ws_request()?;

					if !ws_request.is_connected {
						break;
					}

					let messages = &ws_request.messages[last_length..];

					for message in messages {
						println!(
							"=== {} - New {} message from {} ===\n{}",
							message.timestamp.format("%H:%M:%S %d/%m/%Y"),
							message.content,
							message.sender,
							message.content.to_content()
						)
					}

					last_length = ws_request.messages.len();
				}
			}
		}

		Ok(())
	}
}
