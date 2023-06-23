#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	missing_docs,
	unsafe_code
)]
//! # Odilia Common
//!
//! This crate defines all the types needed to communicate with Odilia.
//! Whether that is from a socket through the `odilia-input` create, or through a direct plugin.
//! These types must be able to compile in a variety of architectures.
//! This means no use of `tokio` (async runtime), or `zbus` (dbus communicate) will be accepted.
//! Or, if they are necessary, they must be behind a feature flag.

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
