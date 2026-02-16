use boa_engine::{Context, Source};
use indexmap::IndexMap;
use tracing::{info, trace};

use crate::app::App;
use crate::models::request::Request;
use crate::models::response::RequestResponse;
use crate::models::scripts::ScriptType;

impl App<'_> {
	pub fn modify_request_script(
		&mut self,
		collection_index: usize,
		request_index: usize,
		script_type: &ScriptType,
		script: Option<String>,
	) -> anyhow::Result<()> {
		self.with_request_write(collection_index, request_index, |req| {
			match script_type {
				ScriptType::Pre => req.scripts.pre_request_script = script,
				ScriptType::Post => req.scripts.post_request_script = script,
			}

			info!("{}-request script set", script_type);
		});

		Ok(())
	}
}

const JS_CONSOLE: &str = r#"
let console_log_output = "";

globalThis.console = {
  log: (msg) => {
    console_log_output += msg + '\n';
    return msg;
  }
}
"#;

const JS_UTILS: &str = r#"
function pretty_print(data) {
    console.log(JSON.stringify(data, null, 2));
}
"#;

pub fn execute_pre_request_script(
	user_script: &String,
	request: &Request,
	env: Option<IndexMap<String, String>>,
) -> (Option<Request>, Option<IndexMap<String, String>>, String) {
	// Instantiate the execution context
	let mut context = Context::default();

	let request_json = match serde_json::to_string(request) {
		Ok(json) => json,
		Err(e) => return (None, env, format!("Failed to serialize request: {e}")),
	};
	let env_json = match &env {
		Some(env) => match serde_json::to_string(env) {
			Ok(json) => json,
			Err(e) => return (None, None, format!("Failed to serialize environment: {e}")),
		},
		None => String::from("undefined"),
	};

	let script = format!(
		r#"
        let request = {request_json};
        let env = {env_json};

        {JS_CONSOLE}
        {JS_UTILS}

        /* Start of the user script */

        {user_script}

        /* End of the user script */

        JSON.stringify([request, env, console_log_output])
    "#
	);

	trace!("Executing pre-request script");

	let result = match context.eval(Source::from_bytes(&script)) {
		Ok(result) => result,
		Err(error) => return (None, env, error.to_string()),
	};

	let stringed_result = match result.as_string() {
		Some(s) => s.to_std_string_escaped(),
		None => return (None, env, "Script result was not a string".to_string()),
	};

	let (result_request, result_env_values, console_output) =
		match serde_json::from_str::<(Request, Option<IndexMap<String, String>>, String)>(
			&stringed_result,
		) {
			Ok((result_request, result_env_values, console_output)) => {
				(Some(result_request), result_env_values, console_output)
			}
			Err(error) => (None, env, error.to_string()),
		};

	(result_request, result_env_values, console_output)
}

