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
use std::{collections::HashMap, sync::Arc, sync::Weak, time::Duration};
use std::{
	sync::{mpsc, Condvar, Mutex},
	thread,
};
use zbus::{
	names::OwnedUniqueName,
	zvariant::{ObjectPath, OwnedObjectPath},
	CacheProperties, ProxyBuilder,
};

type CacheKey = AccessiblePrimitive;
type InnerCache = DashMap<CacheKey, Arc<Mutex<CacheItem>>, FxBuildHasher>;
// TODO we currently pass around an Arc<Cache>, which results in
// Arc<Arc<InnerCache>>; we should reduce this to one level
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
	pub children: Vec<AccessiblePrimitive>,

	#[serde(skip)]
	pub cache: Weak<Cache>,
}
impl CacheItem {
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
		let children_primitives: Vec<AccessiblePrimitive> =
			AccessiblePrimitive::try_from(atspi_cache_item.object.clone())?
				.into_accessible(connection)
				.await?
				.get_children()
				.await?
				.into_iter()
				.map(|child_object_pair| child_object_pair.try_into())
				.collect::<Result<
					Vec<AccessiblePrimitive>,
					AccessiblePrimitiveConversionError,
				>>()?;
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
			children: children_primitives,
		})
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
		let derefed_cache: Arc<Cache> = strong_cache(&self.cache)?;
		derefed_cache
			.get_all(&self.children)
			.into_iter()
			.map(|child| child.ok_or(CacheError::NoItem.into()))
			.collect()
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
		self.get_children()
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
//
// Note: When the cache is created, we start a worker thread that populates
// references to parents and children. This is all pretty non-async, especially
// with dashmap as the backing source, so we're using std thread and sync
// primitives rather than tokio.
//
// Other option would just be to shoot off short-lived tokio tasks rather than
// queue tasks to OS thread. (Or tokio blocking tasks, since these are mostly
// CPU-bound.)
//
// Yet another variant: drop the cond var complexity and just do the work; it
// may not be necessary.
#[derive(Clone, Debug)]
pub struct Cache {
	by_id: ThreadSafeCache,
	connection: zbus::Connection,
	in_use: Arc<(Mutex<usize>, Condvar)>,
	// I don't really want this to be bounded but otherwise we can't impl
	// Accessible, as Cache wouldn't be Sync.
	ref_task_sender: mpsc::SyncSender<Arc<Mutex<CacheItem>>>,
	ref_shutdown_sender: Arc<Mutex<bool>>,
}

// N.B.: we are using std Mutexes internally here, within the cache hashmap
// entries. When adding async methods, take care not to hold these mutexes
// across .await points.
impl Cache {
	/// create a new, fresh cache
	pub fn new(connection: zbus::Connection) -> Self {
		let (ref_task_sender, task_recv) = mpsc::sync_channel(100000000); // :/
		let in_use = Arc::new((Mutex::new(0), Condvar::new()));
		let in_use_2 = Arc::clone(&in_use);
		let by_id = Arc::new(DashMap::default());
		let by_id_2 = Arc::clone(&by_id);
		let ref_shutdown_sender = Arc::new(Mutex::new(false));
		let ref_shutdown_sender_2 = Arc::clone(&ref_shutdown_sender);
		thread::spawn(move || {
			populate_references(in_use_2, by_id_2, task_recv, ref_shutdown_sender_2)
		});
		Self { by_id, connection, in_use, ref_task_sender, ref_shutdown_sender }
	}
	/// add a single new item to the cache. Note that this will empty the bucket
	/// before inserting the `CacheItem` into the cache (this is so there is
	/// never two items with the same ID stored in the cache at the same time).
	pub fn add(&self, cache_item: CacheItem) {
		let id = cache_item.object.clone();
		self.add_ref(id, Arc::new(Mutex::new(cache_item)));
	}

	fn add_ref(&self, id: CacheKey, cache_item: Arc<Mutex<CacheItem>>) {
		self.mark_in_use();
		self.by_id.insert(id, Arc::clone(&cache_item));
		self.ref_task_sender
			.send(cache_item)
			.expect("cache ref populating task has failed!");
		self.mark_not_in_use();
	}

