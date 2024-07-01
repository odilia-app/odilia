use serde::{Deserialize, Serialize};
use zbus::zvariant::{OwnedValue, Type, Value};

/// A priority/urgency level.
/// [See specification here](https://specifications.freedesktop.org/notification-spec/notification-spec-latest.html#urgency-levels)
#[derive(
	Clone,
	Copy,
	Debug,
	Type,
	Serialize,
	Deserialize,
	Default,
	Value,
	OwnedValue,
	Eq,
	PartialEq,
	PartialOrd,
	Ord,
)]
#[zvariant(signature = "y")]
#[repr(u8)]
pub enum Urgency {
	Low = 0,
	#[default]
	Normal = 1,
	Critical = 2,
}
