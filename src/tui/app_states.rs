use crokey::{KeyCombination, key};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::Span;
use ratatui::style::{Color, Stylize};
use ratatui::text::Line;
use strum::Display;

use crate::app::app::App;
use crate::app::files::key_bindings::{CustomTextArea, KEY_BINDINGS, TextAreaMode};
use crate::app::files::theme::THEME;
use crate::models::protocol::protocol::Protocol;
use crate::tui::app_states::AppState::*;
// use crate::tui::event_key_bindings::EventKeyBinding;
// use crate::tui::events::AppEvent;
// use crate::tui::events::AppEvent::*;
// use crate::tui::ui::param_tabs::param_tabs::RequestParamsTabs;
// use crate::tui::ui::views::RequestView;

#[derive(Copy, Clone, PartialEq, Default, Display)]
pub enum AppState {
	#[default]
	#[strum(to_string = "Main menu")]
	Normal,

	/* Env */
	#[strum(to_string = "Displaying environment editor")]
	// DisplayingEnvEditor,
	#[strum(to_string = "Editing env variable")]
	EditingEnvVariable,

	/* Cookies */
	#[strum(to_string = "Displaying cookies")]
	DisplayingCookies,

	#[strum(to_string = "Editing cookies")]
	#[allow(dead_code)]
	EditingCookies,

	/* Logs */
	#[strum(to_string = "Displaying logs")]
	DisplayingLogs,

	/* Collections */
	#[strum(to_string = "Choosing an element to create")]
	ChoosingElementToCreate,

	#[strum(to_string = "Creating new collection")]
	CreatingNewCollection,

	#[strum(to_string = "Creating new request")]
	CreatingNewRequest,

	#[strum(to_string = "Deleting collection")]
	DeletingCollection,

	#[strum(to_string = "Deleting request")]
	DeletingRequest,

	#[strum(to_string = "Renaming collection")]
	RenamingCollection,

	#[strum(to_string = "Renaming request")]
	RenamingRequest,

	/* Request */
	#[strum(to_string = "Request menu")]
	SelectedRequest,

	#[strum(to_string = "Editing request URL")]
	EditingRequestUrl,

	#[strum(to_string = "Editing request param")]
	EditingRequestParam,

	#[strum(to_string = "Editing request auth username")]
	EditingRequestAuthBasicUsername,

	#[strum(to_string = "Editing request auth password")]
	EditingRequestAuthBasicPassword,

	#[strum(to_string = "Editing request auth bearer token")]
	EditingRequestAuthBearerToken,

	#[strum(to_string = "Editing request JWT secret")]
	EditingRequestAuthJwtSecret,

	#[strum(to_string = "Editing request JWT payload")]
	EditingRequestAuthJwtPayload,

	#[strum(to_string = "Editing request digest username")]
	EditingRequestAuthDigestUsername,

	#[strum(to_string = "Editing request digest password")]
	EditingRequestAuthDigestPassword,

	#[strum(to_string = "Editing request digest domains")]
	EditingRequestAuthDigestDomains,

	#[strum(to_string = "Editing request digest realm")]
	EditingRequestAuthDigestRealm,

	#[strum(to_string = "Editing request digest nonce")]
	EditingRequestAuthDigestNonce,

	#[strum(to_string = "Editing request digest opaque")]
	EditingRequestAuthDigestOpaque,

	#[strum(to_string = "Editing request header")]
	EditingRequestHeader,

	#[strum(to_string = "Editing request body (Form)")]
	EditingRequestBodyTable,

	#[strum(to_string = "Editing request body (File)")]
	EditingRequestBodyFile,

	#[strum(to_string = "Editing request body (Text)")]
	EditingRequestBodyString,

	#[strum(to_string = "Editing request message")]
	EditingRequestMessage,

	#[strum(to_string = "Editing pre-request script")]
	EditingPreRequestScript,

	#[strum(to_string = "Editing post-request script")]
	EditingPostRequestScript,

	#[strum(to_string = "Editing request settings")]
	EditingRequestSettings,

	#[strum(to_string = "Choosing request export format")]
	ChoosingRequestExportFormat,

	#[strum(to_string = "Displaying request export")]
	DisplayingRequestExport,
}

lazy_static! {
	pub static ref AVAILABLE_EVENTS: RwLock<Vec<AppEvent>> = RwLock::new(vec![]);
	pub static ref EMPTY_KEY: KeyCombination =
		KeyCombination::new(KeyCode::Null, KeyModifiers::NONE);
}

impl App<'_> {
	pub fn update_current_available_events(&mut self) {
		let is_there_any_env = match self.get_selected_env_as_local() {
			None => false,
			Some(_) => true,
		};

		let protocol = match &self.collections_tree.selected {
			Some(selected_request_index) => {
				let local_selected_request = self.collections[selected_request_index.0].requests
					[selected_request_index.1]
					.clone();
				let selected_request = local_selected_request.read();
				Some(selected_request.protocol.clone())
			}
			None => None,
		};

		*AVAILABLE_EVENTS.write() = self.state.get_available_events(
			self.request_view,
			self.request_param_tab,
			protocol,
			is_there_any_env,
		);
	}
}
