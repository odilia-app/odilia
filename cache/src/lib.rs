#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	unsafe_code
)]
#![allow(clippy::multiple_crate_versions)]

mod convertable;
pub use convertable::Convertable;
mod accessible_ext;
use std::{
	collections::{HashMap, VecDeque},
	fmt::Debug,
	future::Future,
	sync::Arc,
};

pub use accessible_ext::AccessibleExt;
use atspi_common::{EventProperties, InterfaceSet, ObjectRef, RelationType, Role, StateSet};
use atspi_proxies::{accessible::AccessibleProxy, text::TextProxy};
use dashmap::DashMap;
use futures_concurrency::future::TryJoin;
use fxhash::FxBuildHasher;
use indextree::{Arena, NodeId};
use itertools::Itertools;
use odilia_common::{
	cache::AccessiblePrimitive,
	errors::{CacheError, OdiliaError},
	result::OdiliaResult,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use zbus::proxy::CacheProperties;

trait AllText {
	async fn get_all_text(&self) -> Result<String, OdiliaError>;
}
impl AllText for TextProxy<'_> {
	async fn get_all_text(&self) -> Result<String, OdiliaError> {
		let length_of_string = self.character_count().await?;
		Ok(self.get_text(0, length_of_string).await?)
	}
}

pub type CacheKey = AccessiblePrimitive;
type ThreadSafeCache = Arc<RwLock<Arena<CacheItem>>>;
type IdLookupTable = Arc<DashMap<CacheKey, NodeId, FxBuildHasher>>;

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Link {
	Linked(NodeId),
	Unlinked(CacheKey),
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct RelationSet(Vec<(RelationType, Vec<Link>)>);

impl RelationSet {
	/// Turn the `Link` items into `CacheItem`s.
	///
	/// This function ignores [`Link::Unlinked`] variants.
	#[must_use]
	pub fn unchecked_into_cache_items<D: CacheDriver>(
		&self,
		c: &Cache<D>,
	) -> Vec<(RelationType, Vec<CacheItem>)> {
		self.0.iter()
			.map(|(rt, links)| {
				(
					*rt,
					links.iter()
						.filter_map(|link| match link {
							Link::Unlinked(_) => None,
							Link::Linked(nid) => c.get_id(*nid),
						})
						.collect::<Vec<CacheItem>>(),
				)
			})
			.collect()
	}
	fn try_link_values<D: CacheDriver>(
		&mut self,
		cache: &Cache<D>,
	) -> Result<(), Vec<AccessiblePrimitive>> {
		let relations_map = self.0.iter_mut();
		let link_map = relations_map.flat_map(|(_rt, links)| links);
		let mut unlinked = Vec::new();
		for link in link_map {
			if let Link::Unlinked(ap) = link {
				if let Some(nid) = cache.id_lookup.get(ap) {
					*link = Link::Linked(*nid);
				} else {
					unlinked.push(ap.clone());
				}
			}
		}

		if unlinked.is_empty() {
			return Ok(());
		}
		Err(unlinked)
	}
}

impl From<Vec<(RelationType, Vec<ObjectRef>)>> for RelationSet {
	fn from(vec: Vec<(RelationType, Vec<ObjectRef>)>) -> Self {
		vec.into_iter()
			.map(|(rt, vor)| {
				(rt, vor.into_iter().map(|a| Link::Unlinked(a.into())).collect())
			})
			.collect::<Vec<(RelationType, Vec<Link>)>>()
			.into()
	}
}

impl From<Vec<(RelationType, Vec<Link>)>> for RelationSet {
	fn from(vec: Vec<(RelationType, Vec<Link>)>) -> RelationSet {
		RelationSet(vec)
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A struct representing an accessible. To get any information from the cache other than the stored information like role, interfaces, and states, you will need to instantiate an [`atspi_proxies::accessible::AccessibleProxy`] or other `*Proxy` type from atspi to query further info.
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
	/// The text of the accessible
	pub text: String,
	/// Description of the item
	pub description: Option<String>,
	/// Name of the item
	pub name: Option<String>,
	/// The children (ids) of the accessible
	pub children: Vec<AccessiblePrimitive>,
	/// The set of relations between this and other nodes in the graph.
	pub relation_set: RelationSet,
}

impl CacheItem {
	/// Creates a `CacheItem` from an [`atspi::Event`] type.
	/// # Errors
	/// This can fail under three possible conditions:
	///
	/// 1. We are unable to convert information from the event into an [`AccessiblePrimitive`] hashmap key. This should never happen.
	/// 2. We are unable to convert the [`AccessiblePrimitive`] to an [`atspi_proxies::accessible::AccessibleProxy`].
	/// 3. The `accessible_to_cache_item` function fails for any reason. This also shouldn't happen.
	#[tracing::instrument(level = "trace", skip_all, ret, err)]
	pub async fn from_atspi_event<T: EventProperties, E: CacheDriver>(
		event: &T,
		external: &E,
	) -> OdiliaResult<Self> {
		let a11y_prim = AccessiblePrimitive::from_event(event);
		external.lookup_external(&a11y_prim).await
	}
}

/// An internal cache used within Odilia.
///
/// This contains (mostly) all accessibles in the entire accessibility tree, and
/// they are referenced by their IDs. If you are having issues with incorrect or
/// invalid accessibles trying to be accessed, this is code is probably the issue.
#[derive(Clone)]
pub struct Cache<D: CacheDriver> {
	pub tree: ThreadSafeCache,
	pub id_lookup: IdLookupTable,
	pub driver: D,
}

impl<D: CacheDriver> std::fmt::Debug for Cache<D> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// NOTE: This prints the number of items in the cache, INCLUDING "removed" items, which are not
		// actually removed, just "marked for removal".
		let cache = self.tree.read();
		f.write_str(&format!("Cache {{ tree: ...{} nodes..., .. }}", cache.count()))
	}
}

