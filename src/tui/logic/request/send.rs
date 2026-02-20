use crate::app::App;
use crate::app::request::grpc::send::send_grpc_request;
use crate::app::request::http::send::send_http_request;
use crate::app::request::ws::send::send_ws_request;
use crate::models::auth::auth::Auth;
use crate::models::protocol::protocol::Protocol;
use futures_util::SinkExt;
use reqwest_websocket::CloseCode;
use std::sync::Arc;
use tokio::task;
use tracing::info;

impl App<'_> {
	pub async fn tui_send_request(&mut self) {
		let Some(local_selected_request) = self.get_selected_request_as_local() else {
			return;
		};

		{
			let selected_request = local_selected_request.read();

			if selected_request.is_pending {
				selected_request.cancellation_token.cancel();
				info!("Request canceled");
				return;
			}
		}

		// Check for an active WebSocket connection that needs closing.
		// Clone the tx Arc so we can drop the write guard before awaiting.
		let ws_disconnect = {
			let mut selected_request = local_selected_request.write();
			match &mut selected_request.protocol {
				Protocol::HttpRequest(_)
				| Protocol::GraphqlRequest(_)
				| Protocol::GrpcRequest(_) => None,
				Protocol::WsRequest(ws_request) => {
					if ws_request.is_connected
						&& let Some(websocket) = ws_request.websocket.clone()
					{
						// Take ownership of the websocket before dropping the guard
						ws_request.websocket = None;
						ws_request.is_connected = false;
						drop(websocket.rx);
						Some(websocket.tx)
					} else {
						None
					}
				}
			}
		};
		// Guard is dropped here — safe to await

		if let Some(tx) = ws_disconnect {
			if let Err(e) = tx
				.lock()
				.await
				.send(reqwest_websocket::Message::Close {
					code: CloseCode::Normal,
					reason: String::new(),
				})
				.await
			{
				tracing::warn!("Failed to send WebSocket close frame: {e}");
			}

			if let Err(e) = tx.lock().await.close().await {
				tracing::warn!("Failed to close WebSocket connection: {e}");
			}
			return;
		}

		/* PRE-REQUEST SCRIPT */

		// prepare_request is synchronous — safe to call while holding the lock.
		let (prepared, protocol) = {
			let mut selected_request = local_selected_request.write();

			let prepared = match self.prepare_request(&mut selected_request) {
				Ok(result) => result,
				Err(prepare_request_error) => {
					selected_request.response.status_code = Some(prepare_request_error.to_string());
					return;
				}
			};

			let protocol = selected_request.protocol.clone();
			(prepared, protocol)
		};
		// Guard is dropped here — safe to await for file body finalization

		let prepared_request = match App::finalize_prepared_request(prepared).await {
			Ok(builder) => builder,
			Err(finalize_error) => {
				let mut selected_request = local_selected_request.write();
				selected_request.response.status_code = Some(finalize_error.to_string());
				return;
			}
		};

		let Some(local_selected_request) = self.get_selected_request_as_local() else {
			return;
		};
		let local_env = self.get_selected_env_as_local();

		let local_should_refresh_scrollbars = Arc::clone(&self.core.received_response);

		/* SEND REQUEST */

		task::spawn(async move {
			let response = match protocol {
				Protocol::HttpRequest(_) | Protocol::GraphqlRequest(_) => {
					send_http_request(prepared_request, local_selected_request.clone(), &local_env)
						.await
				}
				Protocol::GrpcRequest(ref grpc_req) => {
					let url = {
						let req = local_selected_request.read();
						req.url.clone()
					};
					let headers = {
						let req = local_selected_request.read();
						req.headers
							.iter()
							.filter(|h| h.enabled)
							.map(|h| (h.data.0.clone(), h.data.1.clone()))
							.collect::<Vec<_>>()
					};
					send_grpc_request(
						grpc_req,
						&url,
						&headers,
						local_selected_request.clone(),
						&local_env,
					)
					.await
				}
				Protocol::WsRequest(_) => {
					send_ws_request(
						prepared_request,
						local_selected_request.clone(),
						&local_env,
						local_should_refresh_scrollbars.clone(),
					)
					.await
				}
			};

			match response {
				Ok(response) => {
					let mut selected_request = local_selected_request.write();

					if let Auth::Digest(digest) = &mut selected_request.auth {
						digest.update_from_www_authenticate_header(&response.headers)
					}

					selected_request.response = response;

					*local_should_refresh_scrollbars.lock() = true;
				}
				Err(response_error) => {
					let mut selected_request = local_selected_request.write();
					selected_request.response.status_code = Some(response_error.to_string());
				}
			}
		});
	}
}
