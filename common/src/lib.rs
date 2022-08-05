use zbus::{
  names::UniqueName,
  zvariant::ObjectPath,
};

pub mod settings;
pub mod types;
pub mod input;
pub mod modes;
pub mod errors;
pub mod events;
pub mod elements;

pub type Accessible = (UniqueName<'static>, ObjectPath<'static>);
