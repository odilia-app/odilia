#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	unsafe_code
)]

pub mod cache_item_ext;
use cache_item_ext::{accessible_to_cache_item};

use std::{
	sync::{Arc, Weak},
};
use parking_lot::RwLock;

use async_trait::async_trait;
use atspi_common::{
	GenericEvent
};
use atspi_proxies::{
	accessible::{AccessibleProxy},
	text::{TextProxy},
};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use odilia_common::{
	errors::{AccessiblePrimitiveConversionError, OdiliaError},
	OdiliaResult,
	cache::{AccessiblePrimitive, CacheItem, CacheKey, ThreadSafeCache, CacheRef},
};
use zbus::{
	zvariant::ObjectPath,
	CacheProperties, ProxyBuilder,
};

#[async_trait]
pub trait AccessiblePrimitiveHostExt {
	async fn into_accessible<'a>(self, conn: &zbus::Connection) -> zbus::Result<AccessibleProxy<'a>>;
	async fn into_text<'a>(self, conn: &zbus::Connection) -> zbus::Result<TextProxy<'a>>;
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
	pub by_id: ThreadSafeCache,
	pub connection: zbus::Connection,
}

// N.B.: we are using std RwLockes internally here, within the cache hashmap
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
	pub fn add(&self, cache_item: CacheItem) -> OdiliaResult<()> {
		let id = cache_item.object.clone();
		self.add_ref(id, &Arc::new(RwLock::new(cache_item)))
	}

	/// Add an item via a reference instead of creating the reference.
	/// # Errors
	/// Can error if [`Cache::populate_references`] errors. The insertion is guarenteed to succeed.
	pub fn add_ref(
		&self,
		id: CacheKey,
		cache_item: &Arc<RwLock<CacheItem>>,
	) -> OdiliaResult<()> {
		self.by_id.insert(id, Arc::clone(cache_item));
		Self::populate_references(&self.by_id, cache_item)
	}

	/// Remove a single cache item. This function can not fail.
	pub fn remove(&self, id: &CacheKey) {
		self.by_id.remove(id);
	}

	/// Get a single item from the cache, this only gets a reference to an item, not the item itself.
	/// You will need to either get a read or a write lock on any item returned from this function.
	/// It also may return `None` if a value is not matched to the key.
	#[must_use]
	pub fn get_ref(&self, id: &CacheKey) -> Option<Arc<RwLock<CacheItem>>> {
		self.by_id.get(id).as_deref().cloned()
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
	pub fn get(&self, id: &CacheKey) -> Option<CacheItem> {
		Some(self.by_id.get(id).as_deref()?.read().clone())
	}

	/// get a many items from the cache; this only creates one read handle (note that this will copy all data you would like to access)
	#[must_use]
	pub fn get_all(&self, ids: &[CacheKey]) -> Vec<Option<CacheItem>> {
		ids.iter().map(|id| self.get(id)).collect()
	}

	/// Bulk add many items to the cache; only one accessible should ever be
	/// associated with an id.
	/// # Errors
	/// An `Err(_)` variant may be returned if the [`Cache::populate_references`] function fails.
	pub fn add_all(&self, cache_items: Vec<CacheItem>) -> OdiliaResult<()> {
		cache_items
			.into_iter()
			.map(|cache_item| {
				let id = cache_item.object.clone();
				let arc = Arc::new(RwLock::new(cache_item));
				self.by_id.insert(id, Arc::clone(&arc));
				arc
			})
			.collect::<Vec<_>>() // Insert all items before populating
			.into_iter()
			.try_for_each(|item| Self::populate_references(&self.by_id, &item))
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
	pub fn modify_item<F>(&self, id: &CacheKey, modify: F) -> OdiliaResult<bool>
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
		let mut cache_item = entry.write();
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
		if let Some(cache_item) = self.get(&primitive) {
			return Ok(cache_item);
		}
		// otherwise, build a cache item
		let start = std::time::Instant::now();
		let cache_item = accessible_to_cache_item(accessible).await?;
		let end = std::time::Instant::now();
		let diff = end - start;
		tracing::debug!("Time to create cache item: {:?}", diff);
		// add a clone of it to the cache
		self.add(cache_item.clone())?;
		// return that same cache item
		Ok(cache_item)
	}

	/// Populate children and parent references given a cache and an `Arc<RwLock<CacheItem>>`.
	/// This will unlock the `RwLock<_>`, update the references for children and parents, then go to the parent and children and do the same: update the parent for the children, then update the children referneces for the parent.
	/// # Errors
	/// If any references, either the ones passed in through the `item_ref` parameter, any children references, or the parent reference are unable to be unlocked, an `Err(_)` variant will be returned.
	/// Technically it can also fail if the index of the `item_ref` in its parent exceeds `usize` on the given platform, but this is highly improbable.
	pub fn populate_references(
		cache: &ThreadSafeCache,
		item_ref: &Arc<RwLock<CacheItem>>,
	) -> Result<(), OdiliaError> {
		let item_wk_ref = Arc::downgrade(item_ref);

		let mut item = item_ref.write();
		let item_key = item.object.clone();

		let parent_key = item.parent.key.clone();
		let parent_ref_opt = cache.get(&parent_key);

		// Update this item's parent reference
		let ix_opt = usize::try_from(item.index).ok();

		// Update this item's children references
		for child_ref in &mut item.children {
			if let Some(child_arc) = cache.get(&child_ref.key).as_ref() {
				child_ref.item = Arc::downgrade(child_arc);
				child_arc.write().parent.item = Weak::clone(&item_wk_ref);
			}
		}

		// TODO: Should there be errors for the non let bindings?
		if let Some(parent_ref) = parent_ref_opt {
			item.parent.item = Arc::downgrade(&parent_ref);
			if let Some(ix) = ix_opt {
				if let Some(cache_ref) = parent_ref
					.write()
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

