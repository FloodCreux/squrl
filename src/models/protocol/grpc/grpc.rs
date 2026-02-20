use serde::{Deserialize, Serialize};

/// Model for a gRPC request.
///
/// gRPC requests require a `.proto` file to define the service contract,
/// a fully-qualified service name, a method name, and a JSON message body
/// that will be dynamically converted to protobuf binary format at send time.
///
/// Import paths are additional directories to search when resolving proto
/// imports (similar to protoc's `-I` flag).
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GrpcRequest {
	/// Path to the `.proto` file defining the service.
	pub proto_file: String,
	/// Additional import paths for resolving proto imports.
	pub import_paths: Vec<String>,
	/// Fully-qualified service name (e.g. `"helloworld.Greeter"`).
	pub service: String,
	/// Method name (e.g. `"SayHello"`).
	pub method: String,
	/// Request message body as JSON (will be converted to protobuf).
	pub message: String,
}
