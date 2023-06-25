#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	missing_docs,
	unsafe_code
)]

//! Odilia's Cache
//!
//! The implementation of Odilia's caching system.
//! If any modifications need to be made in relation to loading and storing various bits of information retrieved through the [`atspi`] crate, it will be here.

mod types;
pub use types::*;
mod cache_ref;
pub use cache_ref::*;
mod cache_item;
pub use cache_item::*;

use std::{
	sync::{Arc, Weak},
};
use tokio::sync::Mutex;

use async_trait::async_trait;
use atspi_common::{
	GenericEvent
};
use atspi_client::convertable::Convertable;
use atspi_proxies::{
	accessible::{AccessibleProxy},
	text::{TextProxy},
};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use odilia_common::{
	errors::{AccessiblePrimitiveConversionError, OdiliaError, CacheError},
	OdiliaResult,
	cache::{AccessiblePrimitive, CacheKey},
};
use zbus::{
	zvariant::ObjectPath,
	CacheProperties, ProxyBuilder,
};
use futures::future::join_all;

/// A host extention of the [`AccessiblePrimitive`] type.
/// This enables the use of additional conversion functions for the cache.
#[async_trait]
pub trait AccessiblePrimitiveHostExt {
	/// Convert into an [`atspi_proxies::accessible::AccessibleProxy`].
	/// This is used when needing to query AT-SPI for new data.
	async fn into_accessible<'a>(self, conn: &zbus::Connection) -> zbus::Result<AccessibleProxy<'a>>;
	/// Conver into an [`atspi_proxies::text::TextProxy`].
	/// This is used when needing to query AT-SPI for new data on its text interface.
	async fn into_text<'a>(self, conn: &zbus::Connection) -> zbus::Result<TextProxy<'a>>;
	/// Take any event (which is required to implement [`atspi_common::events::GenericEvent`]) and convert it into an [`AccessiblePrimitive`].
	fn from_event<'a, T: GenericEvent<'a>>(event: &T) -> Result<Self, AccessiblePrimitiveConversionError> where Self: Sized;
}

#[async_trait]
impl AccessiblePrimitiveHostExt for AccessiblePrimitive {
	/// Convert into an [`atspi_proxies::accessible::AccessibleProxy`]. Must be async because the creation of an async proxy requires async itself.
	/// # Errors
	/// Will return a [`zbus::Error`] in the case of an invalid destination, path, or failure to create a `Proxy` from those properties.
	async fn into_accessible<'a>(
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
	/// Convert into an [`atspi_proxies::text::TextProxy`]. Must be async because the creation of an async proxy requires async itself.
	/// # Errors
	/// Will return a [`zbus::Error`] in the case of an invalid destination, path, or failure to create a `Proxy` from those properties.
	async fn into_text<'a>(self, conn: &zbus::Connection) -> zbus::Result<TextProxy<'a>> {
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
	/// Turns any `atspi::event` type into an `AccessiblePrimtive`, the basic type which is used for keys in the cache.
	/// # Errors
	/// The errors are self-explanitory variants of the [`odilia_common::errors::AccessiblePrimitiveConversionError`].
	fn from_event<'a, T: GenericEvent<'a>>(
		event: &T,
	) -> Result<Self, AccessiblePrimitiveConversionError> {
		let sender = event.sender();
		//.map_err(|_| AccessiblePrimitiveConversionError::ErrSender)?
		//.ok_or(AccessiblePrimitiveConversionError::NoSender)?;
		let path = event.path(); //.ok_or(AccessiblePrimitiveConversionError::NoPathId)?;
		let id = path.to_string();
		Ok(Self { id, sender: sender.as_str().into() })
	}
}

//#[inline]
//fn strong_cache(weak_cache: &Weak<Cache>) -> OdiliaResult<Arc<Cache>> {
//	Weak::upgrade(weak_cache).ok_or(OdiliaError::Cache(CacheError::NotAvailable))
//}

/// An internal cache used within Odilia.
///
/// This contains (mostly) all accessibles in the entire accessibility tree, and
/// they are referenced by their IDs. If you are having issues with incorrect or
/// invalid accessibles trying to be accessed, this is code is probably the issue.
#[derive(Clone, Debug)]
pub struct Cache {
	/// A cache lookup by id.
	pub by_id: ThreadSafeCache,
	/// An active connection to AT-SPI.
	/// This is used to query information that the cache stores directly.
	/// Generally speaking, only the cache interfaces with AT-SPI directly.
	/// If Odilia really needs to query something through AT-SPI, the cache should probably store it instead.
	pub connection: zbus::Connection,
}

// N.B.: we are using std Mutexes internally here, within the cache hashmap
// entries. When adding async methods, take care not to hold these mutexes
// across .await points.
impl Cache {
	/// create a new, fresh cache
	#[must_use]
	pub fn new(conn: zbus::Connection) -> Self {
		Self {
			by_id: Arc::new(DashMap::with_capacity_and_hasher(
				10_000,
				FxBuildHasher::default(),
			)),
			connection: conn,
		}
	}
	/// add a single new item to the cache. Note that this will empty the bucket
	/// before inserting the `CacheItem` into the cache (this is so there is
	/// never two items with the same ID stored in the cache at the same time).
	/// # Errors
	/// Fails if the internal call to [`Self::add_ref`] fails.
	pub async fn add(&self, cache_item: CacheItem) -> OdiliaResult<()> {
		let id = cache_item.object.clone();
		self.add_ref(id, &Arc::new(Mutex::new(cache_item))).await
	}

