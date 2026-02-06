use crate::cli::commands::request_commands::new::NewRequestCommand;
use crate::models::request::{KeyValue, Request};

pub fn create_request_from_new_request_command(
	request_name: String,
	new_request_command: NewRequestCommand,
) -> anyhow::Result<Request> {
	let params = string_array_to_key_value_array(new_request_command.add_param);
	let headers = string_array_to_key_value_array(new_request_command.headers);
}

fn string_array_to_key_value_array(string_array: Vec<String>) -> Vec<KeyValue> {
	let mut key_value_array: Vec<KeyValue> = vec![];

	for i in (0..string_array.len()).step_by(2) {
		key_value_array.push(KeyValue {
			enabled: true,
			data: (string_array[i].clone(), string_array[i + 1].clone()),
		})
	}

	key_value_array
}
