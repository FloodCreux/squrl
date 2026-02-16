//! Core application logic: state management, file I/O, request preparation, and startup.

#[allow(clippy::module_inception)]
mod app;
pub(crate) mod collection;
pub mod constants;
pub(crate) mod environment;
pub(crate) mod files;
pub(crate) mod key_value;
pub(crate) mod log;
pub mod request;
pub mod startup;
pub(crate) mod utils;

pub use app::App;
