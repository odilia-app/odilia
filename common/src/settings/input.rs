use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InputSettings {
	pub method: InputMethod,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum InputMethod {
	#[default]
	Keyboard,
	Custom(String),
}
