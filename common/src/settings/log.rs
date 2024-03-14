use serde::{Deserialize, Serialize};
///structure used for all the configurable options related to logging
#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct LogSettings {
	level: String,
}
impl Default for LogSettings {
	fn default() -> Self {
		Self { level: "info".to_owned() }
	}
}
