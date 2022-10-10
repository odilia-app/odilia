use atspi::text::TextGranularity;
use zbus::zvariant::OwnedObjectPath;

pub type Accessible = (String, OwnedObjectPath);

pub struct IndexesSelection {
  pub start: i32,
  pub end: i32,
}
pub struct GranularSelection {
  pub index: i32,
  pub granularity: TextGranularity,
}

pub enum TextSelectionArea {
  Index(IndexesSelection),
  Granular(GranularSelection),
}
