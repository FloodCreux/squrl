use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, MethodDescriptor};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace};

use crate::app::App;
use crate::app::request::send::RequestResponseError;
use crate::app::request::send::RequestResponseError::CouldNotDecodeResponse;
use crate::models::environment::Environment;
use crate::models::protocol::grpc::grpc::GrpcRequest;
use crate::models::request::Request;
use crate::models::response::{RequestResponse, ResponseContent};

/// Parse a `.proto` file (with optional import paths) into a `DescriptorPool`.
fn parse_proto_file(proto_file: &str, import_paths: &[String]) -> anyhow::Result<DescriptorPool> {
	let proto_path = Path::new(proto_file);

	// Determine include directories: the proto file's parent dir + any user-specified import paths
	let mut includes: Vec<&Path> = Vec::new();

	if let Some(parent) = proto_path.parent() {
		if parent.as_os_str().is_empty() {
			includes.push(Path::new("."));
		} else {
			includes.push(parent);
		}
	} else {
		includes.push(Path::new("."));
	}

	for import_path in import_paths {
		includes.push(Path::new(import_path));
	}

	let file_name = proto_path
		.file_name()
		.ok_or_else(|| anyhow::anyhow!("Invalid proto file path"))?
		.to_str()
		.ok_or_else(|| anyhow::anyhow!("Proto file path is not valid UTF-8"))?;

	let file_descriptor_set = protox::Compiler::new(includes)?
		.include_imports(true)
		.open_file(file_name)?
		.file_descriptor_set();

	let pool = DescriptorPool::from_file_descriptor_set(file_descriptor_set)?;

	Ok(pool)
}

/// Resolve a service and method from the descriptor pool.
fn resolve_method(
	pool: &DescriptorPool,
	service_name: &str,
	method_name: &str,
) -> anyhow::Result<MethodDescriptor> {
	let service = pool
		.get_service_by_name(service_name)
		.ok_or_else(|| anyhow::anyhow!("Service '{}' not found in proto file", service_name))?;

	let method = service
		.methods()
		.find(|m| m.name() == method_name)
		.ok_or_else(|| {
			anyhow::anyhow!(
				"Method '{}' not found in service '{}'",
				method_name,
				service_name
			)
		})?;

	Ok(method)
}

/// Encode a JSON string into a protobuf `DynamicMessage` using the method's input type descriptor.
fn json_to_protobuf(method: &MethodDescriptor, json_message: &str) -> anyhow::Result<Vec<u8>> {
	let input_descriptor = method.input();

	let mut deserializer = serde_json::Deserializer::from_str(json_message);
	let dynamic_message = DynamicMessage::deserialize(input_descriptor, &mut deserializer)?;

	let mut buf = Vec::new();
	dynamic_message.encode(&mut buf)?;

	Ok(buf)
}

/// Decode a protobuf response body into JSON using the method's output type descriptor.
fn protobuf_to_json(method: &MethodDescriptor, protobuf_bytes: &[u8]) -> anyhow::Result<String> {
	let output_descriptor = method.output();
	let dynamic_message = DynamicMessage::decode(output_descriptor, protobuf_bytes)?;

	let mut serializer = serde_json::Serializer::pretty(Vec::<u8>::new());
	let serialize_options = prost_reflect::SerializeOptions::new().stringify_64_bit_integers(false);
	dynamic_message.serialize_with_options(&mut serializer, &serialize_options)?;

	let json_bytes: Vec<u8> = serializer.into_inner();
	Ok(String::from_utf8(json_bytes)?)
}

/// Frame a protobuf message with the gRPC Length-Prefixed-Message framing.
/// Format: 1 byte compressed flag (0) + 4 bytes big-endian message length + message bytes.
fn grpc_frame(message: &[u8]) -> Bytes {
	let mut buf = BytesMut::with_capacity(5 + message.len());
	buf.put_u8(0); // compression flag: not compressed
	buf.put_u32(message.len() as u32);
	buf.put_slice(message);
	buf.freeze()
}

/// Extract the protobuf message bytes from a gRPC Length-Prefixed-Message frame.
fn grpc_unframe(mut data: Bytes) -> anyhow::Result<Bytes> {
	if data.len() < 5 {
		return Err(anyhow::anyhow!(
			"gRPC response too short ({} bytes)",
			data.len()
		));
	}

	let _compressed = data.get_u8();
	let length = data.get_u32() as usize;

	if data.len() < length {
		return Err(anyhow::anyhow!(
			"gRPC response frame length mismatch: expected {} bytes, got {}",
			length,
			data.len()
		));
	}

	Ok(data.slice(..length))
}

