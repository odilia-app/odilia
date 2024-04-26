use serde::{Deserialize, Serialize};
///structure for all the speech related configuration options available in odilia
#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct SpeechSettings {
	pub rate: i8,
	pub pitch: i8,
	pub volume: i8,
	pub module: String,
	pub language: String,
	pub person: String,
	pub punctuation: PunctuationSpellingMode,
}
impl Default for SpeechSettings {
	fn default() -> Self {
		Self {
			rate: 50,
			pitch: 0,
			volume: 100,
			module: "espeak-ng".into(),
			language: "en-US".into(),
			person: "English (America)+Max".into(),
			punctuation: PunctuationSpellingMode::Some,
		}
	}
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PunctuationSpellingMode {
	Some,
	Most,
	None,
	All,
}
