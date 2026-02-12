//! Core application logic: state management, file I/O, request preparation, and startup.

#[allow(clippy::module_inception)]
pub mod app;
pub(crate) mod collection;
pub mod constants;
pub(crate) mod environment;
pub(crate) mod files;
pub(crate) mod key_value;
pub(crate) mod log;
pub mod request;
#[allow(clippy::module_inception)]
pub mod startup;
pub(crate) mod utils;
