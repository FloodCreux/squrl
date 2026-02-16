use tracing::info;

use crate::app::App;
use crate::models::protocol::http::method::Method;

impl App<'_> {
	pub fn modify_request_method(
		&mut self,
		collection_index: usize,
		request_index: usize,
		method: Method,
	) -> anyhow::Result<()> {
		self.with_request_write_result(collection_index, request_index, |req| {
			let http = req.get_http_request_mut()?;
			info!("Method set to \"{}\"", method);
			http.method = method;
			Ok(())
		})
	}
}
