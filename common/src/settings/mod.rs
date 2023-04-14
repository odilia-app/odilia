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
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplicationConfig {
	speech: SpeechSettings,
	log: LogSettings,
}

impl ApplicationConfig {
	/// Uses a [`tini`] structure to build the config. This is only used internally to create a common function.
	///
	/// # Errors
	///
	/// This can return `Err(_)` if not all the key/value pairs are defined.
	fn from_ini(ini: &Ini) -> Result<Self, ConfigError> {
		let rate: i32 = ini.get("speech", "rate").ok_or(ConfigError::ValueNotFound)?;
		let level: String = ini.get("log", "level").ok_or(ConfigError::ValueNotFound)?;
		let speech = SpeechSettings::new(rate);
		let log = LogSettings::new(level);
		Ok(Self { speech, log })
	}
	/// Opens a new config file with a certain path.
	///
	/// # Errors
	///
	/// This can return `Err(_)` if the path doesn't exist, or if not all the key/value pairs are defined.
	pub fn from_path(path: &str) -> Result<Self, ConfigError> {
		let ini = Ini::from_file(path)?;
    Self::from_ini(&ini)
	}

	/// Uses the text given to create a config structure.
	///
	/// # Errors
	///
	/// This can return `Err(_)` if not all the key/value pairs are defined.
	pub fn from_string(config_str: &str) -> Result<Self, ConfigError> {
		let ini = Ini::from_string(config_str)?;
    Self::from_ini(&ini)
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

#[cfg(test)]
mod tests {
  use crate::settings::{
    ApplicationConfig,
    SpeechSettings,
    LogSettings,
  };

  #[test]
  fn check_valid_config() {
    let config = ApplicationConfig::from_string(include_str!("../../../odilia/config.toml"));
    assert_eq!(
      config.unwrap(),
      ApplicationConfig {
        speech: SpeechSettings {
          rate: 100,
        },
        log: LogSettings {
          level: "debug".to_string()
        },
      }
    )
  }
  
  #[test]
  fn check_invalid_config() {
    let config = ApplicationConfig::from_string(include_str!("./invalid-config.toml"));
    assert_eq!(
      config.is_err(),
      true
    )
  }
  
}