	/// Add an item via a reference instead of creating the reference.
	/// # Errors
	/// Can error if [`Cache::populate_references`] errors. The insertion is guarenteed to succeed.
	pub async fn add_ref(
		&self,
		id: CacheKey,
		cache_item: &Arc<Mutex<CacheItem>>,
	) -> OdiliaResult<()> {
		self.by_id.insert(id, Arc::clone(cache_item));
		Self::populate_references(&self.by_id, cache_item).await
	}

	/// Remove a single cache item. This function can not fail.
	pub fn remove(&self, id: &CacheKey) {
		self.by_id.remove(id);
	}

	/// Get a single item from the cache, this only gets a reference to an item, not the item itself.
	/// You will need to either get a read or a write lock on any item returned from this function.
	/// It also may return `None` if a value is not matched to the key.
	#[must_use]
	pub fn get_ref(&self, id: &CacheKey) -> Option<Arc<Mutex<CacheItem>>> {
		self.by_id.get(id).as_deref().cloned()
	}

	/// Get a single item from the cache,
	/// but use the CacheRef, and if that doesn't have a live reference, then use the key.
	pub async fn get_from_ref(&self, cache_ref: &CacheRef) -> Option<Arc<Mutex<CacheItem>>> {
		match Weak::upgrade(&cache_ref.item) {
			None => {
				self.get_ref(&cache_ref.key)
			},
			Some(arc) => {
				Some(arc)
			}
		}
	}

	/// Get a single item from the crate, like [`get_ref`], but gives you both an id and a *possible* reference.
	#[must_use]
	pub fn get_key(&self, id: &CacheKey) -> CacheRef {
		CacheRef {
			key: id.clone(),
			item: self.get_ref(id)
				.map(|arc| Arc::downgrade(&arc))
				.unwrap_or(Weak::new()),
		}
	}

	/// Get a single item from the cache.
	///
	/// This will allow you to get the item without holding any locks to it,
	/// at the cost of (1) a clone and (2) no guarantees that the data is kept up-to-date.
	#[must_use]
	pub async fn get(&self, id: &CacheKey) -> Option<CacheItem> {
		Some(self.by_id.get(id).as_deref()?.lock().await.clone())
	}

