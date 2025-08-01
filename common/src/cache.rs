use atspi::{
	proxy::accessible::AccessibleProxy, EventProperties, InterfaceSet, ObjectRef, Role,
	StateSet,
};
use serde::{Deserialize, Serialize};
use zbus::{
	names::OwnedUniqueName,
	proxy::{Builder as ProxyBuilder, CacheProperties},
	zvariant::{ObjectPath, OwnedObjectPath, Type},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Type, Ord, PartialOrd)]
/// A struct which represents the bare minimum of an accessible for purposes of caching.
/// This makes some *possibly eronious* assumptions about what the sender is.
pub struct AccessiblePrimitive {
	/// Assuming that the sender is ":x.y", this stores the (x,y) portion of this sender.
	/// Examples:
	/// * :1.1 (the first window has opened)
	/// * :2.5 (a second session exists, where at least 5 applications have been lauinched)
	/// * :1.262 (many applications have been started on this bus)
	pub sender: String,
	/// The accessible ID, which is an arbitrary string specified by the application.
	/// It is guaranteed to be unique per application.
	/// Examples:
	/// * /org/a11y/atspi/accessible/1234
	/// * /org/a11y/atspi/accessible/null
	/// * /org/a11y/atspi/accessible/root
	/// * /org/Gnome/GTK/abab22-bbbb33-2bba2
	pub id: String,
}

impl AccessiblePrimitive {
	/// Turns any `atspi::event` type into an `AccessiblePrimitive`, the basic type which is used for keys in the cache.
	pub fn from_event<T: EventProperties>(event: &T) -> Self {
		let sender = event.sender();
		let path = event.path();
		let id = path.to_string();
		Self { id, sender: sender.as_str().into() }
	}

	/// Convert into an [`atspi::proxy::accessible::AccessibleProxy`]. Must be async because the creation of an async proxy requires async itself.
	/// # Errors
	/// Will return a [`zbus::Error`] in the case of an invalid destination, path, or failure to create a `Proxy` from those properties.
	#[cfg_attr(feature = "tracing", tracing::instrument(skip_all, level = "trace", ret, err))]
	pub async fn into_accessible<'a>(
		self,
		conn: &zbus::Connection,
	) -> zbus::Result<AccessibleProxy<'a>> {
		let id = self.id;
		let sender = self.sender.clone();
		let path: ObjectPath<'a> = id.try_into()?;
		ProxyBuilder::new(conn)
			.path(path)?
			.destination(sender.as_str().to_owned())?
			.cache_properties(CacheProperties::No)
			.build()
			.await
	}
}

impl From<ObjectRef> for AccessiblePrimitive {
	fn from(atspi_accessible: ObjectRef) -> AccessiblePrimitive {
		let tuple_converter = (atspi_accessible.name, atspi_accessible.path);
		tuple_converter.into()
	}
}

impl From<(OwnedUniqueName, OwnedObjectPath)> for AccessiblePrimitive {
	fn from(so: (OwnedUniqueName, OwnedObjectPath)) -> AccessiblePrimitive {
		let accessible_id = so.1;
		AccessiblePrimitive { id: accessible_id.to_string(), sender: so.0.as_str().into() }
	}
}
impl From<(String, OwnedObjectPath)> for AccessiblePrimitive {
	fn from(so: (String, OwnedObjectPath)) -> AccessiblePrimitive {
		let accessible_id = so.1;
		AccessiblePrimitive { id: accessible_id.to_string(), sender: so.0 }
	}
}
impl<'a> From<(String, ObjectPath<'a>)> for AccessiblePrimitive {
	fn from(so: (String, ObjectPath<'a>)) -> AccessiblePrimitive {
		AccessiblePrimitive { id: so.1.to_string(), sender: so.0 }
	}
}
impl From<&AccessibleProxy<'_>> for AccessiblePrimitive {
	fn from(accessible: &AccessibleProxy<'_>) -> AccessiblePrimitive {
		let accessible = accessible.inner();
		let sender = accessible.destination().as_str().into();
		let id = accessible.path().as_str().into();
		AccessiblePrimitive { sender, id }
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A struct representing an accessible. To get any information from the cache other than the stored information like role, interfaces, and states, you will need to instantiate an [`atspi::proxy::accessible::AccessibleProxy`] or other `*Proxy` type from atspi to query further info.
pub struct CacheItem {
	/// The accessible object (within the application)    (so)
	pub object: AccessiblePrimitive,
	/// The application (root object(?)      (so)
	pub app: AccessiblePrimitive,
	/// The parent object.  (so)
	pub parent: AccessiblePrimitive,
	/// The accessible index in parent.I
	pub index: Option<usize>,
	/// Child count of the accessible.I
	pub children_num: Option<usize>,
	/// The exposed interface(s) set
	pub interfaces: InterfaceSet,
	/// Accessible role. u
	pub role: Role,
	/// The states applicable to the accessible.  au
	pub states: StateSet,
	/// The children (ids) of the accessible
	pub children: Vec<AccessiblePrimitive>,
	/// The human-readable short name of the item. `None` if string is empty.
	pub name: Option<String>,
	/// The human-readable longer name (description) of the item. `None` if string is empty.
	pub description: Option<String>,
	/// The help-text of the item. `None` if string is empty.
	pub help_text: Option<String>,
	/// The actual, internal text of the item; this will be `None` if either the text interface isn't
	/// implemented, or if the response contains an empty string: "".
	pub text: Option<String>,
}
