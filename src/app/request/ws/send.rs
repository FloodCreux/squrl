use crate::app::app::App;
use crate::app::constants::{WS_CONNECTION_TIMEOUT, WS_POLL_INTERVAL};
use crate::app::request::send::RequestResponseError;
use crate::models::environment::Environment;
use crate::models::protocol::ws::message_type::MessageType;
use crate::models::protocol::ws::ws::{Message, Sender, Websocket};
use crate::models::request::Request;
use crate::models::response::{RequestResponse, ResponseContent};
use chrono::Local;
use futures_util::{StreamExt, TryStreamExt};
use parking_lot::{Mutex, RwLock};
use rayon::prelude::*;
use reqwest_websocket::Upgrade;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};

/// Intermediate result from the `tokio::select!` in WebSocket send, used to
/// transfer the WebSocket split back to the caller without holding a lock.
enum WsSendOutcome {
	/// The request was cancelled, timed out, or encountered an error.
	Response(RequestResponse, Duration),
	/// The HTTP upgrade succeeded; carries the response metadata and the
	/// WebSocket halves that need to be stored in the request.
	Upgraded {
		response: RequestResponse,
		elapsed_time: Duration,
		websocket: Websocket,
	},
}

pub async fn send_ws_request(
	prepared_request: reqwest_middleware::RequestBuilder,
	local_request: Arc<RwLock<Request>>,
	env: &Option<Arc<RwLock<Environment>>>,
	received_response: Arc<Mutex<bool>>,
) -> Result<RequestResponse, RequestResponseError> {
	info!("Sending request");

	// Extract values from the lock, then drop it before any await.
	let cancellation_token = {
		let mut request = local_request.write();
		request.is_pending = true;
		let cancellation_token = request.cancellation_token.clone();
		let ws_request = request.get_ws_request_mut().unwrap();
		ws_request.is_connected = false;
		cancellation_token
	};
	// Write guard is dropped here — safe to await

	let timeout = tokio::time::sleep(WS_CONNECTION_TIMEOUT);

	let request_start = Instant::now();
	let outcome = tokio::select! {
		_ = cancellation_token.cancelled() => {
			WsSendOutcome::Response(
				RequestResponse {
					duration: None,
					status_code: Some(String::from("CANCELED")),
					content: None,
					cookies: None,
					headers: vec![],
				},
				request_start.elapsed(),
			)
		},
		_ = timeout => {
			WsSendOutcome::Response(
				RequestResponse {
					duration: None,
					status_code: Some(String::from("TIMEOUT")),
					content: None,
					cookies: None,
					headers: vec![],
				},
				request_start.elapsed(),
			)
		},
		response = prepared_request.upgrade().send() => match response {
			Ok(response) => {
				info!("Response received");

				let elapsed_time = request_start.elapsed();

				let status_code = response.status().to_string();

				let headers: Vec<(String, String)> = response.headers()
					.clone()
					.iter()
					.map(|(header_name, header_value)| {
						let value = header_value.to_str().unwrap_or("").to_string();
						(header_name.to_string(), value)
					})
					.collect();

				let cookies = response.cookies()
					.par_bridge()
					.map(|cookie| {
						format!("{}: {}", cookie.name(), cookie.value())
					})
					.collect::<Vec<String>>()
					.join("\n");

				let ws = match response.into_websocket().await {
					Ok(ws) => ws,
					Err(error) => return Err(RequestResponseError::WebsocketError(error))
				};
				let (tx, rx) = ws.split();

				WsSendOutcome::Upgraded {
					response: RequestResponse {
						duration: None,
						status_code: Some(status_code),
						content: None,
						cookies: Some(cookies),
						headers,
					},
					elapsed_time,
				websocket: Websocket {
					rx: Arc::new(tokio::sync::Mutex::new(rx)),
					tx: Arc::new(tokio::sync::Mutex::new(tx)),
				},
				}
			},
			Err(error) => {
				error!("Sending error: {}", error);

				let error = error.to_string();
				let response_status_code = Some(error.clone());
				let result_body = ResponseContent::Body(error);

				WsSendOutcome::Response(
					RequestResponse {
						duration: None,
						status_code: response_status_code,
						content: Some(result_body),
						cookies: None,
						headers: vec![],
					},
					request_start.elapsed(),
				)
			}
		}
	};

	// Now re-acquire the lock to store the WebSocket and run the post-request script.
	let (mut response, elapsed_time, websocket_halves) = match outcome {
		WsSendOutcome::Response(response, elapsed) => (response, elapsed, None),
		WsSendOutcome::Upgraded {
			response,
			elapsed_time,
			websocket,
		} => (response, elapsed_time, Some(websocket)),
	};

	response.duration = Some(format!("{:?}", elapsed_time));

	trace!("Request sent");

	/* POST-REQUEST SCRIPT */

	// Re-acquire read guard for post-request script, then drop it.
	let request = local_request.read();
	let (modified_response, post_request_output) =
		App::handle_post_request_script(&request, response, env)?;
	drop(request);

	{
		let mut request = local_request.write();

		request.console_output.post_request_output = post_request_output;
		request.is_pending = false;
		request.cancellation_token = CancellationToken::new();

		let ws_request = request.get_ws_request_mut().unwrap();
		ws_request.messages = vec![];

		if modified_response.status_code != Some(String::from("101 Switching Protocols")) {
			return Ok(modified_response);
		}

		// Store the WebSocket halves and mark as connected.
		ws_request.websocket = websocket_halves;
		ws_request.is_connected = true;
	}

	let local_request = local_request.clone();
	let local_websocket = {
		let request = local_request.read();
		let ws_request = request.get_ws_request().unwrap();
		ws_request.websocket.clone().unwrap()
	};

	tokio::spawn(async move {
		'websocket_loop: loop {
			if cancellation_token.is_cancelled() {
				let mut request = local_request.write();
				let ws_request = request.get_ws_request_mut().unwrap();
				ws_request.is_connected = false;
				break 'websocket_loop;
			}

			let mut websocket_rx = local_websocket.rx.lock().await;
			let message = websocket_rx.try_next().await;
			drop(websocket_rx);
			match message {
				Ok(message) => {
					if let Some(message) = message {
						let message_type = match message {
							reqwest_websocket::Message::Text(text) => MessageType::Text(text),
							reqwest_websocket::Message::Binary(binary) => {
								MessageType::Binary(binary.to_vec().into_boxed_slice())
							}
							reqwest_websocket::Message::Ping(ping) => {
								MessageType::Ping(ping.to_vec().into_boxed_slice())
							}
							reqwest_websocket::Message::Pong(pong) => {
								MessageType::Pong(pong.to_vec().into_boxed_slice())
							}
							reqwest_websocket::Message::Close { code, reason } => {
								match reason.is_empty() {
									true => MessageType::Close(format!("Close code: {}", code)),
									false => MessageType::Close(format!(
										"Close code: {}, reason: {}",
										code, reason
									)),
								}
							}
						};

						let mut request = local_request.write();
						let ws_request = request.get_ws_request_mut().unwrap();
						ws_request.messages.push(Message {
							timestamp: Local::now(),
							content: message_type,
							sender: Sender::Server,
						});

						*received_response.lock() = true;
					}
				}
				Err(error) => {
					let mut request = local_request.write();
					let ws_request = request.get_ws_request_mut().unwrap();
					ws_request.is_connected = false;
					ws_request.messages.push(Message {
						timestamp: Local::now(),
						content: MessageType::Close(format!("Connection closed: {}", error)),
						sender: Sender::Server,
					});
					break 'websocket_loop;
				}
			}

			sleep(WS_POLL_INTERVAL).await;
		}
	});

	Ok(modified_response)
}
