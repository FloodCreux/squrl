use std::sync::Arc;

use parking_lot::RwLock;

use crate::app::App;
use crate::cli::commands::request_commands::new::NewRequestCommand;
use crate::cli::commands::request_commands::send::SendCommand;
use crate::cli::request::new::create_request_from_new_request_command;

impl App<'_> {
	pub async fn try_request(
		&mut self,
		new_request_command: &NewRequestCommand,
		send_command: &SendCommand,
	) -> anyhow::Result<()> {
		let new_request =
			create_request_from_new_request_command(String::new(), new_request_command.clone())?;
		let local_request = Arc::new(RwLock::new(new_request));

		self.local_send_request(send_command, local_request, None)
			.await?;

		Ok(())
	}
}
