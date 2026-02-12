use chrono::Utc;
use parking_lot::Mutex;
use std::fmt::{Debug, Write};
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::field::{Field, Visit};
use tracing::{Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

// Avoids been lock inside the logger widget when moving around
pub static SHOULD_RECORD_LOGS: AtomicBool = AtomicBool::new(true);

/// Timestamp, level, target, and message for each log entry.
pub type LogEntry = (String, Level, String, String);

pub static LOGS: LazyLock<Mutex<Vec<LogEntry>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub struct LogCounterLayer;

impl<S: Subscriber> Layer<S> for LogCounterLayer {
	fn on_event(&self, e: &tracing::Event<'_>, _: Context<'_, S>) {
		if !SHOULD_RECORD_LOGS.load(Ordering::SeqCst) {
			return;
		}

		let now = Utc::now().format("%H:%M:%S").to_string();
		let level = *e.metadata().level();
		let target = e.metadata().target().to_string();
		let mut message = String::new();
		e.record(&mut StringVisitor(&mut message));

		let mut logs = LOGS.lock();
		logs.push((now, level, target, message));
		// Prevents keeping too much logs
		if logs.len() > 1000 {
			logs.pop()
				.expect("logs should not be empty when over capacity");
		}
	}
}

pub struct StringVisitor<'a>(&'a mut String);

impl<'a> Visit for StringVisitor<'a> {
	fn record_debug(&mut self, _: &Field, value: &dyn Debug) {
		self.0
			.write_str(&format!("{value:?}"))
			.expect("writing to log buffer should succeed");
	}
}
