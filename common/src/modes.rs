use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize, Copy)]
pub enum ScreenReaderMode {
	Focus,
	Browse,
}
