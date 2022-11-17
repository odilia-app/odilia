#![deny(clippy::all, clippy::pedantic, clippy::cargo, unsafe_code)]
// #![deny(clippy::missing_docs)]

pub mod accessible;
pub mod action;
pub mod application;
pub mod bus;
pub mod cache;
pub mod collection;
pub mod component;
pub mod convertable;
pub mod device_event_controller;
pub mod device_event_listener;
pub mod document;
pub mod editable_text;
pub mod events;
pub use events::EventBody;
pub mod hyperlink;
pub mod hypertext;
pub mod image;
pub mod processed;
pub mod registry;
pub mod selection;
pub mod socket;
pub mod table;
pub mod table_cell;
pub mod text;
pub mod value;

pub mod accessible_ext;

// Hand-written connection module
mod connection;
pub use connection::*;

mod interfaces;
pub use interfaces::*;

mod state;
pub use state::*;

pub use zbus;

use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[repr(u32)]
/// The relative coordinate type.
pub enum CoordType {
    /// In relation to the entire screen.
    Screen,
    /// In relation to only the window.
    Window,
    /// In relation to the parent of the element being checked.
    Parent,
}
