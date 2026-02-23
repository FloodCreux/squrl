use std::io::Stdout;
use std::sync::Arc;
use std::time::Duration;

use crate::app::constants::TICK_RATE;
use parking_lot::{Mutex, RwLock};
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::terminal::disable_raw_mode;
use strum::VariantArray;
use throbber_widgets_tui::ThrobberState;

use crate::app::files::config::Config;
use crate::app::files::theme::THEME;
use crate::models::collection::Collection;
use crate::models::environment::Environment;
use crate::models::export::ExportFormat;
use crate::tui::app_states::AppState;
use crate::tui::ui::param_tabs::param_tabs::RequestParamsTabs;
use crate::tui::ui::result_tabs::RequestResultTabs;
use crate::tui::ui::views::RequestView;
use crate::tui::utils::stateful::choice_popup::ChoicePopup;
use crate::tui::utils::stateful::cookies_popup::CookiesPopup;
use crate::tui::utils::stateful::display_popup::DisplayPopup;
use crate::tui::utils::stateful::help_popup::HelpPopup;
use crate::tui::utils::stateful::new_request_popup::NewRequestPopup;
use crate::tui::utils::stateful::script_console::ScriptConsole;
use crate::tui::utils::stateful::settings_popup::SettingsPopup;
use crate::tui::utils::stateful::stateful_custom_table::StatefulCustomTable;
use crate::tui::utils::stateful::stateful_scrollbar::StatefulScrollbar;
use crate::tui::utils::stateful::stateful_tree::StatefulTree;
use crate::tui::utils::stateful::text_input::TextInput;
use crate::tui::utils::stateful::text_input_selection::TextInputSelection;
use crate::tui::utils::stateful::theme_popup::ThemePopup;
use crate::tui::utils::stateful::validation_popup::ValidationPopup;
use crate::tui::utils::syntax_highlighting::SyntaxHighlighting;
#[cfg(feature = "clipboard")]
use arboard::Clipboard;
use ratatui::prelude::{Line, Stylize};

/// Grouped TUI widget state for all authentication text inputs.
///
/// Each auth type (Basic, Bearer, JWT, Digest) has one or more text inputs.
/// The `text_input_selection` tracks which input is currently focused.
pub struct AuthInputs {
	pub text_input_selection: TextInputSelection,
	pub basic_username: TextInput,
	pub basic_password: TextInput,
	pub bearer_token: TextInput,
	pub jwt_secret: TextInput,
	pub jwt_payload: TextInput,
	pub digest_username: TextInput,
	pub digest_password: TextInput,
	pub digest_domains: TextInput,
	pub digest_realm: TextInput,
	pub digest_nonce: TextInput,
	pub digest_opaque: TextInput,
}

/// Grouped TUI widget state for editing the currently-selected request.
///
/// Contains the URL input, query params table, auth inputs, headers table,
/// body inputs (file, form, text area), and request settings popup.
pub struct RequestEditorState<'a> {
	pub url_input: TextInput,
	pub query_params_table: StatefulCustomTable<'a>,
	pub auth: AuthInputs,
	pub headers_table: StatefulCustomTable<'a>,
	pub body_file_input: TextInput,
	pub body_form_table: StatefulCustomTable<'a>,
	pub body_text_area: TextInput,
	pub settings_popup: SettingsPopup,
}

/// Grouped TUI widget state for collection/request/folder management popups.
///
/// Contains popups and text inputs for creating, renaming, and deleting
/// collections, requests, and folders.
pub struct CollectionPopups {
	pub creation_popup: ChoicePopup<String>,
	pub new_collection_input: TextInput,
	pub rename_collection_input: TextInput,
	pub new_request_popup: NewRequestPopup,
	pub rename_request_input: TextInput,
	pub delete_collection_popup: ValidationPopup,
	pub delete_request_popup: ValidationPopup,
}

/// Grouped TUI widget state for displaying response results.
///
/// Contains the loading throbber, vertical/horizontal scrollbars for the
/// result pane, and the text area for response body selection.
pub struct ResponseViewState {
	pub throbber_state: ThrobberState,
	pub vertical_scrollbar: StatefulScrollbar,
	pub horizontal_scrollbar: StatefulScrollbar,
	pub body_text_area: TextInput,
}