pub fn execute_post_request_script(
	user_script: &String,
	response: &RequestResponse,
	env: Option<IndexMap<String, String>>,
) -> (
	Option<RequestResponse>,
	Option<IndexMap<String, String>>,
	String,
) {
	// Instantiate the execution context
	let mut context = Context::default();

	let response_json = match serde_json::to_string(response) {
		Ok(json) => json,
		Err(e) => return (None, env, format!("Failed to serialize response: {e}")),
	};
	let env_json = match &env {
		Some(env) => match serde_json::to_string(env) {
			Ok(json) => json,
			Err(e) => return (None, None, format!("Failed to serialize environment: {e}")),
		},
		None => String::from("undefined"),
	};

	let script = format!(
		r#"
        let response = {response_json};
        let env = {env_json};

        {JS_CONSOLE}
        {JS_UTILS}

        /* Start of the user script */

        {user_script}

        /* End of the user script */

        JSON.stringify([response, env, console_log_output])
    "#
	);

	trace!("Executing post-request script");

	let result = match context.eval(Source::from_bytes(&script)) {
		Ok(result) => result,
		Err(error) => return (None, env, error.to_string()),
	};

	let stringed_result = match result.as_string() {
		Some(s) => s.to_std_string_escaped(),
		None => return (None, env, "Script result was not a string".to_string()),
	};

	let (response_result, result_env_values, console_output) =
		match serde_json::from_str::<(RequestResponse, Option<IndexMap<String, String>>, String)>(
			&stringed_result,
		) {
			Ok((mut response_result, result_env_values, console_output)) => {
				// Avoid losing those fields since they are not serialized
				response_result.duration = response.duration.clone();
				response_result.status_code = response.status_code.clone();

				(Some(response_result), result_env_values, console_output)
			}
			Err(error) => (None, env, error.to_string()),
		};

	(response_result, result_env_values, console_output)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::response::ResponseContent;

	// ── Pre-request script tests ─────────────────────────────────

	#[test]
	fn pre_request_noop_script_returns_request_unchanged() {
		let request = Request {
			url: "https://example.com".to_string(),
			..Default::default()
		};
		let script = String::new(); // empty script = no-op

		let (result_request, result_env, console_output) =
			execute_pre_request_script(&script, &request, None);

		let result_request = result_request.expect("should return a request");
		assert_eq!(result_request.url, "https://example.com");
		assert!(result_env.is_none());
		assert_eq!(console_output, "");
	}

	#[test]
	fn pre_request_script_modifies_url() {
		let request = Request {
			url: "https://example.com".to_string(),
			..Default::default()
		};
		let script = String::from("request.url = 'https://modified.com';");

		let (result_request, _, _) = execute_pre_request_script(&script, &request, None);

		let result_request = result_request.expect("should return a request");
		assert_eq!(result_request.url, "https://modified.com");
	}

	#[test]
	fn pre_request_script_captures_console_log() {
		let request = Request::default();
		let script = String::from("console.log('hello from script');");

		let (_, _, console_output) = execute_pre_request_script(&script, &request, None);

		assert!(
			console_output.contains("hello from script"),
			"console_output was: {console_output}"
		);
	}

	#[test]
	fn pre_request_script_with_env_reads_env_values() {
		let request = Request::default();
		let mut env = IndexMap::new();
		env.insert("API_KEY".to_string(), "secret123".to_string());
		let script = String::from("console.log(env.API_KEY);");

		let (_, _, console_output) = execute_pre_request_script(&script, &request, Some(env));

		assert!(
			console_output.contains("secret123"),
			"console_output was: {console_output}"
		);
	}

	#[test]
	fn pre_request_script_modifies_env() {
		let request = Request::default();
		let mut env = IndexMap::new();
		env.insert("KEY".to_string(), "old_value".to_string());
		let script = String::from("env.KEY = 'new_value';");

		let (_, result_env, _) = execute_pre_request_script(&script, &request, Some(env));

		let result_env = result_env.expect("should return env");
		assert_eq!(result_env.get("KEY").unwrap(), "new_value");
	}

	#[test]
	fn pre_request_script_without_env_gets_undefined() {
		let request = Request::default();
		let script = String::from("console.log(typeof env);");

		let (_, result_env, console_output) = execute_pre_request_script(&script, &request, None);

		assert!(
			console_output.contains("undefined"),
			"console_output was: {console_output}"
		);
		assert!(result_env.is_none());
	}

	#[test]
	fn pre_request_script_syntax_error_returns_error_string() {
		let request = Request::default();
		let script = String::from("this is not valid javascript {{{");

		let (result_request, _, console_output) =
			execute_pre_request_script(&script, &request, None);

		assert!(result_request.is_none());
		assert!(!console_output.is_empty(), "should contain error message");
	}

	#[test]
	fn pre_request_pretty_print_formats_json() {
		let request = Request::default();
		let script = String::from(r#"pretty_print({key: "value"});"#);

		let (_, _, console_output) = execute_pre_request_script(&script, &request, None);

		assert!(
			console_output.contains("\"key\""),
			"console_output was: {console_output}"
		);
		assert!(
			console_output.contains("\"value\""),
			"console_output was: {console_output}"
		);
	}

	#[test]
	fn pre_request_script_multiple_console_logs() {
		let request = Request::default();
		let script = String::from("console.log('first'); console.log('second');");

		let (_, _, console_output) = execute_pre_request_script(&script, &request, None);

		assert!(
			console_output.contains("first"),
			"console_output was: {console_output}"
		);
		assert!(
			console_output.contains("second"),
			"console_output was: {console_output}"
		);
	}

	// ── Post-request script tests ────────────────────────────────

	#[test]
	fn post_request_noop_script_returns_response_unchanged() {
		let response = RequestResponse {
			duration: Some("100ms".to_string()),
			status_code: Some("200 OK".to_string()),
			content: Some(ResponseContent::Body("hello".to_string())),
			cookies: None,
			headers: vec![],
		};
		let script = String::new();

		let (result_response, result_env, console_output) =
			execute_post_request_script(&script, &response, None);

		let result_response = result_response.expect("should return a response");
		assert_eq!(result_response.duration, Some("100ms".to_string()));
		assert_eq!(result_response.status_code, Some("200 OK".to_string()));
		assert!(result_env.is_none());
		assert_eq!(console_output, "");
	}

	#[test]
	fn post_request_script_preserves_duration_and_status_code() {
		let response = RequestResponse {
			duration: Some("250ms".to_string()),
			status_code: Some("404 Not Found".to_string()),
			content: Some(ResponseContent::Body("not found".to_string())),
			cookies: None,
			headers: vec![],
		};
		// The script could try to modify these, but the code re-assigns them
		let script =
			String::from("response.duration = 'tampered'; response.status_code = 'tampered';");

		let (result_response, _, _) = execute_post_request_script(&script, &response, None);

		let result_response = result_response.expect("should return a response");
		// duration and status_code are re-assigned from original after deserialization
		assert_eq!(result_response.duration, Some("250ms".to_string()));
		assert_eq!(
			result_response.status_code,
			Some("404 Not Found".to_string())
		);
	}

	#[test]
	fn post_request_script_captures_console_log() {
		let response = RequestResponse::default();
		let script = String::from("console.log('post-script output');");

		let (_, _, console_output) = execute_post_request_script(&script, &response, None);

		assert!(
			console_output.contains("post-script output"),
			"console_output was: {console_output}"
		);
	}

	#[test]
	fn post_request_script_with_env_modifies_env() {
		let response = RequestResponse::default();
		let mut env = IndexMap::new();
		env.insert("TOKEN".to_string(), "old".to_string());
		let script = String::from("env.TOKEN = 'refreshed';");

		let (_, result_env, _) = execute_post_request_script(&script, &response, Some(env));

		let result_env = result_env.expect("should return env");
		assert_eq!(result_env.get("TOKEN").unwrap(), "refreshed");
	}

	#[test]
	fn post_request_script_syntax_error_returns_error() {
		let response = RequestResponse::default();
		let script = String::from("function {broken");

		let (result_response, _, console_output) =
			execute_post_request_script(&script, &response, None);

		assert!(result_response.is_none());
		assert!(!console_output.is_empty(), "should contain error message");
	}

	#[test]
	fn post_request_script_reads_response_content() {
		let response = RequestResponse {
			content: Some(ResponseContent::Body(r#"{"data": "test"}"#.to_string())),
			..Default::default()
		};
		let script = String::from(
			r#"
			let body = JSON.parse(response.content);
			console.log(body.data);
		"#,
		);

		let (_, _, console_output) = execute_post_request_script(&script, &response, None);

		assert!(
			console_output.contains("test"),
			"console_output was: {console_output}"
		);
	}

	#[test]
	fn post_request_script_adds_new_env_key() {
		let response = RequestResponse::default();
		let mut env = IndexMap::new();
		env.insert("EXISTING".to_string(), "val".to_string());
		let script = String::from("env.NEW_KEY = 'new_val';");

		let (_, result_env, _) = execute_post_request_script(&script, &response, Some(env));

		let result_env = result_env.expect("should return env");
		assert_eq!(result_env.get("EXISTING").unwrap(), "val");
		assert_eq!(result_env.get("NEW_KEY").unwrap(), "new_val");
	}
}
