mod log;
mod speech;
use log::LogSettings;
use speech::SpeechSettings;

use serde::{Deserialize, Serialize};
use tini::{Error, Ini};

///type representing a *read-only* view of the odilia screenreader configuration
/// this type should only be obtained as a result of parsing odilia's configuration files, as it containes types for each section responsible for controlling various parts of the screenreader
/// the only way this config should change is if the configuration file changes, in which case the entire view will be replaced to reflect the fact
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationConfig {
    speech: SpeechSettings,
    log: LogSettings,
}

impl ApplicationConfig {
    pub fn new(path: &str) -> Result<Self, Error> {
        let ini = Ini::from_file(path)?;
        let rate: i32 = ini.get("speech", "rate").unwrap();
        let level: String = ini.get("log", "level").unwrap();
        let speech = SpeechSettings::new(rate);
        let log = LogSettings::new(level);
        Ok(Self { speech, log })
    }

    pub fn log(&self) -> &LogSettings {
        &self.log
    }

    pub fn speech(&self) -> &SpeechSettings {
        &self.speech
    }
}