/// Core application state shared between TUI and CLI modes.
///
/// This struct holds the data that is needed by both the interactive TUI and
/// the headless CLI: collections, environments, configuration, the cookie
/// store, and the response-received signal.
pub struct CoreState {
	pub config: Config,
	pub collections: Vec<Collection>,
	pub environments: Vec<Arc<RwLock<Environment>>>,
	pub selected_environment: usize,
	pub cookies_popup: CookiesPopup,
	pub received_response: Arc<Mutex<bool>>,
	pub env_json_changed: Arc<Mutex<bool>>,
	pub _env_watcher: Option<notify::RecommendedWatcher>,
}

pub struct App<'a> {
	pub core: CoreState,

	pub tick_rate: Duration,
	pub should_quit: bool,
	pub should_display_help: bool,

	pub state: AppState,
	pub was_last_state_selected_request: bool,

	/* Help */
	pub help_popup: HelpPopup,

	/* Environments */
	pub env_editor_table: StatefulCustomTable<'a>,

	/* Logs */
	pub logs_vertical_scrollbar: StatefulScrollbar,
	pub logs_horizontal_scrollbar: StatefulScrollbar,

	/* Collections */
	pub collections_tree: StatefulTree<'a>,

	pub request_view: RequestView,
	pub request_param_tab: RequestParamsTabs,
	pub request_result_tab: RequestResultTabs,

	pub collection_popups: CollectionPopups,

	/* Request editor (URL, params, auth, headers, body, settings) */
	pub request_editor: RequestEditorState<'a>,

	/* WS message */
	pub message_text_area: TextInput,

	/* GraphQL */
	pub graphql_query_text_area: TextInput,
	pub graphql_variables_text_area: TextInput,

	/* gRPC */
	pub grpc_proto_file_input: TextInput,
	pub grpc_service_input: TextInput,
	pub grpc_method_input: TextInput,
	pub grpc_message_text_area: TextInput,

	/* Response */
	pub response_view: ResponseViewState,

	pub last_messages_area_size: (u16, u16),

	/* Scripts */
	pub script_console: ScriptConsole,

	/* Others */
	pub syntax_highlighting: SyntaxHighlighting,

	pub export_request: ChoicePopup<ExportFormat>,
	pub display_request_export: DisplayPopup,

	/* Theme */
	pub theme_popup: ThemePopup,

	#[cfg(feature = "clipboard")]
	pub clipboard: Option<Clipboard>,
}

