use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct ScreenReaderMode {
	pub name: String,
}

impl ScreenReaderMode {
	pub fn new(name: &str) -> Self {
		ScreenReaderMode { name: name.to_string() }
	}
}
