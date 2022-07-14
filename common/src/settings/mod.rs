mod log;
mod speech;
use log::LogSettings;
use speech::SpeechSettings;

use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};

///type representing a *read-only* view of the odilia screenreader configuration
/// this type should only be obtained as a result of parsing odilia's configuration files, as it containes types for each section responsible for controlling various parts of the screenreader
/// the only way this config should change is if the configuration file changes, in which case the entire view will be replaced to reflect the fact
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationConfig {
    speech: SpeechSettings,
    log: LogSettings,
}

impl ApplicationConfig {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        let s = Config::builder()
            // Start off by merging in the "default" configuration file specified by the path parameter
            .add_source(File::with_name(path))
            //add configuration from the environment, any variable prefixed by odilia is a candidate
            // Eg `ODILIA_LEVEL=info ./target/odilia` would set the log level to `info`
            .add_source(Environment::with_prefix("odilia"))
            //finally, build the config
            .build()?;
        s.try_deserialize()
    }

    pub fn log(&self) -> &LogSettings {
        &self.log
    }

    pub fn speech(&self) -> &SpeechSettings {
        &self.speech
    }
}
