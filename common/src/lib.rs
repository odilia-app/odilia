use zbus::{names::UniqueName, zvariant::ObjectPath};

pub mod elements;
pub mod errors;
pub mod events;
pub mod input;
pub mod modes;
pub mod settings;
pub mod types;

pub type Accessible = (UniqueName<'static>, ObjectPath<'static>);
