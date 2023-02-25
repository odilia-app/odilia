//#![deny(clippy::all, clippy::pedantic, clippy::cargo)]

use atspi::{
	accessible::{AccessibleProxy, Role},
	accessible_ext::AccessibleExt,
	accessible_id::{HasAccessibleId, AccessibleId},
	convertable::Convertable,
	events::GenericEvent,
	text_ext::TextExt,
	InterfaceSet, StateSet,
};
use odilia_common::{errors::{AccessiblePrimitiveConversionError,OdiliaError}, result::OdiliaResult};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use zbus::{
	names::OwnedUniqueName,
	zvariant::{ObjectPath, OwnedObjectPath},
	ProxyBuilder,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
/// A struct which represents the bare minimum of an accessible for purposes of caching.
/// This makes some *possibly eronious* assumptions about what the sender is.
pub struct AccessiblePrimitive {
	/// The accessible ID in /org/a11y/atspi/accessible/XYZ; note that XYZ may be equal to any positive number, 0, "null", or "root".
	pub id: AccessibleId,
	/// Assuming that the sender is ":x.y", this stores the (x,y) portion of this sender.
	pub sender: String,
}
impl AccessiblePrimitive {
	#[allow(dead_code)]
	pub async fn into_accessible<'a>(
		self,
		conn: &zbus::Connection,
	) -> zbus::Result<AccessibleProxy<'a>> {
		let id = self.id;
		let sender = self.sender.clone();
		let path: ObjectPath<'a> = id.try_into()?;
		ProxyBuilder::new(conn).path(path)?.destination(sender)?.build().await
	}
	pub fn from_event<T: GenericEvent>(
		event: &T,
	) -> Result<Self, OdiliaError> {
		let sender = match event.sender() {
			Ok(Some(s)) => s,
			Ok(None) => return Err(OdiliaError::PrimitiveConversionError(AccessiblePrimitiveConversionError::NoSender)),
			Err(_) => return Err(OdiliaError::PrimitiveConversionError(AccessiblePrimitiveConversionError::ErrSender)),
		};
		let path = match event.path() {
			Some(path) => path,
			None => return Err(OdiliaError::PrimitiveConversionError(AccessiblePrimitiveConversionError::NoPathId)),
		};
		let id: AccessibleId = match path.try_into() {
			Ok(id) => id,
			Err(e) => return Err(OdiliaError::Zvariant(e)),
		};
		Ok(Self { id, sender: sender.to_string() })
	}
}
impl TryFrom<atspi::events::Accessible> for AccessiblePrimitive {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(
		atspi_accessible: atspi::events::Accessible,
	) -> Result<AccessiblePrimitive, Self::Error> {
		let tuple_converter = (atspi_accessible.name, atspi_accessible.path);
		tuple_converter.try_into()
	}
}
impl TryFrom<(OwnedUniqueName, OwnedObjectPath)> for AccessiblePrimitive {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(
		so: (OwnedUniqueName, OwnedObjectPath),
	) -> Result<AccessiblePrimitive, Self::Error> {
		let accessible_id: AccessibleId = so.1.try_into()?;
		Ok(AccessiblePrimitive { id: accessible_id, sender: so.0.to_string() })
	}
}
impl TryFrom<(String, OwnedObjectPath)> for AccessiblePrimitive {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(so: (String, OwnedObjectPath)) -> Result<AccessiblePrimitive, Self::Error> {
		let accessible_id: AccessibleId = so.1.try_into()?;
		Ok(AccessiblePrimitive { id: accessible_id, sender: so.0 })
	}
}
impl TryFrom<(String, AccessibleId)> for AccessiblePrimitive {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(so: (String, AccessibleId)) -> Result<AccessiblePrimitive, Self::Error> {
		Ok(AccessiblePrimitive { id: so.1, sender: so.0 })
	}
}
impl<'a> TryFrom<(String, ObjectPath<'a>)> for AccessiblePrimitive {
	type Error = OdiliaError;