pub trait CacheExt {
	/// Get a single item from the cache. This will also get the information from `DBus` if it does not
	/// exist in the cache.
	fn get_ipc(
		&self,
		id: &CacheKey,
	) -> impl Future<Output = Result<CacheItem, OdiliaError>> + Send;
	fn item_from_event<T: EventProperties + Sync>(
		&self,
		ev: &T,
	) -> impl Future<Output = OdiliaResult<CacheItem>> + Send;
}

/// A method of performing I/O side-effects outside the cache itself.
/// This is made an explicit trait such that we can either:
///
/// 1. Call out to `DBus` (production), or
/// 2. Use a fixed set of items (testing).
/// 3. Panic when called.
/// etc.
///
/// Feel free to implement your own at your convenience.
pub trait CacheDriver {
	/// Lookup a given [`CacheKey`] that was not found in the cache..
	fn lookup_external(
		&self,
		key: &CacheKey,
	) -> impl Future<Output = OdiliaResult<CacheItem>> + Send;
}

impl CacheDriver for zbus::Connection {
	#[tracing::instrument(level = "trace", ret, skip(self))]
	async fn lookup_external(&self, key: &CacheKey) -> OdiliaResult<CacheItem> {
		let accessible = AccessibleProxy::builder(&self)
			.destination(key.sender.clone())?
			.cache_properties(CacheProperties::No)
			.path(key.id.clone())?
			.build()
			.await?;
		accessible_to_cache_item(&accessible).await
	}
}

impl CacheDriver for HashMap<CacheKey, CacheItem> {
	#[tracing::instrument(level = "trace", ret, skip(self))]
	async fn lookup_external(&self, key: &CacheKey) -> OdiliaResult<CacheItem> {
		Ok(self.get(key).ok_or::<OdiliaError>(CacheError::NoItem.into())?.clone())
	}
}

