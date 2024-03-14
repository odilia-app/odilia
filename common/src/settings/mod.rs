mod log;
mod speech;

use log::LogSettings;
use speech::SpeechSettings;

use serde::{Deserialize, Serialize};

use figment::{
	providers::{Env, Format, Serialized, Toml},
	Figment,
};

use crate::errors::ConfigError;

///type representing a *read-only* view of the odilia screenreader configuration
/// this type should only be obtained as a result of parsing odilia's configuration files, as it containes types for each section responsible for controlling various parts of the screenreader
/// the only way this config should change is if the configuration file changes, in which case the entire view will be replaced to reflect the fact
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ApplicationConfig {
	speech: SpeechSettings,
	log: LogSettings,
}

impl ApplicationConfig {
	/// Opens a new config file with a certain path.
	///
	/// # Errors
	///
	/// This can return `Err(_)` if the path doesn't exist, or if not all the key/value pairs are defined.
	pub fn new(path: &str) -> Result<Self, ConfigError> {
		let config: Self =
			Figment::from(Serialized::defaults(ApplicationConfig::default()))
				.merge(Toml::file(path))
				.merge(Env::prefixed("ODILIA_"))
				.extract()?;
		Ok(config)
	}

	#[must_use]
	pub fn log(&self) -> &LogSettings {
		&self.log
	}

	#[must_use]
	pub fn speech(&self) -> &SpeechSettings {
		&self.speech
	}
}
