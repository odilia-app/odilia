use atspi_common::Granularity;
use serde::{self, Deserialize, Serialize};
use zvariant::OwnedObjectPath;

pub type Accessible = (String, OwnedObjectPath);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct IndexesSelection {
	pub start: i32,
	pub end: i32,
}
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct GranularSelection {
	pub index: i32,
	pub granularity: Granularity,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
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
