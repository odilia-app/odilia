use serde::{Deserialize, Serialize};
///structure used for all the configurable options related to logging
#[derive(Debug, Serialize, Deserialize)]
pub struct LogSettings {
    level: String,
}
