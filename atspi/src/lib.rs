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
pub mod text;
pub mod selection;
pub mod socket;
pub mod table;
pub mod table_cell;
pub mod value;

pub mod accessible_plus;
pub mod text_plus;

// Hand-written connection module
mod connection;
pub use connection::*;

pub use zbus;

use serde::Serialize;
use zbus::zvariant::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Type)]
#[repr(u32)]
pub enum CoordType {
    Screen,
    Window,
    Parent,
}