	/// Get a single item from the cache.
	///
	/// This will allow you to get the item without holding any locks to it,
	/// at the cost of (1) a clone and (2) no guarantees that the data is kept up-to-date.
	#[must_use]
	pub async fn get_from<'a, T: 'a>(&self, to_key: &'a T) -> Result<CacheItem, OdiliaError> 
		where AccessiblePrimitive: TryFrom<&'a T>,
					OdiliaError: From<<AccessiblePrimitive as TryFrom<&'a T>>::Error> {
		let key = AccessiblePrimitive::try_from(to_key)?;
		Ok(self.by_id.get(&key)
			.as_deref()
			.ok_or(CacheError::NoItem)?
			.lock()
			.await
			.clone())
	}

	/// get a many items from the cache; this only creates one read handle (note that this will copy all data you would like to access)
	#[must_use]
	pub async fn get_all(&self, ids: &[CacheKey]) -> Vec<Option<CacheItem>> {
		join_all(ids.iter().map(|id| self.get(id))).await
	}

	/// Bulk add many items to the cache; only one accessible should ever be
	/// associated with an id.
	/// # Errors
	/// An `Err(_)` variant may be returned if the [`Cache::populate_references`] function fails.
	pub async fn add_all(&self, cache_items: Vec<CacheItem>) -> OdiliaResult<()> {
		let items = cache_items
			.into_iter()
			.map(|cache_item| {
				let id = cache_item.object.clone();
				let arc = Arc::new(Mutex::new(cache_item));
				self.by_id.insert(id, Arc::clone(&arc));
				arc
			})
			.collect::<Vec<_>>() // Insert all items before populating
			.into_iter();
		for item in items {
			Self::populate_references(&self.by_id, &item).await?;
		}
		Ok(())
	}
	/// Bulk remove all ids in the cache; this only refreshes the cache after removing all items.
	pub fn remove_all(&self, ids: &Vec<CacheKey>) {
		for id in ids {
			self.by_id.remove(id);
		}
	}

	/// Edit a mutable `CacheItem`. Returns true if the update was successful.
	///
	/// Note: an exclusive lock for the given cache item will be placed for the
	/// entire length of the passed function, so try to avoid any compute in it.
	///
	/// # Errors
	///
	/// An [`odilia_common::errors::OdiliaError::PoisoningError`] may be returned if a write lock can not be aquired on the `CacheItem` being modified.
	pub async fn modify_item<F>(&self, id: &CacheKey, modify: F) -> OdiliaResult<bool>
	where
		F: FnOnce(&mut CacheItem),
	{
		// I wonder if `get_mut` vs `get` makes any difference here? I suppose
		// it will just rely on the dashmap write access vs mutex lock access.
		// Let's default to the fairness of the mutex.
		let entry = if let Some(i) = self.by_id.get(id) {
			// Drop the dashmap reference immediately, at the expense of an Arc clone.
			(*i).clone()
		} else {
			tracing::trace!("The cache does not contain the requested item: {:?}", id);
			return Ok(false);
		};
		let mut cache_item = entry.lock().await;
		modify(&mut cache_item);
		Ok(true)
	}

	/// Get a single item from the cache (note that this copies some integers to a new struct).
	/// If the `CacheItem` is not found, create one, add it to the cache, and return it.
	/// # Errors
	/// The function will return an error if:
	/// 1. The `accessible` can not be turned into an `AccessiblePrimitive`. This should never happen, but is technically possible.
	/// 2. The [`Self::add`] function fails.
	/// 3. The [`accessible_to_cache_item`] function fails.
	///
	/// NOTE: This is a very expensive function to call, and should only be done within long-running async tasks that do not block other items from running.
	pub async fn get_or_create(
		&self,
		accessible: &AccessibleProxy<'_>,
		_cache: Weak<Self>,
	) -> OdiliaResult<CacheItem> {
		// if the item already exists in the cache, return it
		let primitive = accessible.try_into()?;
		if let Some(cache_item) = self.get(&primitive).await {
			return Ok(cache_item);
		}
		// otherwise, build a cache item
		let start = std::time::Instant::now();
		let cache_item = accessible_to_cache_item(accessible).await?;
		let end = std::time::Instant::now();
		let diff = end - start;
		tracing::debug!("Time to create cache item: {:?}", diff);
		// add a clone of it to the cache
		self.add(cache_item.clone()).await?;
		// return that same cache item
		Ok(cache_item)
	}

