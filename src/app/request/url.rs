use anyhow::anyhow;
use thiserror::Error;
use tracing::info;

use crate::app::App;

#[derive(Error, Debug)]
pub enum UrlError {
	#[error("The URL is empty")]
	UrlIsEmpty,
}

impl App<'_> {
	pub fn modify_request_url(
		&mut self,
		collection_index: usize,
		request_index: usize,
		url: String,
	) -> anyhow::Result<()> {
		if url.trim().is_empty() {
			return Err(anyhow!(UrlError::UrlIsEmpty));
		}

		self.with_request_write(collection_index, request_index, |req| {
			req.update_url_and_params(url);
			info!("URL set to \"{}\"", &req.url);
		});

		Ok(())
	}
}