/// Send a gRPC unary request using raw HTTP/2 via reqwest.
///
/// This function:
/// 1. Parses the .proto file to get service/method descriptors
/// 2. Converts the JSON message body to protobuf binary
/// 3. Sends the request via HTTP/2 with gRPC framing
/// 4. Decodes the protobuf response back to JSON
pub async fn send_grpc_request(
	grpc_request: &GrpcRequest,
	url: &str,
	headers: &[(String, String)],
	local_request: Arc<RwLock<Request>>,
	env: &Option<Arc<RwLock<Environment>>>,
) -> Result<RequestResponse, RequestResponseError> {
	info!("Sending gRPC request");

	let (cancellation_token, timeout_ms) = {
		let mut request = local_request.write();
		request.is_pending = true;
		let cancellation_token = request.cancellation_token.clone();
		let timeout_ms = request.settings.timeout.as_u32().unwrap_or(30000) as u64;
		(cancellation_token, timeout_ms)
	};

	let timeout = tokio::time::sleep(Duration::from_millis(timeout_ms));
	let request_start = Instant::now();
	let elapsed_time: Duration;

	// Phase 1: Parse the proto file and resolve the method
	let method = match parse_proto_file(&grpc_request.proto_file, &grpc_request.import_paths)
		.and_then(|pool| resolve_method(&pool, &grpc_request.service, &grpc_request.method))
	{
		Ok(method) => method,
		Err(e) => {
			let mut request = local_request.write();
			request.is_pending = false;
			request.cancellation_token = CancellationToken::new();
			return Ok(RequestResponse {
				duration: Some(format!("{:?}", request_start.elapsed())),
				status_code: Some(format!("PROTO ERROR: {}", e)),
				content: Some(ResponseContent::Body(e.to_string())),
				cookies: None,
				headers: vec![],
			});
		}
	};

	// Phase 2: Encode the JSON message to protobuf
	let message_json = if grpc_request.message.trim().is_empty() {
		"{}".to_string()
	} else {
		grpc_request.message.clone()
	};

	let encoded_message = match json_to_protobuf(&method, &message_json) {
		Ok(bytes) => bytes,
		Err(e) => {
			let mut request = local_request.write();
			request.is_pending = false;
			request.cancellation_token = CancellationToken::new();
			return Ok(RequestResponse {
				duration: Some(format!("{:?}", request_start.elapsed())),
				status_code: Some(format!("ENCODE ERROR: {}", e)),
				content: Some(ResponseContent::Body(e.to_string())),
				cookies: None,
				headers: vec![],
			});
		}
	};

	// Phase 3: Build the gRPC path
	let grpc_path = format!(
		"{}/{}/{}",
		url.trim_end_matches('/'),
		grpc_request.service,
		grpc_request.method
	);

	// Phase 4: Send via reqwest with HTTP/2 and gRPC framing
	let framed_body = grpc_frame(&encoded_message);

	let client = reqwest::Client::builder()
		.http2_prior_knowledge()
		.build()
		.map_err(|e| {
			error!("Failed to build HTTP/2 client: {}", e);
			CouldNotDecodeResponse
		})?;

	let mut request_builder = client
		.post(&grpc_path)
		.header("content-type", "application/grpc")
		.header("te", "trailers");

	for (key, value) in headers {
		request_builder = request_builder.header(key.as_str(), value.as_str());
	}

	let mut response = tokio::select! {
		_ = cancellation_token.cancelled() => {
			elapsed_time = request_start.elapsed();
			RequestResponse {
				duration: None,
				status_code: Some(String::from("CANCELED")),
				content: None,
				cookies: None,
				headers: vec![],
			}
		},
		_ = timeout => {
			elapsed_time = request_start.elapsed();
			RequestResponse {
				duration: None,
				status_code: Some(String::from("TIMEOUT")),
				content: None,
				cookies: None,
				headers: vec![],
			}
		},
		response = request_builder.body(framed_body).send() => match response {
			Ok(response) => {
				info!("gRPC response received");
				elapsed_time = request_start.elapsed();

				let status_code = response.status().to_string();

				let resp_headers: Vec<(String, String)> = response
					.headers()
					.iter()
					.map(|(name, value)| {
						(name.to_string(), value.to_str().unwrap_or("").to_string())
					})
					.collect();

				// Check for gRPC status in headers/trailers
				let grpc_status = resp_headers
					.iter()
					.find(|(k, _)| k == "grpc-status")
					.map(|(_, v)| v.clone());

				let grpc_message = resp_headers
					.iter()
					.find(|(k, _)| k == "grpc-message")
					.map(|(_, v)| v.clone());

				match response.bytes().await {
					Ok(body_bytes) => {
						let content = if body_bytes.len() >= 5 {
							match grpc_unframe(body_bytes) {
								Ok(message_bytes) => {
									match protobuf_to_json(&method, &message_bytes) {
										Ok(json) => json,
										Err(e) => format!("Failed to decode response: {}", e),
									}
								}
								Err(e) => format!("Failed to unframe gRPC response: {}", e),
							}
						} else if body_bytes.is_empty() {
							// Empty response is valid for some gRPC calls
							String::from("{}")
						} else {
							format!("Unexpected response body ({} bytes)", body_bytes.len())
						};

						let display_status = match &grpc_status {
							Some(s) if s == "0" => format!("{} (OK)", status_code),
							Some(s) => {
								let msg = grpc_message.as_deref().unwrap_or("");
								format!("{} (gRPC {}{})", status_code, s,
									if msg.is_empty() { String::new() } else { format!(": {}", msg) })
							}
							None => status_code,
						};

						RequestResponse {
							duration: None,
							status_code: Some(display_status),
							content: Some(ResponseContent::Body(content)),
							cookies: None,
							headers: resp_headers,
						}
					}
					Err(_) => {
						RequestResponse {
							duration: None,
							status_code: Some(status_code),
							content: Some(ResponseContent::Body("Failed to read response body".to_string())),
							cookies: None,
							headers: resp_headers,
						}
					}
				}
			}
			Err(error) => {
				error!("gRPC sending error: {}", error);
				elapsed_time = request_start.elapsed();

				let response_status_code = error.status().map(|s| s.to_string());
				let result_body = ResponseContent::Body(error.to_string());

				RequestResponse {
					duration: None,
					status_code: response_status_code,
					content: Some(result_body),
					cookies: None,
					headers: vec![],
				}
			}
		}
	};

	response.duration = Some(format!("{:?}", elapsed_time));
	trace!("gRPC request sent");

	// Post-request script
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
