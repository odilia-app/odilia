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
pub mod elements;
pub mod errors;
pub mod events;
pub mod commands;
pub mod modes;
pub mod result;
pub mod settings;
pub mod types;

/// A result type that is generally quicker to write when using Result<T, OdiliaError>.
pub type OdiliaResult<T> = Result<T, errors::OdiliaError>;
