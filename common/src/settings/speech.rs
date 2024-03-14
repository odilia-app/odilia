use serde::{Deserialize, Serialize};
///structure for all the speech related configuration options available in odilia
#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct SpeechSettings {
	pub rate: i32,
}
impl Default for SpeechSettings {
	fn default() -> Self {
		Self { rate: 50 }
	}
}
