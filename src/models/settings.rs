use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSettings {
	pub timeout: Setting,
	pub pretty_print_response_content: Setting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Setting {
	Bool(bool),
	U32(u32),
}

impl Setting {
	pub fn as_bool(&self) -> bool {
		match self {
			Setting::Bool(bool) => *bool,
			Setting::U32(_) => unreachable!(),
		}
	}

	pub fn as_u32(&self) -> u32 {
		match self {
			Setting::Bool(_) => unreachable!(),
			Setting::U32(u32) => *u32,
		}
	}
}

impl Default for RequestSettings {
	fn default() -> Self {
		RequestSettings {
			timeout: Setting::U32(30000),
			pretty_print_response_content: Setting::Bool(true),
		}
	}
}
