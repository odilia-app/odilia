use std::{
	collections::HashMap,
	sync::{Arc, Mutex, Weak},
};

use async_trait::async_trait;
use atspi::{
	accessible::{Accessible, AccessibleProxy, RelationType, Role},
	accessible_id::{AccessibleId, HasAccessibleId},
	convertable::Convertable,
	events::GenericEvent,
	signify::Signified,
	text_ext::TextExt,
	InterfaceSet, StateSet,
};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use odilia_common::{
	errors::{AccessiblePrimitiveConversionError, CacheError, OdiliaError},
	result::OdiliaResult,
};
use serde::{Deserialize, Serialize};
use zbus::{
	names::OwnedUniqueName,
	zvariant::{ObjectPath, OwnedObjectPath},
	CacheProperties, ProxyBuilder,
};

type CacheKey = AccessiblePrimitive;
type InnerCache = DashMap<CacheKey, Arc<Mutex<CacheItem>>, FxBuildHasher>;
type ThreadSafeCache = Arc<InnerCache>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
/// A struct which represents the bare minimum of an accessible for purposes of caching.
/// This makes some *possibly eronious* assumptions about what the sender is.
pub struct AccessiblePrimitive {
	/// The accessible ID in /org/a11y/atspi/accessible/XYZ; note that XYZ may be equal to any positive number, 0, "null", or "root".
	pub id: AccessibleId,
	/// Assuming that the sender is ":x.y", this stores the (x,y) portion of this sender.
	pub sender: smartstring::alias::String,
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
		ProxyBuilder::new(conn)
			.path(path)?
			.destination(sender.as_str().to_owned())?
			.cache_properties(CacheProperties::No)
			.build()
			.await
	}
	pub fn from_event<T: GenericEvent>(event: &T) -> Result<Self, OdiliaError> {
		let sender = match event.sender() {
			Ok(Some(s)) => s,
			Ok(None) => {
				return Err(OdiliaError::PrimitiveConversionError(
					AccessiblePrimitiveConversionError::NoSender,
				))
			}
			Err(_) => {
				return Err(OdiliaError::PrimitiveConversionError(
					AccessiblePrimitiveConversionError::ErrSender,
				))
			}
		};
		let path = match event.path() {
			Some(path) => path,
			None => {
				return Err(OdiliaError::PrimitiveConversionError(
					AccessiblePrimitiveConversionError::NoPathId,
				))
			}
		};
		let id: AccessibleId = match path.try_into() {
			Ok(id) => id,
			Err(e) => return Err(OdiliaError::Zvariant(e)),
		};
		Ok(Self { id, sender: sender.as_str().into() })
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
		Ok(AccessiblePrimitive { id: accessible_id, sender: so.0.as_str().into() })
	}
}
impl TryFrom<(String, OwnedObjectPath)> for AccessiblePrimitive {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(so: (String, OwnedObjectPath)) -> Result<AccessiblePrimitive, Self::Error> {
		let accessible_id: AccessibleId = so.1.try_into()?;
		Ok(AccessiblePrimitive { id: accessible_id, sender: so.0.into() })
	}
}
impl TryFrom<(String, AccessibleId)> for AccessiblePrimitive {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(so: (String, AccessibleId)) -> Result<AccessiblePrimitive, Self::Error> {
		Ok(AccessiblePrimitive { id: so.1, sender: so.0.into() })
	}
}
impl<'a> TryFrom<(String, ObjectPath<'a>)> for AccessiblePrimitive {
	type Error = OdiliaError;

	fn try_from(so: (String, ObjectPath<'a>)) -> Result<AccessiblePrimitive, Self::Error> {
		let accessible_id: AccessibleId = so.1.try_into()?;
		Ok(AccessiblePrimitive { id: accessible_id, sender: so.0.into() })
	}
}
impl<'a> TryFrom<&AccessibleProxy<'a>> for AccessiblePrimitive {
	type Error = AccessiblePrimitiveConversionError;

