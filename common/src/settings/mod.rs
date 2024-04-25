pub mod log;
pub mod speech;

use log::LogSettings;
use speech::SpeechSettings;

use serde::{Deserialize, Serialize};

///type representing a *read-only* view of the odilia screenreader configuration
/// this type should only be obtained as a result of parsing odilia's configuration files, as it containes types for each section responsible for controlling various parts of the screenreader
/// the only way this config should change is if the configuration file changes, in which case the entire view will be replaced to reflect the fact
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ApplicationConfig {
	pub speech: SpeechSettings,
	pub log: LogSettings,
}

