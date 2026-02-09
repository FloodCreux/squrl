use std::sync::Arc;

use crate::tui::utils::stateful::cookie_table::StatefulCookieTable;
use reqwest_cookie_store::CookieStoreRwLock;

#[derive(Default)]
pub struct CookiesPopup {
	pub cookies_table: StatefulCookieTable,
	pub cookie_store: Arc<CookieStoreRwLock>,
}
