use crate::models::protocol::graphql::graphql::GraphqlRequest;
use crate::models::protocol::http::http::HttpRequest;
use crate::models::protocol::ws::ws::WsRequest;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ProtocolTypeError {
	#[error("The request is not an HTTP request")]
	NotAnHttpRequest,
	#[error("The request is not an websocket request")]
	NotAWsRequest,
	#[error("The request is not a GraphQL request")]
	NotAGraphqlRequest,
}

#[derive(Debug, Clone, EnumString, Display, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::enum_variant_names)]
pub enum Protocol {
	#[serde(rename = "http", alias = "http", alias = "HTTP")]
	#[strum(to_string = "HTTP")]
	HttpRequest(HttpRequest),

	#[serde(rename = "websocket", alias = "websocket", alias = "WEBSOCKET")]
	#[strum(to_string = "websocket")]
	WsRequest(WsRequest),

	#[serde(rename = "graphql", alias = "graphql", alias = "GRAPHQL")]
	#[strum(to_string = "graphql")]
	GraphqlRequest(GraphqlRequest),
}

impl Default for Protocol {
	fn default() -> Self {
		Protocol::HttpRequest(HttpRequest::default())
	}
}