	fn mark_in_use(&self) {
		let (lock, _) = &*self.in_use;
		let mut in_use = lock.lock().unwrap();
		*in_use += 1;
	}

	fn mark_not_in_use(&self) {
		let (lock, cvar) = &*self.in_use;
		let mut in_use = lock.lock().unwrap();
		*in_use -= 1;
		cvar.notify_all();
	}

	/// Remove a single cache item
	pub fn remove(&self, id: &CacheKey) {
		self.by_id.remove(id);
	}
	/// Get a single item (mutable via lock) from the cache.
	// For now this is kept private, as it would be easy to naively deadlock if
	// someone does a chain of `get_ref`s on parent->child->parent, etc.
	#[allow(dead_code)]
	fn get_ref(&self, id: &CacheKey) -> Option<Arc<Mutex<CacheItem>>> {
		self.by_id.get(id).as_deref().cloned()
	}

	/// Get a single item from the cache.
	///
	/// This will allow you to get the item without holding any locks to it,
	/// at the cost of (1) a clone and (2) no guarantees that the data is kept up-to-date.
	#[allow(dead_code)]
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
	// TODO: is it better to mark in use for the entirety of a document load?
	pub fn add_all(&self, cache_items: Vec<CacheItem>) {
		cache_items.into_iter().for_each(|cache_item| {
			self.add(cache_item);
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
		self.mark_in_use();
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
		self.mark_not_in_use();
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
			.map(AccessiblePrimitive::try_from)
			.collect::<Result<Vec<AccessiblePrimitive>, _>>()?,
		cache,
	})
}

fn clone_arc_mutex<T: Clone>(arc: &Arc<Mutex<T>>) -> T {
	arc.lock().unwrap().clone()
}

/// Populate references forever.
///
/// Starts a loop that, whenever the cache is not in use, uses the time to
/// populate the weak references. The references to populate are informed by the
/// queue of incoming cache items.
//
// TODO maybe this should be immortal, panic-happy, and loud in logs.
fn populate_references(
	in_use_arc: Arc<(Mutex<usize>, Condvar)>,
	cache: ThreadSafeCache,
	tasks: mpsc::Receiver<Arc<Mutex<CacheItem>>>,
	shutdown_sender: Arc<Mutex<bool>>,
) {
	// Wait until no one is using the cache
	// But don't hold the lock! Let someone else start in between our tasks,
	// whenever they want. In other words, _this_ thread has low priority.
	enum Action {
		Continue,
		Break,
	}
	let wait_until_not_in_use = || {
		let (lock, cvar) = &*in_use_arc;
		let mut in_use = lock.lock().unwrap();
		while *in_use > 0 {
			let res = cvar.wait_timeout(in_use, Duration::from_millis(100)).unwrap();
			in_use = res.0;
			if res.1.timed_out() && *shutdown_sender.lock().unwrap() {
				return Action::Break;
			}
		}
		Action::Continue
	};
	// Keep trying to process incoming tasks until the channel hangs up (e.g.
	// when cache is dropped)
	'tasks: while let Ok(item_arc) = tasks.recv() {
		// First update this item's parent ref
		if let Action::Break = wait_until_not_in_use() {
			break 'tasks;
		}
		let mut item = item_arc.lock().unwrap();
		if let Some(parent_arc) = cache.get(&item.parent.key).as_deref() {
			item.parent.item = Arc::downgrade(parent_arc);
		}
		let children = item.children.clone();
		drop(item); // drop the lock

		// Next update the existing children to point to this item
		let item_ref = Arc::downgrade(&item_arc);
		for child_key in &children {
			if let Action::Break = wait_until_not_in_use() {
				break 'tasks;
			}
			if let Some(child) = cache.get(child_key).as_deref() {
				child.lock().unwrap().parent.item = Weak::clone(&item_ref);
			}
		}
	}
}

impl Drop for Cache {
	fn drop(&mut self) {
		let mut shutdown = self.ref_shutdown_sender.lock().unwrap();
		*shutdown = true;
	}
}