	fn try_from(accessible: &AccessibleProxy<'_>) -> Result<AccessiblePrimitive, Self::Error> {
		let sender = accessible.destination().as_str().into();
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
		let sender = accessible.destination().as_str().into();
		let id = match accessible.id() {
			Ok(path_id) => path_id,
			Err(_) => return Err(AccessiblePrimitiveConversionError::NoPathId),
		};
		Ok(AccessiblePrimitive { id, sender })
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A struct representing an accessible. To get any information from the cache other than the stored information like role, interfaces, and states, you will need to instantiate an [`atspi::accessible::AccessibleProxy`] or other `*Proxy` type from atspi to query further info.
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

	#[serde(skip)]
	pub cache: Weak<Cache>,
}
impl CacheItem {
	pub fn parent_ref(&mut self) -> OdiliaResult<Arc<std::sync::Mutex<CacheItem>>> {
		let parent_ref = Weak::upgrade(&self.parent.item);
		match parent_ref {
			Some(p) => Ok(p),
			None => {
				let cache = strong_cache(&self.cache)?;
				let arc_mut_parent = cache
					.get_ref(&self.parent.key.clone())
					.ok_or(CacheError::NoItem)?;
				self.parent.item = Arc::downgrade(&arc_mut_parent);
				Ok(arc_mut_parent)
			}
		}
	}
	pub async fn from_atspi_event<T: Signified>(
		event: &T,
		cache: Weak<Cache>,
		connection: &zbus::Connection,
	) -> OdiliaResult<Self> {
		let a11y_prim = AccessiblePrimitive::from_event(event)?;
		accessible_to_cache_item(&a11y_prim.into_accessible(connection).await?, cache).await
	}
	pub async fn from_atspi_cache_item(
		atspi_cache_item: atspi::cache::CacheItem,
		cache: Weak<Cache>,
		connection: &zbus::Connection,
	) -> OdiliaResult<Self> {
		let children: Vec<CacheRef> =
			AccessiblePrimitive::try_from(atspi_cache_item.object.clone())?
				.into_accessible(connection)
				.await?
				.get_children()
				.await?
				.into_iter()
				.map(|child_object_pair| {
					Ok(CacheRef::new(child_object_pair.try_into()?))
				})
				.collect::<Result<_, AccessiblePrimitiveConversionError>>()?;
		Ok(Self {
			object: atspi_cache_item.object.try_into()?,
			app: atspi_cache_item.app.try_into()?,
			parent: CacheRef::new(atspi_cache_item.parent.try_into()?),
			index: atspi_cache_item.index,
			children_num: atspi_cache_item.children,
			interfaces: atspi_cache_item.ifaces,
			role: atspi_cache_item.role,
			states: atspi_cache_item.states,
			text: atspi_cache_item.name,
			cache,
			children,
		})
	}
	// Same as [`Accessible::get_children`], just offered as a non-async version.
	pub fn get_children(&self) -> OdiliaResult<Vec<Self>> {
		let derefed_cache: Arc<Cache> = strong_cache(&self.cache)?;
		let children = self
			.children
			.iter()
			.map(|child_ref| {
				child_ref
					.clone_inner()
					.or_else(|| derefed_cache.get(&child_ref.key))
					.ok_or(CacheError::NoItem)
			})
			.collect::<Result<Vec<_>, _>>()?;
		Ok(children)
	}
}

/// A composition of an accessible ID and (possibly) a reference
/// to its `CacheItem`, if the item has not been dropped from the cache yet.
/// TODO if desirable, we could make one direction strong references (e.g. have
/// the parent be an Arc, xor have the children be Arcs). Might even be possible to have both.
/// BUT - is it even desirable to keep an item pinned in an Arc from its
/// releatives after it has been removed from the cache?
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheRef {
	pub key: CacheKey,
	#[serde(skip)]
	item: Weak<Mutex<CacheItem>>,
}

impl CacheRef {
	pub fn new(key: AccessiblePrimitive) -> Self {
		Self { key, item: Weak::new() }
	}

