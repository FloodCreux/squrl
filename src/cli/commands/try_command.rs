use crate::cli::commands::request_commands::new::NewRequestCommand;
use crate::cli::commands::request_commands::send::SendCommand;

#[derive(clap::Args, Debug, Clone)]
pub struct TryCommand {
	#[command(flatten)]
	pub new_request_command: NewRequestCommand,

	#[command(flatten)]
	pub send_command: SendCommand,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::cli::commands::request_commands::new::NewRequestCommand;
	use crate::cli::commands::request_commands::send::SendCommand;
	use clap::Parser;

	#[derive(Parser)]
	struct NewRequestCli {
		#[command(flatten)]
		cmd: NewRequestCommand,
	}

	#[derive(Parser)]
	struct SendCli {
		#[command(flatten)]
		cmd: SendCommand,
	}

	#[derive(Parser)]
	struct TryCli {
		#[command(flatten)]
		cmd: TryCommand,
	}

	// === NewRequestCommand defaults ===

	#[test]
	fn new_request_defaults_url_to_empty() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert_eq!(cli.cmd.url, "");
	}

	#[test]
	fn new_request_defaults_protocol_to_http() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert_eq!(cli.cmd.protocol.to_string(), "HTTP");
	}

	#[test]
	fn new_request_defaults_method_to_get() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert_eq!(cli.cmd.method.to_string(), "GET");
	}

	#[test]
	fn new_request_defaults_timeout_to_30000() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert_eq!(cli.cmd.timeout, 30000);
	}

	#[test]
	fn new_request_defaults_no_cookies_to_false() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.no_cookies);
	}

	#[test]
	fn new_request_defaults_no_pretty_to_false() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.no_pretty);
	}

	#[test]
	fn new_request_defaults_accept_invalid_certs_to_false() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.accept_invalid_certs);
	}

	#[test]
	fn new_request_defaults_accept_invalid_hostnames_to_false() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.accept_invalid_hostnames);
	}

	#[test]
	fn new_request_defaults_no_redirects_to_false() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.no_redirects);
	}

	#[test]
	fn new_request_defaults_no_proxy_to_false() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.no_proxy);
	}

	#[test]
	fn new_request_defaults_no_base_headers_to_false() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.no_base_headers);
	}

	#[test]
	fn new_request_defaults_pre_request_script_to_none() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(cli.cmd.pre_request_script.is_none());
	}

	#[test]
	fn new_request_defaults_post_request_script_to_none() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(cli.cmd.post_request_script.is_none());
	}

	#[test]
	fn new_request_defaults_empty_params() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(cli.cmd.add_param.is_empty());
	}

	#[test]
	fn new_request_defaults_empty_headers() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(cli.cmd.add_header.is_empty());
	}

	// === NewRequestCommand parsing ===

	#[test]
	fn new_request_parses_protocol_http() {
		let cli = NewRequestCli::try_parse_from(["test", "--protocol", "HTTP"]).unwrap();
		assert_eq!(cli.cmd.protocol.to_string(), "HTTP");
	}

	#[test]
	fn new_request_parses_url() {
		let cli = NewRequestCli::try_parse_from(["test", "--url", "http://example.com"]).unwrap();
		assert_eq!(cli.cmd.url, "http://example.com");
	}

	#[test]
	fn new_request_parses_url_short_flag() {
		let cli = NewRequestCli::try_parse_from(["test", "-u", "http://example.com"]).unwrap();
		assert_eq!(cli.cmd.url, "http://example.com");
	}

	#[test]
	fn new_request_parses_method_post() {
		let cli = NewRequestCli::try_parse_from(["test", "--method", "POST"]).unwrap();
		assert_eq!(cli.cmd.method.to_string(), "POST");
	}

	#[test]
	fn new_request_parses_method_put() {
		let cli = NewRequestCli::try_parse_from(["test", "--method", "PUT"]).unwrap();
		assert_eq!(cli.cmd.method.to_string(), "PUT");
	}

	#[test]
	fn new_request_parses_method_patch() {
		let cli = NewRequestCli::try_parse_from(["test", "--method", "PATCH"]).unwrap();
		assert_eq!(cli.cmd.method.to_string(), "PATCH");
	}

	#[test]
	fn new_request_parses_method_delete() {
		let cli = NewRequestCli::try_parse_from(["test", "--method", "DELETE"]).unwrap();
		assert_eq!(cli.cmd.method.to_string(), "DELETE");
	}

	#[test]
	fn new_request_parses_method_options() {
		let cli = NewRequestCli::try_parse_from(["test", "--method", "OPTIONS"]).unwrap();
		assert_eq!(cli.cmd.method.to_string(), "OPTIONS");
	}

	#[test]
	fn new_request_parses_method_head() {
		let cli = NewRequestCli::try_parse_from(["test", "--method", "HEAD"]).unwrap();
		assert_eq!(cli.cmd.method.to_string(), "HEAD");
	}

	#[test]
	fn new_request_parses_method_short_flag() {
		let cli = NewRequestCli::try_parse_from(["test", "-m", "DELETE"]).unwrap();
		assert_eq!(cli.cmd.method.to_string(), "DELETE");
	}

	#[test]
	fn new_request_parses_custom_timeout() {
		let cli = NewRequestCli::try_parse_from(["test", "--timeout", "5000"]).unwrap();
		assert_eq!(cli.cmd.timeout, 5000);
	}

	#[test]
	fn new_request_no_cookies_flag() {
		let cli = NewRequestCli::try_parse_from(["test", "--no-cookies"]).unwrap();
		assert!(cli.cmd.no_cookies);
	}

	#[test]
	fn new_request_no_pretty_flag() {
		let cli = NewRequestCli::try_parse_from(["test", "--no-pretty"]).unwrap();
		assert!(cli.cmd.no_pretty);
	}

	#[test]
	fn new_request_accept_invalid_certs_flag() {
		let cli = NewRequestCli::try_parse_from(["test", "--accept-invalid-certs"]).unwrap();
		assert!(cli.cmd.accept_invalid_certs);
	}

	#[test]
	fn new_request_no_redirects_flag() {
		let cli = NewRequestCli::try_parse_from(["test", "--no-redirects"]).unwrap();
		assert!(cli.cmd.no_redirects);
	}

	#[test]
	fn new_request_parses_single_query_param() {
		let cli = NewRequestCli::try_parse_from(["test", "--add-param", "key", "value"]).unwrap();
		assert_eq!(cli.cmd.add_param, vec!["key", "value"]);
	}

	#[test]
	fn new_request_parses_multiple_query_params() {
		let cli = NewRequestCli::try_parse_from([
			"test",
			"--add-param",
			"k1",
			"v1",
			"--add-param",
			"k2",
			"v2",
		])
		.unwrap();
		assert_eq!(cli.cmd.add_param, vec!["k1", "v1", "k2", "v2"]);
	}

	#[test]
	fn new_request_parses_single_header() {
		let cli = NewRequestCli::try_parse_from([
			"test",
			"--add-header",
			"Content-Type",
			"application/json",
		])
		.unwrap();
		assert_eq!(cli.cmd.add_header, vec!["Content-Type", "application/json"]);
	}

	#[test]
	fn new_request_parses_multiple_headers() {
		let cli = NewRequestCli::try_parse_from([
			"test",
			"--add-header",
			"Accept",
			"text/html",
			"--add-header",
			"X-Custom",
			"value",
		])
		.unwrap();
		assert_eq!(
			cli.cmd.add_header,
			vec!["Accept", "text/html", "X-Custom", "value"]
		);
	}

	#[test]
	fn new_request_parses_pre_request_script() {
		let cli =
			NewRequestCli::try_parse_from(["test", "--pre-request-script", "echo hello"]).unwrap();
		assert_eq!(cli.cmd.pre_request_script.unwrap(), "echo hello");
	}

	#[test]
	fn new_request_parses_post_request_script() {
		let cli =
			NewRequestCli::try_parse_from(["test", "--post-request-script", "echo done"]).unwrap();
		assert_eq!(cli.cmd.post_request_script.unwrap(), "echo done");
	}

	// === AuthArgs ===

	#[test]
	fn new_request_parses_basic_auth() {
		let cli = NewRequestCli::try_parse_from(["test", "--auth-basic", "user", "pass"]).unwrap();
		assert_eq!(cli.cmd.auth.auth_basic, vec!["user", "pass"]);
	}

	#[test]
	fn new_request_parses_bearer_token() {
		let cli =
			NewRequestCli::try_parse_from(["test", "--auth-bearer-token", "mytoken123"]).unwrap();
		assert_eq!(cli.cmd.auth.auth_bearer_token, vec!["mytoken123"]);
	}

	#[test]
	fn new_request_parses_jwt_token() {
		let cli = NewRequestCli::try_parse_from([
			"test",
			"--auth-jwt-token",
			"HS256",
			"plain",
			"mysecret",
			r#"{"sub":"1234"}"#,
		])
		.unwrap();
		assert_eq!(
			cli.cmd.auth.auth_jwt_token,
			vec!["HS256", "plain", "mysecret", r#"{"sub":"1234"}"#]
		);
	}

	#[test]
	fn new_request_parses_digest_auth() {
		let cli = NewRequestCli::try_parse_from([
			"test",
			"--auth-digest",
			"admin",
			"secret",
			"Digest realm=\"test\"",
		])
		.unwrap();
		assert_eq!(
			cli.cmd.auth.auth_digest,
			vec!["admin", "secret", "Digest realm=\"test\""]
		);
	}

	#[test]
	fn new_request_auth_types_are_mutually_exclusive() {
		let result = NewRequestCli::try_parse_from([
			"test",
			"--auth-basic",
			"user",
			"pass",
			"--auth-bearer-token",
			"token",
		]);
		assert!(result.is_err());
	}

	#[test]
	fn new_request_basic_and_jwt_are_mutually_exclusive() {
		let result = NewRequestCli::try_parse_from([
			"test",
			"--auth-basic",
			"user",
			"pass",
			"--auth-jwt-token",
			"HS256",
			"plain",
			"secret",
			"{}",
		]);
		assert!(result.is_err());
	}

	#[test]
	fn new_request_defaults_empty_auth_args() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(cli.cmd.auth.auth_basic.is_empty());
		assert!(cli.cmd.auth.auth_bearer_token.is_empty());
		assert!(cli.cmd.auth.auth_jwt_token.is_empty());
		assert!(cli.cmd.auth.auth_digest.is_empty());
	}

	// === BodyArgs ===

	#[test]
	fn new_request_parses_body_json() {
		let cli =
			NewRequestCli::try_parse_from(["test", "--body-json", r#"{"key":"value"}"#]).unwrap();
		assert_eq!(cli.cmd.body.body_json.unwrap(), r#"{"key":"value"}"#);
	}

	#[test]
	fn new_request_parses_body_raw() {
		let cli = NewRequestCli::try_parse_from(["test", "--body-raw", "some raw text"]).unwrap();
		assert_eq!(cli.cmd.body.body_raw.unwrap(), "some raw text");
	}

	#[test]
	fn new_request_parses_body_xml() {
		let cli = NewRequestCli::try_parse_from(["test", "--body-xml", "<root/>"]).unwrap();
		assert_eq!(cli.cmd.body.body_xml.unwrap(), "<root/>");
	}

	#[test]
	fn new_request_parses_body_html() {
		let cli = NewRequestCli::try_parse_from(["test", "--body-html", "<p>hello</p>"]).unwrap();
		assert_eq!(cli.cmd.body.body_html.unwrap(), "<p>hello</p>");
	}

	#[test]
	fn new_request_parses_body_javascript() {
		let cli = NewRequestCli::try_parse_from(["test", "--body-javascript", "console.log('hi')"])
			.unwrap();
		assert_eq!(cli.cmd.body.body_javascript.unwrap(), "console.log('hi')");
	}

	#[test]
	fn new_request_parses_body_form() {
		let cli =
			NewRequestCli::try_parse_from(["test", "--add-body-form", "username", "john"]).unwrap();
		assert_eq!(cli.cmd.body.add_body_form, vec!["username", "john"]);
	}

	#[test]
	fn new_request_parses_multiple_body_form_fields() {
		let cli = NewRequestCli::try_parse_from([
			"test",
			"--add-body-form",
			"user",
			"john",
			"--add-body-form",
			"age",
			"30",
		])
		.unwrap();
		assert_eq!(
			cli.cmd.body.add_body_form,
			vec!["user", "john", "age", "30"]
		);
	}

	#[test]
	fn new_request_parses_body_multipart() {
		let cli = NewRequestCli::try_parse_from(["test", "--add-body-multipart", "field", "data"])
			.unwrap();
		assert_eq!(cli.cmd.body.add_body_multipart, vec!["field", "data"]);
	}

	#[test]
	fn new_request_parses_body_file() {
		let cli =
			NewRequestCli::try_parse_from(["test", "--body-file", "/path/to/file.txt"]).unwrap();
		assert_eq!(cli.cmd.body.body_file.unwrap(), "/path/to/file.txt");
	}

	#[test]
	fn new_request_body_types_are_mutually_exclusive() {
		let result =
			NewRequestCli::try_parse_from(["test", "--body-json", "{}", "--body-raw", "text"]);
		assert!(result.is_err());
	}

	#[test]
	fn new_request_body_json_and_xml_are_mutually_exclusive() {
		let result =
			NewRequestCli::try_parse_from(["test", "--body-json", "{}", "--body-xml", "<root/>"]);
		assert!(result.is_err());
	}

	#[test]
	fn new_request_defaults_empty_body_args() {
		let cli = NewRequestCli::try_parse_from(["test"]).unwrap();
		assert!(cli.cmd.body.body_file.is_none());
		assert!(cli.cmd.body.add_body_multipart.is_empty());
		assert!(cli.cmd.body.add_body_form.is_empty());
		assert!(cli.cmd.body.body_raw.is_none());
		assert!(cli.cmd.body.body_json.is_none());
		assert!(cli.cmd.body.body_xml.is_none());
		assert!(cli.cmd.body.body_html.is_none());
		assert!(cli.cmd.body.body_javascript.is_none());
	}

	// === SendCommand defaults ===

	#[test]
	fn send_command_defaults_hide_content_to_false() {
		let cli = SendCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.hide_content);
	}

	#[test]
	fn send_command_defaults_status_code_to_false() {
		let cli = SendCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.status_code);
	}

	#[test]
	fn send_command_defaults_duration_to_false() {
		let cli = SendCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.duration);
	}

	#[test]
	fn send_command_defaults_headers_to_false() {
		let cli = SendCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.headers);
	}

	#[test]
	fn send_command_defaults_cookies_to_false() {
		let cli = SendCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.cookies);
	}

	#[test]
	fn send_command_defaults_console_to_false() {
		let cli = SendCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.console);
	}

	#[test]
	fn send_command_defaults_request_name_to_false() {
		let cli = SendCli::try_parse_from(["test"]).unwrap();
		assert!(!cli.cmd.request_name);
	}

	#[test]
	fn send_command_defaults_env_to_none() {
		let cli = SendCli::try_parse_from(["test"]).unwrap();
		assert!(cli.cmd.env.is_none());
	}

	// === SendCommand parsing ===

	#[test]
	fn send_command_hide_content_flag() {
		let cli = SendCli::try_parse_from(["test", "--hide-content"]).unwrap();
		assert!(cli.cmd.hide_content);
	}

	#[test]
	fn send_command_status_code_flag() {
		let cli = SendCli::try_parse_from(["test", "--status-code"]).unwrap();
		assert!(cli.cmd.status_code);
	}

	#[test]
	fn send_command_duration_flag() {
		let cli = SendCli::try_parse_from(["test", "--duration"]).unwrap();
		assert!(cli.cmd.duration);
	}

	#[test]
	fn send_command_headers_flag() {
		let cli = SendCli::try_parse_from(["test", "--headers"]).unwrap();
		assert!(cli.cmd.headers);
	}

	#[test]
	fn send_command_cookies_flag() {
		let cli = SendCli::try_parse_from(["test", "--cookies"]).unwrap();
		assert!(cli.cmd.cookies);
	}

	#[test]
	fn send_command_console_flag() {
		let cli = SendCli::try_parse_from(["test", "--console"]).unwrap();
		assert!(cli.cmd.console);
	}

	#[test]
	fn send_command_request_name_flag() {
		let cli = SendCli::try_parse_from(["test", "--request-name"]).unwrap();
		assert!(cli.cmd.request_name);
	}

	#[test]
	fn send_command_parses_env() {
		let cli = SendCli::try_parse_from(["test", "--env", "production"]).unwrap();
		assert_eq!(cli.cmd.env.unwrap(), "production");
	}

	#[test]
	fn send_command_all_flags_together() {
		let cli = SendCli::try_parse_from([
			"test",
			"--hide-content",
			"--status-code",
			"--duration",
			"--headers",
			"--cookies",
			"--console",
			"--request-name",
			"--env",
			"staging",
		])
		.unwrap();
		assert!(cli.cmd.hide_content);
		assert!(cli.cmd.status_code);
		assert!(cli.cmd.duration);
		assert!(cli.cmd.headers);
		assert!(cli.cmd.cookies);
		assert!(cli.cmd.console);
		assert!(cli.cmd.request_name);
		assert_eq!(cli.cmd.env.unwrap(), "staging");
	}

	// === TryCommand (combines NewRequestCommand + SendCommand) ===

	#[test]
	fn try_command_parses_url_and_send_flags() {
		let cli = TryCli::try_parse_from([
			"test",
			"--url",
			"http://example.com",
			"--method",
			"POST",
			"--status-code",
			"--duration",
		])
		.unwrap();
		assert_eq!(cli.cmd.new_request_command.url, "http://example.com");
		assert_eq!(cli.cmd.new_request_command.method.to_string(), "POST");
		assert!(cli.cmd.send_command.status_code);
		assert!(cli.cmd.send_command.duration);
	}

	#[test]
	fn try_command_with_all_send_flags() {
		let cli = TryCli::try_parse_from([
			"test",
			"--hide-content",
			"--status-code",
			"--duration",
			"--headers",
			"--cookies",
			"--console",
			"--request-name",
		])
		.unwrap();
		assert!(cli.cmd.send_command.hide_content);
		assert!(cli.cmd.send_command.status_code);
		assert!(cli.cmd.send_command.duration);
		assert!(cli.cmd.send_command.headers);
		assert!(cli.cmd.send_command.cookies);
		assert!(cli.cmd.send_command.console);
		assert!(cli.cmd.send_command.request_name);
	}

	#[test]
	fn try_command_with_auth_and_body() {
		let cli = TryCli::try_parse_from([
			"test",
			"--url",
			"http://api.example.com/data",
			"--method",
			"POST",
			"--auth-bearer-token",
			"tok_abc",
			"--body-json",
			r#"{"name":"test"}"#,
			"--status-code",
		])
		.unwrap();
		assert_eq!(
			cli.cmd.new_request_command.url,
			"http://api.example.com/data"
		);
		assert_eq!(cli.cmd.new_request_command.method.to_string(), "POST");
		assert_eq!(
			cli.cmd.new_request_command.auth.auth_bearer_token,
			vec!["tok_abc"]
		);
		assert_eq!(
			cli.cmd.new_request_command.body.body_json.as_deref(),
			Some(r#"{"name":"test"}"#)
		);
		assert!(cli.cmd.send_command.status_code);
	}

	#[test]
	fn try_command_with_headers_and_params() {
		let cli = TryCli::try_parse_from([
			"test",
			"--url",
			"http://example.com",
			"--add-header",
			"Authorization",
			"Bearer xyz",
			"--add-param",
			"page",
			"1",
			"--headers",
		])
		.unwrap();
		assert_eq!(
			cli.cmd.new_request_command.add_header,
			vec!["Authorization", "Bearer xyz"]
		);
		assert_eq!(cli.cmd.new_request_command.add_param, vec!["page", "1"]);
		assert!(cli.cmd.send_command.headers);
	}

	#[test]
	fn try_command_with_env() {
		let cli = TryCli::try_parse_from(["test", "--url", "http://example.com", "--env", "local"])
			.unwrap();
		assert_eq!(cli.cmd.send_command.env.as_deref(), Some("local"));
	}

	#[test]
	fn try_command_defaults_match_subcommand_defaults() {
		let cli = TryCli::try_parse_from(["test"]).unwrap();
		assert_eq!(cli.cmd.new_request_command.url, "");
		assert_eq!(cli.cmd.new_request_command.protocol.to_string(), "HTTP");
		assert_eq!(cli.cmd.new_request_command.method.to_string(), "GET");
		assert_eq!(cli.cmd.new_request_command.timeout, 30000);
		assert!(!cli.cmd.new_request_command.no_cookies);
		assert!(!cli.cmd.new_request_command.no_pretty);
		assert!(!cli.cmd.send_command.hide_content);
		assert!(!cli.cmd.send_command.status_code);
		assert!(cli.cmd.send_command.env.is_none());
	}

	// === Invalid input handling ===

	#[test]
	fn new_request_rejects_invalid_method() {
		let result = NewRequestCli::try_parse_from(["test", "--method", "INVALID"]);
		assert!(result.is_err());
	}

	#[test]
	fn new_request_rejects_missing_param_value() {
		let result = NewRequestCli::try_parse_from(["test", "--add-param", "key"]);
		assert!(result.is_err());
	}

	#[test]
	fn new_request_rejects_missing_header_value() {
		let result = NewRequestCli::try_parse_from(["test", "--add-header", "key"]);
		assert!(result.is_err());
	}
}
