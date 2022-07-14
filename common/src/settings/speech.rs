
use serde::{Deserialize, Serialize};
///structure for all the speech related configuration options available in odilia
#[derive(Debug, Serialize, Deserialize)]
pub struct SpeechSettings {
    pub rate: i32,
}
