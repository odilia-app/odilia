use serde::{Deserialize, Serialize};
///structure used for all the configurable options related to logging
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct LogSettings {
	pub level: String,
}
impl LogSettings {
	pub fn new(level: String) -> Self {
		Self { level }
	}
}
