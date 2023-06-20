use parking_lot::RwLock;
use std::{
	sync::{Arc, Weak},
	hash::{Hash, Hasher},
};
use atspi_common::{InterfaceSet, StateSet, Role};
use serde::{Serialize, Deserialize};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use zvariant::{OwnedObjectPath, ObjectPath};
use zbus_names::OwnedUniqueName;
use crate::errors::AccessiblePrimitiveConversionError;
#[cfg(feature = "proxies")]
use atspi_proxies::accessible::AccessibleProxy;

/// This is the type alias refering to the key for all cache items.
/// Please do not use its underlying type explicitly, since this will cause compiler errors when this is modified.
pub type CacheKey = AccessiblePrimitive;
/// This is the type alis refeering to the value for all cache items.
/// This includes thread-safe and concurrency-safe wrappers.
pub type CacheValue = Arc<RwLock<CacheItem>>;
/// This is the type alis refereing to a weak version of the value for all cache items.
/// This can be upgraded to a [`CacheValue`] with `.upgrade()`, where it may or may not be found.
pub type WeakCacheValue = Weak<RwLock<CacheItem>>;
/// The `InnerCache` type alias defines the data structure to be used to hold the entire cache.
pub type InnerCache = DashMap<CacheKey, CacheValue, FxBuildHasher>;
/// A wrapped [`InnerCache`] in a thread-safe type.
pub type ThreadSafeCache = Arc<InnerCache>;

/// A composition of an accessible ID and (possibly) a reference
/// to its `CacheItem`, if the item has not been dropped from the cache yet.
/// TODO if desirable, we could make one direction strong references (e.g. have
/// the parent be an Arc, xor have the children be Arcs). Might even be possible to have both.
/// BUT - is it even desirable to keep an item pinned in an Arc from its
/// releatives after it has been removed from the cache?
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CacheRef {
	pub key: CacheKey,
	#[serde(skip)]
	pub item: Weak<RwLock<CacheItem>>,
}
impl Hash for CacheRef {
	fn hash<H>(&self, hasher: &mut H) where H: Hasher {
		self.key.hash(hasher)
	}
}
impl PartialEq for CacheRef {
	fn eq(&self, other: &CacheRef) -> bool {
		self.key == other.key
	}
}
impl Eq for CacheRef {}

impl CacheRef {
	#[must_use]
	pub fn new(key: AccessiblePrimitive) -> Self {
		Self { key, item: Weak::new() }
	}

	#[must_use]
	pub fn clone_inner(&self) -> Option<CacheItem> {
		Some(self.item.upgrade().as_ref()?.read().clone())
	}
}

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

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A struct representing an accessible. To get any information from the cache other than the stored information like role, interfaces, and states, you will need to instantiate an [`atspi_proxies::accessible::AccessibleProxy`] or other `*Proxy` type from atspi to query further info.
pub struct CacheItem {
	// The accessible object (within the application)	(so)
	pub object: AccessiblePrimitive,
	// The application (root object(?)	  (so)
	pub app: AccessiblePrimitive,
	// The parent object.  (so)
	pub parent: CacheRef,
	// The accessbile index in parent.	i
	pub index: i32,
	// Child count of the accessible  i
	pub children_num: i32,
	// The exposed interfece(s) set.  as
	pub interfaces: InterfaceSet,
	// Accessible role. u
	pub role: Role,
	// The states applicable to the accessible.  au
	pub states: StateSet,
	// The text of the accessible.
	pub text: String,
	// The children (ids) of the accessible.
	pub children: Vec<CacheRef>,
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
	fn from(
		atspi_accessible: atspi_common::events::Accessible,
	) -> AccessiblePrimitive {
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
