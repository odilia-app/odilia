// Needed because this is generated code
// Todo: Remove when we've defined propper types for all arguments and return values
#![allow(clippy::type_complexity, clippy::too_many_arguments)]
pub mod accessible;
pub mod action;
pub mod application;
pub mod bus;
pub mod cache;
pub mod collection;
pub mod component;
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
pub mod value;

// Hand-written connection module
mod connection;
pub use connection::*;

pub use zbus;
