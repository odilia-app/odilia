use atspi::{proxy::accessible::AccessibleProxy, EventProperties, ObjectRef};
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