impl App<'_> {
	pub fn new<'a>() -> anyhow::Result<App<'a>> {
		Ok(App {
			core: CoreState {
				config: Config::default(),
				collections: vec![],
				environments: vec![],
				selected_environment: 0,
				cookies_popup: CookiesPopup::default(),
				received_response: Arc::new(Mutex::new(false)),
				env_json_changed: Arc::new(Mutex::new(false)),
				_env_watcher: None,
			},

			tick_rate: TICK_RATE,
			should_quit: false,
			should_display_help: false,

			state: AppState::Normal,
			was_last_state_selected_request: false,

			/* Help */
			help_popup: HelpPopup::default(),

			/* Environments */
			env_editor_table: StatefulCustomTable::new(
				vec![
					Line::default(),
					Line::from("No environment variable").fg(THEME.read().ui.font_color),
					Line::from("(Add one with n)").fg(THEME.read().ui.secondary_foreground_color),
				],
				"Key",
				"Value",
			),

			/* Logs */
			logs_vertical_scrollbar: StatefulScrollbar::default(),
			logs_horizontal_scrollbar: StatefulScrollbar::default(),

			/* Collections */
			collections_tree: StatefulTree::default(),

			request_view: RequestView::Normal,

			request_param_tab: RequestParamsTabs::QueryParams,
			request_result_tab: RequestResultTabs::Body,

			collection_popups: CollectionPopups {
				creation_popup: ChoicePopup {
					choices: vec![
						String::from("Collection"),
						String::from("Request"),
						String::from("Folder"),
					],
					selection: 0,
				},
				new_collection_input: TextInput::new(None),
				rename_collection_input: TextInput::new(None),
				new_request_popup: NewRequestPopup::default(),
				rename_request_input: TextInput::new(None),
				delete_collection_popup: ValidationPopup::default(),
				delete_request_popup: ValidationPopup::default(),
			},

			/* Request editor */
			request_editor: RequestEditorState {
				url_input: TextInput::new(Some(String::from("URL"))),
				query_params_table: StatefulCustomTable::new(
					vec![
						Line::default(),
						Line::from("No params").fg(THEME.read().ui.font_color),
						Line::from("(Add one with n or via the URL)")
							.fg(THEME.read().ui.secondary_foreground_color),
					],
					"Param",
					"Value",
				),
				auth: AuthInputs {
					text_input_selection: TextInputSelection::default(),
					basic_username: TextInput::new(Some(String::from("Username"))),
					basic_password: TextInput::new(Some(String::from("Password"))),
					bearer_token: TextInput::new(Some(String::from("Bearer token"))),
					jwt_secret: TextInput::new(Some(String::from("Secret"))),
					jwt_payload: TextInput::new(Some(String::from("Payload"))),
					digest_username: TextInput::new(Some(String::from("Username"))),
					digest_password: TextInput::new(Some(String::from("Password"))),
					digest_domains: TextInput::new(Some(String::from("Domains"))),
					digest_realm: TextInput::new(Some(String::from("Realm"))),
					digest_nonce: TextInput::new(Some(String::from("Nonce"))),
					digest_opaque: TextInput::new(Some(String::from("Opaque"))),
				},
				headers_table: StatefulCustomTable::new(
					vec![
						Line::default(),
						Line::from("Default headers").fg(THEME.read().ui.font_color),
						Line::from("(Add one with n)")
							.fg(THEME.read().ui.secondary_foreground_color),
					],
					"Header",
					"Value",
				),
				body_file_input: TextInput::new(Some(String::from("File path"))),
				body_form_table: StatefulCustomTable::new(
					vec![
						Line::default(),
						Line::from("No form data").fg(THEME.read().ui.font_color),
						Line::from("(Add one with n)")
							.fg(THEME.read().ui.secondary_foreground_color),
					],
					"Key",
					"Value",
				),
				body_text_area: TextInput::new(None),
				settings_popup: SettingsPopup::default(),
			},

			/* WS message */
			message_text_area: TextInput::new(None),

			/* GraphQL */
			graphql_query_text_area: TextInput::new(None),
			graphql_variables_text_area: TextInput::new(None),

			/* gRPC */
			grpc_proto_file_input: TextInput::new(Some(String::from("Proto file path"))),
			grpc_service_input: TextInput::new(Some(String::from("Service"))),
			grpc_method_input: TextInput::new(Some(String::from("Method"))),
			grpc_message_text_area: TextInput::new(None),

			/* Response */
			response_view: ResponseViewState {
				throbber_state: ThrobberState::default(),
				vertical_scrollbar: StatefulScrollbar::default(),
				horizontal_scrollbar: StatefulScrollbar::default(),
				body_text_area: TextInput::new_multiline(),
			},

			last_messages_area_size: (0, 0),
			script_console: ScriptConsole {
				pre_request_text_area: TextInput::new(None),
				post_request_text_area: TextInput::new(None),
				script_selection: 0,
			},

			/* Others */
			syntax_highlighting: SyntaxHighlighting::default(),
			export_request: ChoicePopup {
				choices: ExportFormat::VARIANTS.to_vec(),
				selection: 0,
			},
			display_request_export: DisplayPopup::default(),

			/* Theme */
			theme_popup: ThemePopup::new(),

			#[cfg(feature = "clipboard")]
			clipboard: Clipboard::new().ok(),
		})
	}

	pub async fn run(
		&mut self,
		mut terminal: Terminal<CrosstermBackend<Stdout>>,
	) -> Result<(), <CrosstermBackend<Stdout> as Backend>::Error> {
		terminal.clear()?;

		while !self.should_quit {
			self.update_current_available_events();
			self.draw(&mut terminal)?;
			self.handle_events(&mut terminal).await;
		}

		Ok(())
	}

	pub fn chain_hook(&mut self) -> &mut Self {
		let original_hook = std::panic::take_hook();

		std::panic::set_hook(Box::new(move |panic| {
			let _ = disable_raw_mode();
			original_hook(panic);
		}));

		self
	}
}
