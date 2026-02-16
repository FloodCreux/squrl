use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::app::App;
use crate::app::request::send::RequestResponseError;
use crate::app::request::send::RequestResponseError::CouldNotDecodeResponse;
use crate::models::environment::Environment;
use crate::models::protocol::http::body::find_file_format_in_content_type;
use crate::models::request::Request;
use crate::models::response::{ImageResponse, RequestResponse, ResponseContent};
use rayon::prelude::*;
use reqwest::header::CONTENT_TYPE;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};

pub async fn send_http_request(
	prepared_request: reqwest_middleware::RequestBuilder,
	local_request: Arc<RwLock<Request>>,
	env: &Option<Arc<RwLock<Environment>>>,
) -> Result<RequestResponse, RequestResponseError> {
	info!("Sending request");

	// Extract the values we need from the lock, then drop it before any await.
	let (cancellation_token, timeout_ms, pretty_print) = {
		let mut request = local_request.write();
		request.is_pending = true;
		let cancellation_token = request.cancellation_token.clone();
		let timeout_ms = request.settings.timeout.as_u32().unwrap_or(30000) as u64;
		let pretty_print = request
			.settings
			.pretty_print_response_content
			.as_bool()
			.unwrap_or(true);
		(cancellation_token, timeout_ms, pretty_print)
	};
	// Write guard is dropped here — safe to await

	let timeout = tokio::time::sleep(Duration::from_millis(timeout_ms));

	let request_start = Instant::now();
	let elapsed_time: Duration;

	let mut response = tokio::select! {
		_ = cancellation_token.cancelled() => {
			elapsed_time = request_start.elapsed();

			RequestResponse {
				duration: None,
				status_code: Some(String::from("CANCELED")),
				content: None,
				cookies: None,
				headers: vec![]
			}
		},
		_ = timeout => {
			elapsed_time = request_start.elapsed();

			RequestResponse {
				duration: None,
				status_code: Some(String::from("TIMEOUT")),
				content: None,
				cookies: None,
				headers: vec![]
			}
		},
		response = prepared_request.send() => match response {
			Ok(response) => {
				info!("Response received");

				elapsed_time = request_start.elapsed();

				let status_code = response.status().to_string();

				let mut is_image = false;

				let headers: Vec<(String, String)> = response.headers().clone()
					.iter()
					.map(|(header_name, header_value)| {
						let value = header_value.to_str().unwrap_or("").to_string();

						if header_name == CONTENT_TYPE && value.starts_with("image/") {
							is_image = true;
						}

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

				let response_content = match is_image {
					true => match response.bytes().await {
						Ok(content) => {
							let image = image::load_from_memory(content.as_ref());

							ResponseContent::Image(ImageResponse {
								data: content.to_vec(),
								image: image.ok(),
							})
						}
						Err(_) => return Err(CouldNotDecodeResponse)
					},
					false => match response.bytes().await {
						Ok(bytes) => match String::from_utf8(bytes.to_vec()) {
							Ok(mut result_body) => {
								// If a file format has been found in the content-type header
								if let Some(file_format) = find_file_format_in_content_type(&headers) {
									// If the request response content can be pretty printed
									if pretty_print {
										// Match the file format
										if file_format.as_str() == "json" {
														  result_body = jsonxf::pretty_print(&result_body).unwrap_or(result_body);
													  }
									}
								}

								ResponseContent::Body(result_body)
							},
							Err(_) => ResponseContent::Body(format!("{:#X?}", bytes))
						},
						Err(_) => return Err(CouldNotDecodeResponse)
					}
				};

				RequestResponse {
					duration: None,
					status_code: Some(status_code),
					content: Some(response_content),
					cookies: Some(cookies),
					headers
				}
			},
			Err(error) => {
				error!("Sending error: {}", error);

				elapsed_time = request_start.elapsed();

				let response_status_code;

				if let Some(status_code) = error.status() {
					response_status_code = Some(status_code.to_string());
				} else {
					response_status_code = None;
				}

				let result_body = ResponseContent::Body(error.to_string());

				RequestResponse {
					duration: None,
					status_code: response_status_code,
					content: Some(result_body),
					cookies: None,
					headers: vec![]
				}
			}
		}
	};

	response.duration = Some(format!("{:?}", elapsed_time));

	trace!("Request sent");

	/* POST-REQUEST SCRIPT */

	// Re-acquire a read guard only for the post-request script.
	let request = local_request.read();
	let (modified_response, post_request_output) =
		App::handle_post_request_script(&request, response, env)?;
	drop(request);

	{
		let mut request = local_request.write();

		request.console_output.post_request_output = post_request_output;
		request.is_pending = false;
		request.cancellation_token = CancellationToken::new();
	}

	Ok(modified_response)
}
