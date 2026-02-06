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

	/// Add query param
	/// (can be used multiple times)
	#[arg(long, action = clap::ArgAction::Append, num_args = 2, value_names = ["KEY", "VALUE"], display_order = 3)]
	pub add_param: Vec<String>,

	#[command(flatten)]
	pub auth: AuthArgs,

	#[arg(long, default_value_t = false, display_order = 7)]
	pub no_base_headers: bool,

	/// Add header
	/// (can be used multiple times)
	#[arg(long, action = clap::ArgAction::Append, num_args = 2, value_names = ["KEY", "VALUE"], display_order = 8)]
	pub add_header: Vec<String>,

	#[command(flatten)]
	pub body: BodyArgs,

	#[arg(long, display_order = 17)]
	pub pre_request_script: Option<String>,

	#[arg(long, display_order = 18)]
	pub post_request_script: Option<String>,

	#[arg(long, default_value_t = false, display_order = 19)]
	pub no_proxy: bool,

	#[arg(long, default_value_t = false, display_order = 20)]
	pub no_redirects: bool,

	#[arg(long, default_value_t = 30000, display_order = 21)]
	pub timeout: u32,

	#[arg(long, default_value_t = false, display_order = 22)]
	pub no_cookies: bool,

	#[arg(long, default_value_t = false, display_order = 23)]
	pub no_pretty: bool,

	#[arg(long, default_value_t = false, display_order = 24)]
	pub accept_invalid_certs: bool,

	#[arg(long, default_value_t = false, display_order = 25)]
	pub accept_invalid_hostnames: bool,
}

#[derive(clap::Args, Debug, Clone)]
#[group(multiple = false)]
pub struct AuthArgs {
	#[arg(long, group = "auth", action = clap::ArgAction::Set, num_args = 2, value_names = ["USERNAME", "PASSWORD"], display_order = 3)]
	pub auth_basic: Vec<String>,

	#[arg(long, group = "auth", action = clap::ArgAction::Set, num_args = 1, value_name = "TOKEN", display_order = 4)]
	pub auth_bearer_token: Vec<String>,

	#[arg(long, group = "auth", action = clap::ArgAction::Set, num_args = 4, value_names = ["ALGORITHM", "SECRET_TYPE", "SECRET", "PAYLOAD"], display_order = 5)]
	pub auth_jwt_token: Vec<String>,

	#[arg(long, group = "auth", action = clap::ArgAction::Set, num_args = 3, value_names = ["USERNAME", "PASSWORD", "WWW_AUTHENTICATE_HEADER"], display_order = 6)]
	pub auth_digest: Vec<String>,
}

#[derive(clap::Args, Debug, Clone)]
#[group(multiple = false)]
pub struct BodyArgs {
	#[arg(long, group = "body", value_name = "FILE_PATH", value_hint = clap::ValueHint::FilePath, display_order = 9)]
	pub body_file: Option<String>,

	#[arg(long, group = "body", action = clap::ArgAction::Append, num_args = 2, value_names = ["KEY", "VALUE"], display_order = 10)]
	pub add_body_multipart: Vec<String>,

	#[arg(long, group = "body", action = clap::ArgAction::Append, num_args = 2, value_names = ["KEY", "VALUE"], display_order = 11)]
	pub add_body_form: Vec<String>,

	#[arg(long, group = "body", value_name = "TEXT", display_order = 12)]
	pub body_raw: Option<String>,

	#[arg(long, group = "body", value_name = "JSON", display_order = 13)]
	pub body_json: Option<String>,

	#[arg(long, group = "body", value_name = "XML", display_order = 14)]
	pub body_xml: Option<String>,

	#[arg(long, group = "body", value_name = "HTML", display_order = 15)]
	pub body_html: Option<String>,

	#[arg(long, group = "body", value_name = "JAVASCRIPT", display_order = 16)]
	pub body_javascript: Option<String>,
}
