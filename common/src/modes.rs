use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize, Copy)]
#[repr(u32)]
pub enum ScreenReaderMode {
	Focus = 1,
	Browse = 2,
}