	fn try_from(so: (String, ObjectPath<'a>)) -> Result<AccessiblePrimitive, Self::Error> {
		let accessible_id: AccessibleId = so.1.try_into()?;
		Ok(AccessiblePrimitive { id: accessible_id, sender: so.0 })
	}
}
impl<'a> TryFrom<&AccessibleProxy<'a>> for AccessiblePrimitive {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(accessible: &AccessibleProxy<'_>) -> Result<AccessiblePrimitive, Self::Error> {
		let sender = accessible.destination().to_string();
		let id = match accessible.id() {
			Ok(path_id) => path_id,
			Err(_) => return Err(AccessiblePrimitiveConversionError::NoPathId),
		};
		Ok(AccessiblePrimitive { id, sender })
	}
}
impl<'a> TryFrom<AccessibleProxy<'a>> for AccessiblePrimitive {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(accessible: AccessibleProxy<'_>) -> Result<AccessiblePrimitive, Self::Error> {
		let sender = accessible.destination().to_string();
		let id = match accessible.id() {
			Ok(path_id) => path_id,
			Err(_) => return Err(AccessiblePrimitiveConversionError::NoPathId),
		};
		Ok(AccessiblePrimitive { id, sender })
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
/// A struct representing an accessible. To get any information from the cache other than the stored information like role, interfaces, and states, you will need to instantiate an [`atspi::accessible::AccessibleProxy`] or other `*Proxy` type from atspi to query further info.
pub struct CacheItem {
	// The accessible object (within the application)   (so)
	pub object: AccessiblePrimitive,
	// The application (root object(?)    (so)
	pub app: AccessiblePrimitive,
	// The parent object.  (so)
	pub parent: AccessiblePrimitive,
	// The accessbile index in parent.  i
	pub index: i32,
	// Child count of the accessible  i
	pub children: i32,
	// The exposed interfece(s) set.  as
	pub ifaces: InterfaceSet,
	// Accessible role. u
	pub role: Role,
	// The states applicable to the accessible.  au
	pub states: StateSet,
	// The text of the accessible.
	pub text: String,
}
impl TryFrom<atspi::cache::CacheItem> for CacheItem {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(atspi_cache_item: atspi::cache::CacheItem) -> Result<Self, Self::Error> {
		Ok(Self {
			object: atspi_cache_item.object.try_into()?,
			app: atspi_cache_item.app.try_into()?,
			parent: atspi_cache_item.parent.try_into()?,
			index: atspi_cache_item.index,
			children: atspi_cache_item.children,
			ifaces: atspi_cache_item.ifaces,
			role: atspi_cache_item.role,
			states: atspi_cache_item.states,
			text: atspi_cache_item.name,
		})
	}
}

/// The root of the accessible cache.
#[derive(Clone)]
pub struct Cache {
	pub by_id: Arc<RwLock<HashMap<AccessibleId, CacheItem>>>,
}
// clippy wants this
impl Default for Cache {
	fn default() -> Self {
		Self::new()
	}
}

/// Copy all info into a plain CacheItem struct.
/// This is very cheap, and the locking overhead will vastly outstrip making this a non-copy struct.
#[inline]
fn copy_into_cache_item(cache_item_with_handle: &CacheItem) -> CacheItem {
	CacheItem {
		object: cache_item_with_handle.object.clone(),
		parent: cache_item_with_handle.parent.clone(),
		states: cache_item_with_handle.states,
		role: cache_item_with_handle.role,
		app: cache_item_with_handle.app.clone(),
		children: cache_item_with_handle.children,
		ifaces: cache_item_with_handle.ifaces,
		index: cache_item_with_handle.index,
		text: cache_item_with_handle.text.clone(),
	}
}

/// An internal cache used within Odilia.
/// This contains (mostly) all accessibles in the entire accessibility tree, and they are referenced by their IDs.
/// When setting or getting information from the cache, be sure to use the most appropriate function.
/// For example, you would not want to remove individual items using the `remove()` function.
/// You should use the `remove_all()` function to acheive this, since this will only lock the cache mutex once, remove all ids, then refresh the cache.
/// If you are having issues with incorrect or invalid accessibles trying to be accessed, this is code is probably the issue.
/// This implementation is not very efficient, but it is very safe:
/// This is because before inserting, the incomming bucket is cleared (there will never be duplicate accessibles or accessibles at different states stored in the same bucket).
impl Cache {
	/// create a new, fresh cache
	pub fn new() -> Self {
		Self { by_id: Arc::new(RwLock::new(HashMap::new())) }
	}
	/// add a single new item to the cache. Note that this will empty the bucket before inserting the `CacheItem` into the cache (this is so there is never two items with the same ID stored in the cache at the same time).
	pub async fn add(&self, cache_item: CacheItem) {
		let mut cache_writer = self.by_id.write().await;
		cache_writer.insert(cache_item.object.id, cache_item);
	}
	/// remove a single cache item
	pub async fn remove(&self, id: &AccessibleId) {
		let mut cache_writer = self.by_id.write().await;
		cache_writer.remove(id);
	}
	/// get a single item from the cache (note that this copies some integers to a new struct)
	#[allow(dead_code)]
	pub async fn get(&self, id: &AccessibleId) -> Option<CacheItem> {
		let read_handle = self.by_id.read().await;
		read_handle.get(id).cloned()
	}
	/// get a many items from the cache; this only creates one read handle (note that this will copy all data you would like to access)
	#[allow(dead_code)]
	pub async fn get_all(&self, ids: Vec<AccessibleId>) -> Vec<Option<CacheItem>> {
		let read_handle = self.by_id.read().await;
		ids.iter()
			.map(|id| read_handle.get(id).map(copy_into_cache_item))
			.collect()
	}
	/// Bulk add many items to the cache; this only refreshes the cache after adding all items. Note that this will empty the bucket before inserting. Only one accessible should ever be associated with an id.
	pub async fn add_all(&self, cache_items: Vec<CacheItem>) {
		let mut cache_writer = self.by_id.write().await;
		cache_items.into_iter().for_each(|cache_item| {
			cache_writer.insert(cache_item.object.id, cache_item);
		});
	}
	/// Bulk remove all ids in the cache; this only refreshes the cache after removing all items.
	#[allow(dead_code)]
	pub async fn remove_all(&self, ids: Vec<AccessibleId>) {
		let mut cache_writer = self.by_id.write().await;
		ids.iter().for_each(|id| {
			cache_writer.remove(id);
		});
	}

	/// Edit a mutable CacheItem using a function which returns the edited version.
	/// Note: an exclusive lock will be placed for the entire length of the passed function, so don't do any compute in it.
	/// Returns true if the update was successful.
	pub async fn modify_item<F>(&self, id: &AccessibleId, modify: F) -> bool
	where
		F: FnOnce(&mut CacheItem),
	{
		let mut cache_write = self.by_id.write().await;
		let cache_item = match cache_write.get_mut(id) {
			Some(i) => i,
			None => {
				tracing::trace!(
					"The cache has the following items: {:?}",
					cache_write.keys()
				);
				return false;
			}
		};
		modify(cache_item);
		true
	}

	/// get a single item from the cache (note that this copies some integers to a new struct).
	/// If the CacheItem is not found, create one, add it to the cache, and return it.
	pub async fn get_or_create(
		&self,
		accessible: &AccessibleProxy<'_>,
	) -> OdiliaResult<CacheItem> {
		// if the item already exists in the cache, return it
		if let Some(cache_item) = self
			.get(&accessible.id().expect("Could not get ID from accessible path"))
			.await
		{
			return Ok(cache_item);
		}
		// otherwise, build a cache item
		let start = std::time::Instant::now();
		let cache_item = accessible_to_cache_item(accessible).await?;
		let end = std::time::Instant::now();
		let diff = end - start;
		tracing::debug!("Time to create cache item: {:?}", diff);
		// add a clone of it to the cache
		self.add(copy_into_cache_item(&cache_item)).await;
		// return that same cache item
		Ok(cache_item)
	}
}

pub async fn accessible_to_cache_item(accessible: &AccessibleProxy<'_>) -> OdiliaResult<CacheItem> {
	let (app, parent, index, children, ifaces, role, states) = tokio::try_join!(
		accessible.get_application(),
		accessible.parent(),
		accessible.get_index_in_parent(),
		accessible.child_count(),
		accessible.get_interfaces(),
		accessible.get_role(),
		accessible.get_state(),
	)?;
	// if it implements the Text interface
	let text = match accessible.to_text().await {
		// get *all* the text
		Ok(text_iface) => text_iface.get_all_text().await,
		// otherwise, use the name instaed
		Err(_) => Ok(accessible.name().await?),
	}?;
	Ok(CacheItem {
		object: accessible.try_into()?,
		app: app.try_into()?,
		parent: parent.try_into()?,
		index,
		children,
		ifaces,
		role,
		states,
		text,
	})
}
