//! # Cache
//!
//! Common types used by the Odilia caching crate.
//! Some types are not specified here. These are ones which require access to [`tokio`] types.

use crate::errors::AccessiblePrimitiveConversionError;
use atspi_common::{events::Accessible, AtspiError, InterfaceSet, Role, StateSet};
#[cfg(feature = "proxies")]
use atspi_proxies::accessible::AccessibleProxy;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use zbus_names::{OwnedUniqueName, UniqueName};
use zvariant::{ObjectPath, OwnedObjectPath};

/// This is the type alias refering to the key for all cache items.
/// Please do not use its underlying type explicitly, since this will cause compiler errors when this is modified.
#[allow(clippy::module_name_repetitions)]
pub type CacheKey = AccessiblePrimitive;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
/// A struct which represents the bare minimum of an accessible for purposes of caching.
/// This makes some *possibly eronious* assumptions about what the sender is.
pub struct AccessiblePrimitive {
	/// The accessible ID, which is an arbitrary string specified by the application.
	/// It is guarenteed to be unique per application.
	/// Examples:
	/// * /org/a11y/atspi/accessible/1234
	/// * /org/a11y/atspi/accessible/null
	/// * /org/a11y/atspi/accessible/root
	/// * /org/Gnome/GTK/abab22-bbbb33-2bba2
	pub id: String,
	/// Assuming that the sender is ":x.y", this stores the (x,y) portion of this sender.
	/// Examples:
	/// * :1.1 (the first window has opened)
	/// * :2.5 (a second session exists, where at least 5 applications have been lauinched)
	/// * :1.262 (many applications have been started on this bus)
	pub sender: smartstring::alias::String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// A struct representing an accessible to be shared across IPC.
/// This type has simplified versions of some other types that can be referenced but not directly interacted with.
/// For example, this contains no direct referencing of smart pointers, and instead simply uses the [`CacheKey`] type so that lookups to the cache can be done for any additional data.
pub struct ExternalCacheItem {
	/// The accessible object (within the application). `(so)`
	pub object: CacheKey,
	/// The application root object. `(so)`
	pub app: CacheKey,
	/// The parent object. `(so)`
	pub parent: CacheKey,
	/// The accessbile index in parent. `i`
	pub index: i32,
	/// Child count of the accessible. `i`
	pub children_num: i32,
	/// The exposed interfece(s) set. `as`
	pub interfaces: InterfaceSet,
	/// Accessible role. `u`
	pub role: Role,
	/// The states applicable to the accessible. `au`
	pub states: StateSet,
	/// The text of the accessible.
	pub text: String,
	/// The children (ids) of the accessible. `a(so)`
	pub children: Vec<CacheKey>,
}

impl From<(String, OwnedObjectPath)> for AccessiblePrimitive {
	fn from(so: (String, OwnedObjectPath)) -> AccessiblePrimitive {
		let accessible_id = so.1;
		AccessiblePrimitive { id: accessible_id.to_string(), sender: so.0.into() }
	}
}
impl<'a> From<(String, ObjectPath<'a>)> for AccessiblePrimitive {
	fn from(so: (String, ObjectPath<'a>)) -> AccessiblePrimitive {
		AccessiblePrimitive { id: so.1.to_string(), sender: so.0.into() }
	}
}
#[cfg(feature = "proxies")]
impl<'a> TryFrom<&AccessibleProxy<'a>> for AccessiblePrimitive {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(accessible: &AccessibleProxy<'_>) -> Result<AccessiblePrimitive, Self::Error> {
		let sender = accessible.destination().as_str().into();
		let id = accessible.path().as_str().into();
		Ok(AccessiblePrimitive { id, sender })
	}
}
#[cfg(feature = "proxies")]
impl<'a> TryFrom<AccessibleProxy<'a>> for AccessiblePrimitive {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(accessible: AccessibleProxy<'_>) -> Result<AccessiblePrimitive, Self::Error> {
		let sender = accessible.destination().as_str().into();
		let id = accessible.path().as_str().into();
		Ok(AccessiblePrimitive { id, sender })
	}
}

impl From<atspi_common::events::Accessible> for AccessiblePrimitive {
	fn from(atspi_accessible: atspi_common::events::Accessible) -> AccessiblePrimitive {
		AccessiblePrimitive {
			id: atspi_accessible.path.to_string(),
			sender: atspi_accessible.name.to_string().into(),
		}
	}
}
impl TryFrom<(OwnedUniqueName, OwnedObjectPath)> for AccessiblePrimitive {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(
		so: (OwnedUniqueName, OwnedObjectPath),
	) -> Result<AccessiblePrimitive, Self::Error> {
		let accessible_id = so.1;
		Ok(AccessiblePrimitive {
			id: accessible_id.to_string(),
			sender: so.0.as_str().into(),
		})
	}
}
impl From<&Accessible> for AccessiblePrimitive {
	fn from(acc: &Accessible) -> AccessiblePrimitive {
		AccessiblePrimitive {
			id: acc.path.to_string(),
			sender: acc.name.to_string().into(),
		}
	}
}
impl TryFrom<AccessiblePrimitive> for Accessible {
	type Error = AtspiError;

	fn try_from(prim: AccessiblePrimitive) -> Result<Accessible, AtspiError> {
		Ok(Accessible {
			path: ObjectPath::try_from(prim.id)?.into(),
			name: UniqueName::try_from(prim.sender.to_string())?.into(),
		})
	}
}
