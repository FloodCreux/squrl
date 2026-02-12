use crokey::KeyCombination;
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;
use std::io::Stdout;

use crate::app::app::App;
use crate::tui::events::AppEvent;

impl App<'_> {
	pub(in crate::tui::events) async fn handle_text_input_event(
		&mut self,
		event: &AppEvent,
		key: KeyCombination,
		terminal: &mut Terminal<CrosstermBackend<Stdout>>,
	) {
		match event {
			/* Url */
			AppEvent::ModifyRequestUrl(_) => match self.url_text_input.is_in_default_mode() {
				true => self.tui_modify_request_url(),
				false => self.url_text_input.key_event(key, None),
			},
			AppEvent::CancelEditRequestUrl(_) => match self.url_text_input.is_in_default_mode() {
				true => self.select_request_state(),
				false => self.url_text_input.key_event(key, None),
			},
			AppEvent::KeyEventEditRequestUrl(_) => self.url_text_input.key_event(key, None),

			/* Query params */
			AppEvent::ModifyRequestQueryParam(_) => match self
				.query_params_table
				.selection_text_input
				.is_in_default_mode()
			{
				true => self.tui_modify_request_query_param(),
				false => self
					.query_params_table
					.selection_text_input
					.key_event(key, None),
			},
			AppEvent::CancelEditRequestQueryParam(_) => match self
				.query_params_table
				.selection_text_input
				.is_in_default_mode()
			{
				true => self.select_request_state(),
				false => self
					.query_params_table
					.selection_text_input
					.key_event(key, None),
			},
			AppEvent::KeyEventEditRequestQueryParam(_) => self
				.query_params_table
				.selection_text_input
				.key_event(key, None),

			/* Auth - Basic Username */
			AppEvent::ModifyRequestAuthBasicUsername(_) => {
				match self.auth_basic_username_text_input.is_in_default_mode() {
					true => self.tui_modify_request_auth_basic_username(),
					false => self.auth_basic_username_text_input.key_event(key, None),
				}
			}
			AppEvent::CancelEditRequestAuthBasicUsername(_) => {
				match self.auth_basic_password_text_input.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.auth_basic_password_text_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventEditRequestAuthBasicUsername(_) => {
				self.auth_basic_password_text_input.key_event(key, None)
			}

			/* Auth - Basic Password */
			AppEvent::ModifyRequestAuthBasicPassword(_) => {
				match self.auth_basic_password_text_input.is_in_default_mode() {
					true => self.tui_modify_request_auth_basic_password(),
					false => self.auth_basic_password_text_input.key_event(key, None),
				}
			}
			AppEvent::CancelEditRequestAuthBasicPassword(_) => {
				match self.auth_basic_password_text_input.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.auth_basic_password_text_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventEditRequestAuthBasicPassword(_) => {
				self.auth_digest_nonce_text_input.key_event(key, None)
			}

			/* Auth - Bearer Token */
			AppEvent::ModifyRequestAuthBearerToken(_) => {
				match self.auth_bearer_token_text_input.is_in_default_mode() {
					true => self.tui_modify_request_auth_bearer_token(),
					false => self.auth_bearer_token_text_input.key_event(key, None),
				}
			}
			AppEvent::CancelEditRequestAuthBearerToken(_) => {
				match self.auth_bearer_token_text_input.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.auth_bearer_token_text_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventEditRequestAuthBearerToken(_) => {
				self.auth_bearer_token_text_input.key_event(key, None)
			}

			/* Auth - JWT Secret */
			AppEvent::ModifyRequestAuthJwtSecret(_) => {
				match self.auth_jwt_secret_text_input.is_in_default_mode() {
					true => self.tui_modify_request_auth_jwt_secret(),
					false => self.auth_jwt_secret_text_input.key_event(key, None),
				}
			}
			AppEvent::CancelEditRequestAuthJwtSecret(_) => {
				match self.auth_jwt_secret_text_input.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.auth_jwt_secret_text_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventEditRequestAuthJwtSecret(_) => {
				self.auth_jwt_secret_text_input.key_event(key, None)
			}

			/* Auth - JWT Payload */
			AppEvent::ModifyRequestAuthJwtPayload(_) => {
				match self.auth_jwt_payload_text_area.is_in_default_mode() {
					true => self.tui_modify_request_auth_jwt_payload(),
					false => self
						.auth_jwt_payload_text_area
						.key_event(key, Some(terminal)),
				}
			}
			AppEvent::CancelEditRequestAuthJwtPayload(_) => {
				match self.auth_jwt_payload_text_area.is_in_default_mode() {
					true => self.select_request_state(),
					false => self
						.auth_jwt_payload_text_area
						.key_event(key, Some(terminal)),
				}
			}
			AppEvent::KeyEventEditRequestAuthJwtPayload(_) => self
				.auth_jwt_payload_text_area
				.key_event(key, Some(terminal)),

			/* Auth - Digest Username */
			AppEvent::ModifyRequestAuthDigestUsername(_) => {
				match self.auth_digest_username_text_input.is_in_default_mode() {
					true => self.tui_modify_request_auth_digest_username(),
					false => self.auth_digest_username_text_input.key_event(key, None),
				}
			}
			AppEvent::CancelEditRequestAuthDigestUsername(_) => {
				match self.auth_digest_username_text_input.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.auth_digest_username_text_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventEditRequestAuthDigestUsername(_) => {
				self.auth_digest_username_text_input.key_event(key, None)
			}

			/* Auth - Digest Password */
			AppEvent::ModifyRequestAuthDigestPassword(_) => {
				match self.auth_digest_password_text_input.is_in_default_mode() {
					true => self.tui_modify_request_auth_digest_password(),
					false => self.auth_digest_password_text_input.key_event(key, None),
				}
			}
			AppEvent::CancelEditRequestAuthDigestPassword(_) => {
				match self.auth_digest_password_text_input.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.auth_digest_password_text_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventEditRequestAuthDigestPassword(_) => {
				self.auth_digest_password_text_input.key_event(key, None)
			}

			/* Auth - Digest Domains */
			AppEvent::ModifyRequestAuthDigestDomains(_) => {
				match self.auth_digest_domains_text_input.is_in_default_mode() {
					true => self.tui_modify_request_auth_digest_domains(),
					false => self.auth_digest_domains_text_input.key_event(key, None),
				}
			}
			AppEvent::CancelEditRequestAuthDigestDomains(_) => {
				match self.auth_digest_domains_text_input.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.auth_digest_domains_text_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventEditRequestAuthDigestDomains(_) => {
				self.auth_digest_domains_text_input.key_event(key, None)
			}

			/* Auth - Digest Realm */
			AppEvent::ModifyRequestAuthDigestRealm(_) => {
				match self.auth_digest_realm_text_input.is_in_default_mode() {
					true => self.tui_modify_request_auth_digest_realm(),
					false => self.auth_digest_realm_text_input.key_event(key, None),
				}
			}
			AppEvent::CancelEditRequestAuthDigestRealm(_) => {
				match self.auth_digest_realm_text_input.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.auth_digest_realm_text_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventEditRequestAuthDigestRealm(_) => {
				self.auth_digest_realm_text_input.key_event(key, None)
			}

			/* Auth - Digest Nonce */
			AppEvent::ModifyRequestAuthDigestNonce(_) => {
				match self.auth_digest_nonce_text_input.is_in_default_mode() {
					true => self.tui_modify_request_auth_digest_nonce(),
					false => self.auth_digest_nonce_text_input.key_event(key, None),
				}
			}
			AppEvent::CancelEditRequestAuthDigestNonce(_) => {
				match self.auth_digest_nonce_text_input.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.auth_digest_nonce_text_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventEditRequestAuthDigestNonce(_) => {
				self.auth_digest_nonce_text_input.key_event(key, None)
			}

			/* Auth - Digest Opaque */
			AppEvent::ModifyRequestAuthDigestOpaque(_) => {
				match self.auth_digest_opaque_text_input.is_in_default_mode() {
					true => self.tui_modify_request_auth_digest_opaque(),
					false => self.auth_digest_opaque_text_input.key_event(key, None),
				}
			}
			AppEvent::CancelEditRequestAuthDigestOpaque(_) => {
				match self.auth_digest_opaque_text_input.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.auth_digest_opaque_text_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventEditRequestAuthDigestOpaque(_) => {
				self.auth_digest_opaque_text_input.key_event(key, None)
			}

			/* Header */
			AppEvent::ModifyRequestHeader(_) => {
				match self.headers_table.selection_text_input.is_in_default_mode() {
					true => self.tui_modify_request_header(),
					false => self.headers_table.selection_text_input.key_event(key, None),
				}
			}
			AppEvent::CancelEditRequestHeader(_) => {
				match self.headers_table.selection_text_input.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.headers_table.selection_text_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventEditRequestHeader(_) => {
				self.headers_table.selection_text_input.key_event(key, None)
			}

			/* Body - Table */
			AppEvent::ModifyRequestBodyTable(_) => match self
				.body_form_table
				.selection_text_input
				.is_in_default_mode()
			{
				true => self.tui_modify_request_form_data(),
				false => self
					.body_form_table
					.selection_text_input
					.key_event(key, None),
			},
			AppEvent::CancelEditRequestBodyTable(_) => match self
				.body_form_table
				.selection_text_input
				.is_in_default_mode()
			{
				true => self.select_request_state(),
				false => self
					.body_form_table
					.selection_text_input
					.key_event(key, None),
			},
			AppEvent::KeyEventEditRequestBodyTable(_) => self
				.body_form_table
				.selection_text_input
				.key_event(key, None),

			/* Body - File */
			AppEvent::ModifyRequestBodyFile(_) => {
				match self.body_file_text_input.is_in_default_mode() {
					true => self.tui_modify_request_body(),
					false => self.body_file_text_input.key_event(key, None),
				}
			}
			AppEvent::CancelEditRequestBodyFile(_) => {
				match self.body_file_text_input.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.body_file_text_input.key_event(key, None),
				}
			}
			AppEvent::KeyEventEditRequestBodyFile(_) => {
				self.body_file_text_input.key_event(key, None)
			}

			/* Body - String */
			AppEvent::ModifyRequestBodyString(_) => {
				match self.body_text_area.is_in_default_mode() {
					true => self.tui_modify_request_body(),
					false => self.body_text_area.key_event(key, Some(terminal)),
				}
			}
			AppEvent::CancelEditRequestBodyString(_) => {
				match self.body_text_area.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.body_text_area.key_event(key, Some(terminal)),
				}
			}
			AppEvent::KeyEventEditRequestBodyString(_) => {
				self.body_text_area.key_event(key, Some(terminal))
			}

			/* Websocket Message */
			AppEvent::ModifyRequestMessage(_) => {
				match self.message_text_area.is_in_default_mode() {
					true => self.tui_send_request_message().await,
					false => self.message_text_area.key_event(key, Some(terminal)),
				}
			}
			AppEvent::CancelEditRequestMessage(_) => {
				match self.message_text_area.is_in_default_mode() {
					true => self.select_request_state(),
					false => self.message_text_area.key_event(key, Some(terminal)),
				}
			}
			AppEvent::KeyEventEditRequestMessage(_) => {
				self.message_text_area.key_event(key, Some(terminal))
			}

			/* Scripts - Pre-request */
			AppEvent::ModifyRequestPreRequestScript(_) => match self
				.script_console
				.pre_request_text_area
				.is_in_default_mode()
			{
				true => self.tui_modify_pre_request_script(),
				false => self
					.script_console
					.pre_request_text_area
					.key_event(key, Some(terminal)),
			},
			AppEvent::CancelEditRequestPreRequestScript(_) => match self
				.script_console
				.pre_request_text_area
				.is_in_default_mode()
			{
				true => self.select_request_state(),
				false => self
					.script_console
					.pre_request_text_area
					.key_event(key, Some(terminal)),
			},
			AppEvent::KeyEventEditRequestPreRequestScript(_) => self
				.script_console
				.pre_request_text_area
				.key_event(key, Some(terminal)),

			/* Scripts - Post-request */
			AppEvent::ModifyRequestPostRequestScript(_) => match self
				.script_console
				.post_request_text_area
				.is_in_default_mode()
			{
				true => self.tui_modify_post_request_script(),
				false => self
					.script_console
					.post_request_text_area
					.key_event(key, None),
			},
			AppEvent::CancelEditRequestPostRequestScript(_) => match self
				.script_console
				.post_request_text_area
				.is_in_default_mode()
			{
				true => self.select_request_state(),
				false => self
					.script_console
					.post_request_text_area
					.key_event(key, None),
			},
			AppEvent::KeyEventEditRequestPostRequestScript(_) => self
				.script_console
				.post_request_text_area
				.key_event(key, None),

			_ => unreachable!("handle_text_input_event called with non-text-input event"),
		}
	}
}
