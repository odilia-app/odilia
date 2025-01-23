use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub enum ScreenReaderMode {
	Focus,
	Browse,
}