	pub fn clone_inner(&self) -> Option<CacheItem> {
		self.item.upgrade().as_ref().map(clone_arc_mutex)
	}
}

impl From<AccessiblePrimitive> for CacheRef {
	fn from(value: AccessiblePrimitive) -> Self {
		Self::new(value)
	}
}

#[inline]
async fn as_accessible(cache_item: &CacheItem) -> OdiliaResult<AccessibleProxy<'_>> {
	let cache = strong_cache(&cache_item.cache)?;
	Ok(cache_item.object.clone().into_accessible(&cache.connection).await?)
}

#[inline]
fn strong_cache(weak_cache: &Weak<Cache>) -> OdiliaResult<Arc<Cache>> {
	Weak::upgrade(weak_cache).ok_or(OdiliaError::Cache(CacheError::NotAvailable))
}

#[async_trait]
impl Accessible for CacheItem {
	type Error = OdiliaError;

	async fn get_application(&self) -> Result<Self, Self::Error> {
		let derefed_cache: Arc<Cache> = strong_cache(&self.cache)?;
		derefed_cache.get(&self.app).ok_or(CacheError::NoItem.into())
	}
	async fn parent(&self) -> Result<Self, Self::Error> {
		let parent_item = self
			.parent
			.clone_inner()
			.or_else(|| self.cache.upgrade()?.get(&self.parent.key));
		parent_item.ok_or(CacheError::NoItem.into())
	}
	async fn get_children(&self) -> Result<Vec<Self>, Self::Error> {
		self.get_children()
	}
	async fn child_count(&self) -> Result<i32, Self::Error> {
		Ok(self.children_num)
	}
	async fn get_index_in_parent(&self) -> Result<i32, Self::Error> {
		Ok(self.index)
	}
	async fn get_role(&self) -> Result<Role, Self::Error> {
		Ok(self.role)
	}
	async fn get_interfaces(&self) -> Result<InterfaceSet, Self::Error> {
		Ok(self.interfaces)
	}
	async fn get_attributes(&self) -> Result<HashMap<String, String>, Self::Error> {
		Ok(as_accessible(self).await?.get_attributes().await?)
	}
	async fn name(&self) -> Result<String, Self::Error> {
		Ok(as_accessible(self).await?.name().await?)
	}
	async fn locale(&self) -> Result<String, Self::Error> {
		Ok(as_accessible(self).await?.locale().await?)
	}
	async fn description(&self) -> Result<String, Self::Error> {
		Ok(as_accessible(self).await?.description().await?)
	}
	async fn get_relation_set(&self) -> Result<Vec<(RelationType, Vec<Self>)>, Self::Error> {
		let cache = strong_cache(&self.cache)?;
		as_accessible(self)
			.await?
			.get_relation_set()
			.await?
			.into_iter()
			.map(|(relation, object_pairs)| {
				(
					relation,
					object_pairs
						.into_iter()
						.map(|object_pair| {
							cache.get(&object_pair.try_into()?).ok_or(
								OdiliaError::Cache(
									CacheError::NoItem,
								),
							)
						})
						.collect::<Result<Vec<Self>, OdiliaError>>(),
				)
			})
			.map(|(relation, result_selfs)| Ok((relation, result_selfs?)))
			.collect::<Result<Vec<(RelationType, Vec<Self>)>, OdiliaError>>()
	}
	async fn get_role_name(&self) -> Result<String, Self::Error> {
		Ok(as_accessible(self).await?.get_role_name().await?)
	}
	async fn get_state(&self) -> Result<StateSet, Self::Error> {
		Ok(self.states)
	}
	async fn get_child_at_index(&self, idx: i32) -> Result<Self, Self::Error> {
		<Self as Accessible>::get_children(self)
			.await?
			.get(idx as usize)
			.ok_or(CacheError::NoItem.into())
			.cloned()
	}
	async fn get_localized_role_name(&self) -> Result<String, Self::Error> {
		Ok(as_accessible(self).await?.get_localized_role_name().await?)
	}
	async fn accessible_id(&self) -> Result<AccessibleId, Self::Error> {
		Ok(self.object.id)
	}
}

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

// N.B.: we are using std Mutexes internally here, within the cache hashmap
// entries. When adding async methods, take care not to hold these mutexes
// across .await points.
impl Cache {
	/// create a new, fresh cache
	pub fn new(conn: zbus::Connection) -> Self {
		Self { by_id: Arc::new(DashMap::default()), connection: conn }
	}
	/// add a single new item to the cache. Note that this will empty the bucket
	/// before inserting the `CacheItem` into the cache (this is so there is
	/// never two items with the same ID stored in the cache at the same time).
	pub fn add(&self, cache_item: CacheItem) {
		let id = cache_item.object.clone();
		self.add_ref(id, Arc::new(Mutex::new(cache_item)));
	}

