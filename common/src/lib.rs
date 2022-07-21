use zbus::{
  names::UniqueName,
  zvariant::ObjectPath,
};

pub mod settings;
pub mod types;

pub type Accessible = (UniqueName<'static>, ObjectPath<'static>);
