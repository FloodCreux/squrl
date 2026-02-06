use crate::models::protocol::http::method::Method;
use crate::models::protocol::protocol::Protocol;

#[derive(clap::Args, Debug, Clone)]
pub struct NewRequestCommand {
	#[arg(short, long, value_hint = clap::ValueHint::Url, default_value_t = String::new(), display_order = 0)]
	pub url: String,

	#[arg(short, long, default_value_t = Protocol::default(), display_order=1)]
	pub protocol: Protocol,

	#[arg(short, long, default_value_t = Method::GET, display_order=2)]
	pub method: Method,
}
