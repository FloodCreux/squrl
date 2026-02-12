//! Application-wide constants to avoid magic numbers and strings scattered throughout the codebase.

use std::time::Duration;

/// How often the TUI event loop polls for new events.
pub const TICK_RATE: Duration = Duration::from_millis(250);

/// Default timeout for WebSocket connection attempts.
pub const WS_CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Polling interval for reading WebSocket messages.
pub const WS_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Prefix used in multipart form values to indicate the value is a file path.
/// For example, `"!!/path/to/file"` means the multipart part should read from `/path/to/file`.
pub const FILE_VALUE_PREFIX: &str = "!!";