	/// Populate children and parent references given a cache and an `Arc<Mutex<CacheItem>>`.
	/// This will unlock the `Mutex<_>`, update the references for children and parents, then go to the parent and children and do the same: update the parent for the children, then update the children referneces for the parent.
	/// # Errors
	/// If any references, either the ones passed in through the `item_ref` parameter, any children references, or the parent reference are unable to be unlocked, an `Err(_)` variant will be returned.
	/// Technically it can also fail if the index of the `item_ref` in its parent exceeds `usize` on the given platform, but this is highly improbable.
	pub async fn populate_references(
		cache: &ThreadSafeCache,
		item_ref: &Arc<Mutex<CacheItem>>,
	) -> Result<(), OdiliaError> {
		let item_wk_ref = Arc::downgrade(item_ref);

		let mut item = item_ref.lock().await;
		let item_key = item.object.clone();

		let parent_key = item.parent.key.clone();
		let parent_ref_opt = cache.get(&parent_key);

		// Update this item's parent reference
		let ix_opt = usize::try_from(item.index).ok();

		// Update this item's children references
		for child_ref in &mut item.children {
			if let Some(child_arc) = cache.get(&child_ref.key).as_ref() {
				child_ref.item = Arc::downgrade(child_arc);
				child_arc.lock().await.parent.item = Weak::clone(&item_wk_ref);
			}
		}

		// TODO: Should there be errors for the non let bindings?
		if let Some(parent_ref) = parent_ref_opt {
			item.parent.item = Arc::downgrade(&parent_ref);
			if let Some(ix) = ix_opt {
				if let Some(cache_ref) = parent_ref
					.lock()
					.await
					.children
					.get_mut(ix)
					.filter(|i| i.key == item_key)
				{
					cache_ref.item = Weak::clone(&item_wk_ref);
				}
			}
		}
		Ok(())
	}
}

/// Convert an [`atspi_proxies::accessible::AccessibleProxy`] into a [`crate::CacheItem`].
/// This runs a bunch of long-awaiting code and can take quite some time; use this sparingly.
/// This takes most properties and some function calls through the `AccessibleProxy` structure and generates a new `CacheItem`, which will be written to cache before being sent back.
///
/// # Errors
///
/// Will return an `Err(_)` variant when:
///
/// 1. The `cache` parameter does not reference an active cache once the `Weak` is upgraded to an `Option<Arc<_>>`.
/// 2. Any of the function calls on the `accessible` fail.
/// 3. Any `(String, OwnedObjectPath) -> AccessiblePrimitive` conversions fail. This *should* never happen, but technically it is possible.
pub async fn accessible_to_cache_item(
	accessible: &AccessibleProxy<'_>,
) -> OdiliaResult<CacheItem> {
	let (app, parent, index, children_num, interfaces, role, states, children) = tokio::try_join!(
		accessible.get_application(),
		accessible.parent(),
		accessible.get_index_in_parent(),
		accessible.child_count(),
		accessible.get_interfaces(),
		accessible.get_role(),
		accessible.get_state(),
		accessible.get_children(),
	)?;
	// if it implements the Text interface
	let text = match accessible.to_text().await {
		// get *all* the text
		Ok(text_iface) => {
			// yes this is actually how you need to do it.
			let len = text_iface.character_count().await?;
			text_iface.get_text(0, len).await
		},
		// otherwise, use the name instaed
		Err(_) => Ok(accessible.name().await?),
	}?;
	Ok(CacheItem {
		object: accessible.try_into()?,
		app: app.try_into()?,
		parent: CacheRef::new(parent.try_into()?),
		index,
		children_num,
		interfaces,
		role,
		states,
		text,
		children: children.into_iter().map(|k| CacheRef::new(k.into())).collect(),
	})
}
