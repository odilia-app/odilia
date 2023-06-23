#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	missing_docs,
	unsafe_code
)]

pub mod cache;
pub mod errors;
pub mod events;
pub mod commands;
pub mod settings;
pub mod types;

/// A result type that is generally quicker to write when using Result<T, OdiliaError>.
pub type OdiliaResult<T> = Result<T, errors::OdiliaError>;

/// The mode of the screen reader.
/// This is merely a way to indicate that certain key combinations or functioanlity should or should not work.
pub type ScreenReaderMode = String;