// N.B.: we are using std RwLockes internally here, within the cache hashmap
// entries. When adding async methods, take care not to hold these mutexes
// across .await points.
impl<D: CacheDriver> Cache<D> {
	/// create a new, fresh cache
	#[must_use]
	#[tracing::instrument(level = "debug", ret, skip_all)]
	pub fn new(driver: D) -> Self {
		Self {
			tree: Arc::new(RwLock::new(Arena::with_capacity(10_000))),
			id_lookup: Arc::new(DashMap::with_capacity_and_hasher(
				10_000,
				FxBuildHasher::default(),
			)),
			driver,
		}
	}
	/// Add an item via a reference instead of creating the reference.
	#[tracing::instrument(level = "trace", ret, err)]
	pub fn add(&self, mut cache_item: CacheItem) -> OdiliaResult<()> {
		// Do not create new items when not necessary.
		if let Some(self_id) = self.id_lookup.get(&cache_item.object) {
			return Err(CacheError::DuplicateItem(*self_id).into());
		}
		let maybe_index = cache_item.index;
		let key = cache_item.object.clone();
		let parent_key = cache_item.parent.clone();
		let mut cache = self.tree.write();
		let unlinked_related_items = cache_item.relation_set.try_link_values(self);
		let children = cache_item.children.clone();
		let id = cache.new_node(cache_item);
		self.id_lookup.insert(key, id);
		// no need to connect to the rest of the graph, because it's the first item.
		if self.id_lookup.len() == 1 {
			return Ok(());
		}
		let Some(parent_id) = self.id_lookup.get(&parent_key) else {
			return Err(CacheError::MoreData(Vec::from([parent_key])).into());
		};
		if let Err(unlinked_related_items) = unlinked_related_items {
			return Err(CacheError::MoreData(unlinked_related_items).into());
		}
		let Some(sibling_index) = maybe_index else {
			return Ok(());
		};
		if sibling_index == 0 {
			parent_id.checked_prepend(id, &mut cache)?;
		} else if sibling_index == parent_id.children(&cache).count() {
			parent_id.checked_append(id, &mut cache)?;
		} else {
			match parent_id.children(&cache).nth(sibling_index) {
				Some(left_sibling) => {
					left_sibling.checked_insert_after(id, &mut cache)?;
				}
				// TODO: specific child?
				None => return Err(CacheError::MoreData(children).into()),
			}
		}
		Ok(())
	}

	/// Remove a single cache item. This function can not fail.
	#[tracing::instrument(level = "trace", ret)]
	pub fn remove(&self, id: &CacheKey) {
		// if the item is not found in the lookup table, just return.
		let Some((_key, node_id)) = self.id_lookup.remove(id) else {
			tracing::warn!("Attempted to remove an item that doesn't exist: {id:?}");
			return;
		};
		let mut cache = self.tree.write();
		// Remove the item from the tree structure.
		node_id.remove(&mut cache);
	}

	/// Get a single item from the cache by ID.
	///
	/// This will allow you to get the item without holding any locks to it,
	/// at the cost of (1) a clone and (2) no guarantees that the data is kept up-to-date.
	#[must_use]
	#[tracing::instrument(level = "trace", ret)]
	pub fn get_id(&self, id: NodeId) -> Option<CacheItem> {
		let cache = self.tree.read();
		let ref_item = cache.get(id)?;
		// clone the reference into an owned value
		Some(ref_item.get().to_owned())
	}

	/// Get a single item from the cache.
	///
	/// This will allow you to get the item without holding any locks to it,
	/// at the cost of (1) a clone and (2) no guarantees that the data is kept up-to-date.
	#[must_use]
	#[tracing::instrument(level = "trace", ret)]
	pub fn get(&self, id: &CacheKey) -> Option<CacheItem> {
		let cache = self.tree.read();
		let node_id = self.id_lookup.get(id)?;
		let ref_item = cache.get(*node_id)?;
		// clone the reference into an owned value
		Some(ref_item.get().to_owned())
	}

	/// get a many items from the cache; this only creates one read handle (note that this will copy all data you would like to access)
	#[must_use]
	#[tracing::instrument(level = "trace", ret)]
	pub fn get_all(&self, ids: &[CacheKey]) -> Vec<Option<CacheItem>> {
		ids.iter().map(|id| self.get(id)).collect()
	}

	/// Bulk add many items to the cache; only one accessible should ever be
	/// associated with an id.
	/// # Errors
	/// An `Err(_)` variant may be returned if the [`Cache::populate_references`] function fails.
	// TODO: add ", err" back to instrumentqation
	#[tracing::instrument(level = "trace", ret)]
	pub fn add_all(&self, cache_items: Vec<CacheItem>) -> OdiliaResult<()> {
		for cache_item in cache_items {
			let _ = self.add(cache_item);
		}
		Ok(())
	}
	/// Bulk remove all ids in the cache; this only refreshes the cache after removing all items.
	#[tracing::instrument(level = "trace", ret)]
	pub fn remove_all(&mut self, ids: &[CacheKey]) {
		for id in ids {
			self.remove(id);
		}
	}

