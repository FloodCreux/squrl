use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Default, Debug, Copy, Clone, EnumString, Display, Serialize, Deserialize)]
pub enum Method {
	#[default]
	#[strum(to_string = "GET")]
	GET,
	#[strum(to_string = "POST")]
	POST,
	#[strum(to_string = "PUT")]
	PUT,
	#[strum(to_string = "PATCH")]
	PATCH,
	#[strum(to_string = "DELETE")]
	DELETE,
	#[strum(to_string = "OPTIONS")]
	OPTIONS,
	#[strum(to_string = "HEAD")]
	HEAD,
	#[strum(to_string = "TRACE")]
	TRACE,
	#[strum(to_string = "CONNECT")]
	CONNECT,
}