	pub fn add_ref(&self, id: CacheKey, cache_item: Arc<Mutex<CacheItem>>) {
		self.by_id.insert(id, Arc::clone(&cache_item));
		Self::populate_references(&self.by_id, cache_item);
	}

	/// Remove a single cache item
	pub fn remove(&self, id: &CacheKey) {
		self.by_id.remove(id);
	}
	/// Get a single item (mutable via lock) from the cache.
	// For now this is kept private, as it would be easy to naively deadlock if
	// someone does a chain of `get_ref`s on parent->child->parent, etc.
	pub fn get_ref(&self, id: &CacheKey) -> Option<Arc<Mutex<CacheItem>>> {
		self.by_id.get(id).as_deref().cloned()
	}

	/// Get a single item from the cache.
	///
	/// This will allow you to get the item without holding any locks to it,
	/// at the cost of (1) a clone and (2) no guarantees that the data is kept up-to-date.
	pub fn get(&self, id: &CacheKey) -> Option<CacheItem> {
		self.by_id.get(id).as_deref().map(clone_arc_mutex)
	}

	/// get a many items from the cache; this only creates one read handle (note that this will copy all data you would like to access)
	#[allow(dead_code)]
	pub fn get_all(&self, ids: &[CacheKey]) -> Vec<Option<CacheItem>> {
		ids.iter().map(|id| self.get(id)).collect()
	}

	/// Bulk add many items to the cache; only one accessible should ever be
	/// associated with an id.
	pub fn add_all(&self, cache_items: Vec<CacheItem>) {
		cache_items
			.into_iter()
			.map(|cache_item| {
				let id = cache_item.object.clone();
				let arc = Arc::new(Mutex::new(cache_item));
				self.by_id.insert(id, Arc::clone(&arc));
				arc
			})
			.collect::<Vec<_>>() // Insert all items before populating
			.into_iter()
			.for_each(|item| {
				Self::populate_references(&self.by_id, item);
			});
	}
	/// Bulk remove all ids in the cache; this only refreshes the cache after removing all items.
	#[allow(dead_code)]
	pub fn remove_all(&self, ids: Vec<CacheKey>) {
		ids.iter().for_each(|id| {
			self.by_id.remove(id);
		});
	}

