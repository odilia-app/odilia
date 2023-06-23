#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	missing_docs,
	unsafe_code
)]

use zvariant::ObjectPath;
use zbus_names::UniqueName; 

pub mod cache;
pub mod elements;
pub mod errors;
pub mod events;
pub mod commands;
pub mod modes;
pub mod result;
pub mod settings;
pub mod types;

pub use result::OdiliaResult as Result;
