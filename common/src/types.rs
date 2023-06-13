use atspi_common::Granularity;
use serde::{self, Deserialize, Serialize};
use zbus::zvariant::OwnedObjectPath;

pub type Accessible = (String, OwnedObjectPath);

pub struct IndexesSelection {
	pub start: i32,
	pub end: i32,
}
pub struct GranularSelection {
	pub index: i32,
	pub granularity: Granularity,
}

pub enum TextSelectionArea {
	Index(IndexesSelection),
	Granular(GranularSelection),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase", untagged)]
pub enum AriaLive {
	Off,
	Assertive,
	Polite,
	Other(String),
}

pub type AriaAtomic = bool;
