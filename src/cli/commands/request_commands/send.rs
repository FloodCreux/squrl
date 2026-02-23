#[derive(clap::Args, Debug, Clone)]
pub struct SendCommand {
	#[arg(long, default_value_t = false)]
	pub hide_content: bool,

	#[arg(long, default_value_t = false)]
	pub status_code: bool,

	#[arg(long, default_value_t = false)]
	pub duration: bool,

	#[arg(long, default_value_t = false)]
	pub headers: bool,

	#[arg(long, default_value_t = false)]
	pub cookies: bool,

	#[arg(long, default_value_t = false)]
	pub console: bool,

	#[arg(long, default_value_t = false)]
	pub request_name: bool,

	/// Name of the global environment to use, e.g. my_env (from file .env.my_env)
	#[arg(long, value_name = "ENV_NAME", display_order = 98)]
	pub env: Option<String>,

	/// Name of the collection-scoped environment to use (defined in collection file or squrl-env.json)
	#[arg(long, value_name = "COLLECTION_ENV_NAME", display_order = 99)]
	pub collection_env: Option<String>,
}
