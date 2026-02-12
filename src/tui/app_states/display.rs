use crokey::KeyCombination;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::Span;
use ratatui::style::{Color, Stylize};
use ratatui::text::Line;

use crate::app::app::App;
use crate::app::files::theme::THEME;
use crate::tui::app_states::AppState::*;
use crate::tui::events::AppEvent;
use crate::tui::events::AppEvent::*;

pub fn event_available_keys_to_spans(
	events: &[AppEvent],
	fg_color: Color,
	bg_color: Color,
	short_only: bool,
) -> Vec<Vec<Span<'_>>> {
	let mut spans: Vec<Vec<Span>> = vec![];

	for event in events.iter() {
		let is_documentation = matches!(event, Documentation(_));

		let event_key_bindings = event.get_event_key_bindings();

		if let Some(key_spans) =
			event_key_bindings.to_spans(fg_color, bg_color, short_only, is_documentation)
		{
			spans.push(key_spans);
		}
	}

	spans.last_mut().unwrap().pop();

	spans
}

lazy_static! {
	pub static ref AVAILABLE_EVENTS: RwLock<Vec<AppEvent>> = RwLock::new(vec![]);
	pub static ref EMPTY_KEY: KeyCombination =
		KeyCombination::new(KeyCode::Null, KeyModifiers::NONE);
}

impl App<'_> {
	pub fn update_current_available_events(&mut self) {
		let is_there_any_env = self.get_selected_env_as_local().is_some();

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

	pub fn get_state_line(&self) -> Line<'_> {
		match self.state {
			Normal
			| ChoosingElementToCreate
			| CreatingNewCollection
			| CreatingNewRequest
			| DisplayingCookies
			| DisplayingLogs => Line::from(self.state.to_string())
				.fg(THEME.read().ui.font_color)
				.bg(THEME.read().ui.main_background_color),

			DeletingCollection | RenamingCollection => {
				let collection_index = self.collections_tree.state.selected()[0];
				let collection_name = &self.collections[collection_index].name;

				Line::from(vec![
					Span::raw("Collection > ").fg(THEME.read().ui.secondary_foreground_color),
					Span::raw(format!("{} > ", collection_name))
						.fg(THEME.read().ui.secondary_foreground_color),
					Span::raw(self.state.to_string())
						.fg(THEME.read().ui.font_color)
						.bg(THEME.read().ui.main_background_color),
				])
			}

			DeletingRequest | RenamingRequest => {
				let selected_request_index = &self.collections_tree.state.selected();
				let selected_request = &self.collections[selected_request_index[0]].requests
					[selected_request_index[1]]
					.read();

				Line::from(vec![
					Span::raw("Request > ").fg(THEME.read().ui.secondary_foreground_color),
					Span::raw(format!("{} > ", selected_request.name))
						.fg(THEME.read().ui.secondary_foreground_color),
					Span::raw(self.state.to_string())
						.fg(THEME.read().ui.font_color)
						.bg(THEME.read().ui.main_background_color),
				])
			}

			DisplayingEnvEditor | EditingEnvVariable => {
				let local_env = self.get_selected_env_as_local().unwrap();
				let env = local_env.read();

				Line::from(vec![
					Span::raw("Environment editor > ")
						.fg(THEME.read().ui.secondary_foreground_color),
					Span::raw(env.name.clone())
						.fg(THEME.read().ui.font_color)
						.bg(THEME.read().ui.main_background_color),
				])
			}

			SelectedRequest
			| EditingRequestUrl
			| EditingRequestParam
			| EditingRequestAuthBasicUsername
			| EditingRequestAuthBasicPassword
			| EditingRequestAuthBearerToken
			| EditingRequestAuthJwtSecret
			| EditingRequestAuthJwtPayload
			| EditingRequestAuthDigestUsername
			| EditingRequestAuthDigestPassword
			| EditingRequestAuthDigestDomains
			| EditingRequestAuthDigestRealm
			| EditingRequestAuthDigestNonce
			| EditingRequestAuthDigestOpaque
			| EditingRequestHeader
			| EditingRequestBodyTable
			| EditingRequestBodyFile
			| EditingRequestBodyString
			| EditingRequestMessage
			| EditingPreRequestScript
			| EditingPostRequestScript
			| EditingRequestSettings
			| ChoosingRequestExportFormat
			| DisplayingRequestExport
			| SelectingResponseBody => {
				let local_selected_request = self.get_selected_request_as_local();
				let selected_request = local_selected_request.read();

				if self.state == SelectedRequest {
					Line::from(vec![
						Span::raw("Request > ").fg(THEME.read().ui.secondary_foreground_color),
						Span::raw(selected_request.name.clone())
							.fg(THEME.read().ui.font_color)
							.bg(THEME.read().ui.main_background_color),
					])
				} else {
					Line::from(vec![
						Span::raw("Request > ").fg(THEME.read().ui.secondary_foreground_color),
						Span::raw(format!("{} > ", selected_request.name))
							.fg(THEME.read().ui.secondary_foreground_color),
						Span::raw(self.state.to_string())
							.fg(THEME.read().ui.font_color)
							.bg(THEME.read().ui.main_background_color),
					])
				}
			}
		}
	}

	pub fn in_input(&self) -> bool {
		matches!(
			self.state,
			EditingEnvVariable
				| CreatingNewCollection
				| CreatingNewRequest
				| RenamingCollection
				| RenamingRequest
				| EditingRequestUrl
				| EditingRequestParam
				| EditingRequestAuthBasicUsername
				| EditingRequestAuthBasicPassword
				| EditingRequestAuthBearerToken
				| EditingRequestAuthJwtSecret
				| EditingRequestAuthJwtPayload
				| EditingRequestHeader
				| EditingRequestBodyTable
				| EditingRequestBodyFile
				| EditingRequestBodyString
				| EditingPreRequestScript
				| EditingPostRequestScript
				| EditingRequestSettings
				| SelectingResponseBody
		)
	}
}