	/// Edit a mutable CacheItem. Returns true if the update was successful.
	///
	/// Note: an exclusive lock for the given cache item will be placed for the
	/// entire length of the passed function, so try to avoid any compute in it.
	pub fn modify_item<F>(&self, id: &CacheKey, modify: F) -> bool
	where
		F: FnOnce(&mut CacheItem),
	{
		// I wonder if `get_mut` vs `get` makes any difference here? I suppose
		// it will just rely on the dashmap write access vs mutex lock access.
		// Let's default to the fairness of the mutex.
		let entry = match self.by_id.get(id) {
			// Drop the dashmap reference immediately, at the expense of an Arc clone.
			Some(i) => (*i).clone(),
			None => {
				tracing::trace!(
					"The cache does not contain the requested item: {:?}",
					id
				);
				return false;
			}
		};
		let mut cache_item = entry.lock().unwrap();
		modify(&mut cache_item);
		true
	}

	/// Get a single item from the cache (note that this copies some integers to a new struct).
	/// If the CacheItem is not found, create one, add it to the cache, and return it.
	pub async fn get_or_create(
		&self,
		accessible: &AccessibleProxy<'_>,
		cache: Weak<Self>,
	) -> OdiliaResult<CacheItem> {
		// if the item already exists in the cache, return it
		let primitive = accessible.try_into()?;
		if let Some(cache_item) = self.get(&primitive) {
			return Ok(cache_item);
		}
		// otherwise, build a cache item
		let start = std::time::Instant::now();
		let cache_item = accessible_to_cache_item(accessible, cache).await?;
		let end = std::time::Instant::now();
		let diff = end - start;
		tracing::debug!("Time to create cache item: {:?}", diff);
		// add a clone of it to the cache
		self.add(cache_item.clone());
		// return that same cache item
		Ok(cache_item)
	}

	fn populate_references(cache: &ThreadSafeCache, item_ref: Arc<Mutex<CacheItem>>) {
		let item_wk_ref = Arc::downgrade(&item_ref);

		let mut item = item_ref.lock().unwrap();
		let item_key = item.object.clone();
		let parent_key = item.parent.key.clone();
		let ix_opt = usize::try_from(item.index)
			.map_err(|_| {
				tracing::debug!("Item has invalid index: {}", item.index);
			})
			.ok();

		// Update this item's children references
		let mut child_arcs = Vec::with_capacity(item.children.len());
		for child_ref in item.children.iter_mut() {
			if let Some(child_arc) = cache.get(&child_ref.key).as_deref() {
				child_ref.item = Arc::downgrade(child_arc);
				child_arcs.push(Arc::clone(child_arc));
			}
		}
		// Update this item's parent reference
		let parent_ref_opt = cache.get(&parent_key).as_deref().map(|parent_ref| {
			item.parent.item = Arc::downgrade(parent_ref);
			Arc::clone(parent_ref)
		});

		// Drop item while we update parent/children to avoid deadlocking
		drop(item);

		// Update children to point to this item as parent
		for child_arc in child_arcs {
			let mut child = child_arc.lock().unwrap();
			child.parent.item = Weak::clone(&item_wk_ref);
		}

		// Populate parent's ref to this item as child
		if let Some((parent_ref, ix)) = parent_ref_opt.zip(ix_opt) {
			let mut parent = parent_ref.lock().unwrap();
			match parent.children.get_mut(ix).as_mut() {
				Some(i) if i.key == item_key => {
					i.item = Weak::clone(&item_wk_ref);
				}
				Some(_) => {
					tracing::debug!(
						"Parent cache item mismatched child at index {}",
						ix
					);
				}
				None => {
					tracing::debug!(
						"Parent cache item missing child at index {}",
						ix
					);
				}
			}
		}
	}
}

pub async fn accessible_to_cache_item(
	accessible: &AccessibleProxy<'_>,
	cache: Weak<Cache>,
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
		Ok(text_iface) => text_iface.get_all_text().await,
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
		children: children
			.into_iter()
			.map(|k| Ok(CacheRef::new(k.try_into()?)))
			.collect::<Result<_, AccessiblePrimitiveConversionError>>()?,
		cache,
	})
}

pub fn clone_arc_mutex<T: Clone>(arc: &Arc<Mutex<T>>) -> T {
	arc.lock().unwrap().clone()
}
