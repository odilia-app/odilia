//! All settings used within Odilia.
//! This may eventually need to be extensible for use in addons.

mod log;
mod speech;
use log::LogSettings;
use speech::SpeechSettings;

use serde::{Deserialize, Serialize};
use tini::Ini;

use crate::errors::ConfigError;

///type representing a *read-only* view of the odilia screenreader configuration
/// this type should only be obtained as a result of parsing odilia's configuration files, as it containes types for each section responsible for controlling various parts of the screenreader
/// the only way this config should change is if the configuration file changes, in which case the entire view will be replaced to reflect the fact
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationConfig {
	/// specific speech settings
	speech: SpeechSettings,
	/// specific log settings
	log: LogSettings,
}

impl ApplicationConfig {
	/// Opens a new config file with a certain path.
	///
	/// # Errors
	///
	/// This can return `Err(_)` if the path doesn't exist, or if not all the key/value pairs are defined.
	pub fn new(path: &str) -> Result<Self, ConfigError> {
		let ini = Ini::from_file(path)?;
		let rate: i32 = ini.get("speech", "rate").ok_or(ConfigError::ValueNotFound)?;
		let level: String = ini.get("log", "level").ok_or(ConfigError::ValueNotFound)?;
		let speech = SpeechSettings::new(rate);
		let log = LogSettings::new(level);
		Ok(Self { speech, log })
	}

	/// Get log settings
	#[must_use]
	pub fn log(&self) -> &LogSettings {
		&self.log
	}

	/// Get speech settings
	#[must_use]
	pub fn speech(&self) -> &SpeechSettings {
		&self.speech
	}
}
