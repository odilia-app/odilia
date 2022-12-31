use zbus::{names::UniqueName, zvariant::ObjectPath};

pub mod elements;
pub mod errors;
pub mod events;
pub mod input;
pub mod modes;
pub mod settings;
pub mod types;
pub mod result;

pub type Accessible = (UniqueName<'static>, ObjectPath<'static>);
pub use result::OdiliaResult as Result;
