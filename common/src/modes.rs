use serde::{Deserialize, Serialize};
use zbus_lockstep_macros::validate;

#[validate(signal: "ModeChanged")]
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize, Copy, zbus::zvariant::Type)]
#[repr(u32)]
pub enum ScreenReaderMode {
	Focus = 1,
	Browse = 2,
}
