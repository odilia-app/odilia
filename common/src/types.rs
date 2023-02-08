use atspi::text::Granularity;

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
