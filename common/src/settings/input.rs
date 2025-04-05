use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum InputSettings {
	#[default]
	Keyboard,
	Custom(String),
}