	/// Edit a mutable `CacheItem`. Returns true if the update was successful.
	///
	/// Note: an exclusive lock for the given cache item will be placed for the
	/// entire length of the passed function, so try to avoid any compute in it.
	///
	/// # Errors
	///
	/// An [`odilia_common::errors::OdiliaError::PoisoningError`] may be returned if a write lock can not be acquired on the `CacheItem` being modified.
	#[tracing::instrument(level = "trace", skip(modify), ret, err)]
	pub fn modify_item<F>(&self, id: &CacheKey, modify: F) -> OdiliaResult<bool>
	where
		F: FnOnce(&mut CacheItem),
	{
		let Some(node_id) = self.id_lookup.get(id) else {
			tracing::warn!("The lookup table does not contain this item: {:?}", id);
			return Ok(false);
		};
		let mut cache = self.tree.write();
		let Some(node) = cache.get_mut(*node_id) else {
			tracing::warn!("The tree cache does not contain this item: {:?}", node_id);
			return Ok(false);
		};
		let cache_item = node.get_mut();
		modify(cache_item);
		Ok(true)
	}

	/// Get a single item from the cache (note that this copies some integers to a new struct).
	/// If the `CacheItem` is not found, create one, add it to the cache, and return it.
	/// # Errors
	/// The function will return an error if:
	/// 1. The `accessible` can not be turned into an `AccessiblePrimitive`. This should never happen, but is technically possible.
	/// 2. The [`Self::add`] function fails.
	/// 3. The [`accessible_to_cache_item`] function fails.
	#[tracing::instrument(level = "debug", ret, err, skip(self))]
	pub async fn get_or_create(&self, key: &AccessiblePrimitive) -> OdiliaResult<CacheItem> {
		// if the item already exists in the cache, return it
		if let Some(cache_item) = self.get(key) {
			return Ok(cache_item);
		}
		// otherwise, build a cache item
		// NOTE: recursion CAN NOT be used here due to the size of some a11y trees and the size of
		// async stack frames.
		let mut stack: VecDeque<AccessiblePrimitive> = vec![key.to_owned()].into();
		let mut first_ci = None;
		while let Some(item) = stack.pop_front() {
			let cache_item = self.driver.lookup_external(&item).await?;
			if first_ci.is_none() {
				first_ci = Some(cache_item.clone());
			}
			if let Err(OdiliaError::Cache(CacheError::MoreData(items))) =
				self.add(cache_item)
			{
				stack.extend(items);
			}
		}
		// return that same cache item
		// SAFETY: this is okay because we always have one item in the stack, and guarantee that any
		// errors along the way to setting the value cause an early return.
		Ok(first_ci.expect("Able to extract first CacheItem!"))
	}

	/// Clears the cache completely.
	pub fn clear(&self) {
		self.tree.write().clear();
		self.id_lookup.clear();
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
#[tracing::instrument(level = "trace", ret, err)]
pub async fn accessible_to_cache_item(accessible: &AccessibleProxy<'_>) -> OdiliaResult<CacheItem> {
	let (app, parent, index, children_num, interfaces, role, states, children) = (
		accessible.get_application(),
		accessible.parent(),
		accessible.get_index_in_parent(),
		accessible.child_count(),
		accessible.get_interfaces(),
		accessible.get_role(),
		accessible.get_state(),
		accessible.get_children(),
	)
		.try_join()
		.await?;
	let rs = accessible.get_relation_set().await?.into();
	let name = accessible
		.name()
		.await
		.map(|s| if s.is_empty() { None } else { Some(s) })?;
	let desc = accessible
		.description()
		.await
		.map(|s| if s.is_empty() { None } else { Some(s) })?;
	// if it implements the Text interface
	let text = match accessible.to_text().await {
		// get *all* the text
		Ok(text_iface) => text_iface.get_all_text().await,
		// otherwise, use the name instaed
		Err(_) => Ok(accessible.name().await?),
	}?;
	Ok(CacheItem {
		object: accessible.try_into()?,
		app: app.into(),
		parent: parent.into(),
		index: index.try_into().ok(),
		children_num: children_num.try_into().ok(),
		interfaces,
		role,
		states,
		text,
		children: children.into_iter().map_into().collect(),
		relation_set: rs,
		name,
		description: desc,
	})
}
