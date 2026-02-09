use crate::cli::commands::request_commands::new::NewRequestCommand;
use crate::cli::commands::request_commands::send::SendCommand;

impl App {
	pub async fn try_request(
		&mut self,
		new_request_command: &NewRequestCommand,
		send_command: &SendCommand,
	) -> anyhow::Result<()> {
		// let new_request =
	}
}
