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
pub use accessible_ext::AccessibleExt;

use itertools::Itertools;
use parking_lot::RwLock;
use std::{
	collections::{HashMap, VecDeque},
	fmt::Debug,
	future::Future,
	sync::Arc,
};

use atspi_common::{EventProperties, InterfaceSet, ObjectRef, RelationType, Role, StateSet};
use atspi_proxies::{accessible::AccessibleProxy, text::TextProxy};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use indextree::{Arena, NodeId};
use odilia_common::{
	cache::AccessiblePrimitive,
	errors::{CacheError, OdiliaError},
	result::OdiliaResult,
};
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

type CacheKey = AccessiblePrimitive;
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
	pub fn unchecked_into_cache_itmes(&self, c: &Cache) -> Vec<(RelationType, Vec<CacheItem>)> {
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
	fn try_link_values(&mut self, cache: &Cache) -> Result<(), Vec<AccessiblePrimitive>> {
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
	pub async fn from_atspi_event<T: EventProperties, E: CacheSideEffect>(
		event: &T,
		external: &E,
	) -> OdiliaResult<Self> {
		let a11y_prim = AccessiblePrimitive::from_event(event);
		external.lookup_external(&a11y_prim).await
	}
	/// Convert an [`atspi::LegacyCacheItem`] into a [`crate::CacheItem`].
	/// This requires calls to `DBus`, which is quite expensive. Beware calling this too often.
	/// # Errors
	/// This function can fail under the following conditions:
	///
	/// 1. The [`atspi::CacheItem`] can not be turned into a [`crate::AccessiblePrimitive`]. This should never happen.
	/// 2. The [`crate::AccessiblePrimitive`] can not be turned into a [`atspi_proxies::accessible::AccessibleProxy`]. This should never happen.
	/// 3. Getting children from the `AccessibleProxy` fails. This should never happen.
	///
	/// The only time these can fail is if the item is removed on the application side before the conversion to `AccessibleProxy`.
	#[tracing::instrument(level = "trace", skip_all, ret, err)]
	pub async fn from_atspi_legacy_cache_item(
		atspi_cache_item: atspi_common::LegacyCacheItem,
		connection: &zbus::Connection,
	) -> OdiliaResult<Self> {
		let acc = AccessiblePrimitive::from(atspi_cache_item.object.clone())
			.into_accessible(connection)
			.await?;
		let index: i32 = acc.get_index_in_parent().await?;
		let rs = acc.get_relation_set().await?.into();
		let name = acc.name().await.map(|s| if s.is_empty() { None } else { Some(s) })?;
		let desc =
			acc.description()
				.await
				.map(|s| if s.is_empty() { None } else { Some(s) })?;
		Ok(Self {
			object: atspi_cache_item.object.into(),
			app: atspi_cache_item.app.into(),
			parent: atspi_cache_item.parent.into(),
			index: index.try_into().ok(),
			children_num: Some(atspi_cache_item.children.len()),
			interfaces: atspi_cache_item.ifaces,
			role: atspi_cache_item.role,
			states: atspi_cache_item.states,
			text: atspi_cache_item.name,
			children: atspi_cache_item.children.into_iter().map_into::<_>().collect(),
			relation_set: rs,
			name,
			description: desc,
		})
	}
	/*
	      // Same as [`AccessibleProxy::get_children`], just offered as a non-async version.
	      /// Get a `Vec` of children with the same type as `Self`.
	      /// # Errors
	      /// 1. Will return an `Err` variant if `self.cache` does not reference an active cache. This should never happen, but it is technically possible.
	      /// 2. Any children keys' values are not found in the cache itself.
	      #[tracing::instrument(level = "trace", skip_all, ret, err)]
	      pub fn get_children(&self) -> OdiliaResult<Vec<Self>> {
		      let derefed_cache: Arc<Cache> = strong_cache(&self.cache)?;
		      let children = self
			      .children
			      .iter()
			      .map(|child_ref| derefed_cache.get(child_ref).ok_or(CacheError::NoItem))
			      .collect::<Result<Vec<_>, _>>()?;
		      Ok(children)
	      }
	*/
}

impl CacheItem {
	/*
	      /// See [`atspi_proxies::accessible::AccessibleProxy::get_application`]
	      /// # Errors
	      /// - [`CacheError::NoItem`] if application is not in cache
	      pub fn get_application(&self) -> Result<Self, OdiliaError> {
		      let derefed_cache: Arc<Cache> = strong_cache(&self.cache)?;
		      derefed_cache.get(&self.app).ok_or(CacheError::NoItem.into())
	      }
	      /// See [`atspi_proxies::accessible::AccessibleProxy::parent`]
	      /// # Errors
	      /// - [`CacheError::NoItem`] if application is not in cache
	      pub fn parent(&self) -> Result<Self, OdiliaError> {
		      self.cache
			      .upgrade()
			      .ok_or::<OdiliaError>(CacheError::NotAvailable.into())?
			      .get(&self.parent)
			      .ok_or(CacheError::NoItem.into())
	      }
	      /// See [`atspi_proxies::accessible::AccessibleProxy::get_attributes`]
	      /// # Errors
	      /// - If the item is no longer available over the AT-SPI connection.
	      pub async fn get_attributes(&self) -> Result<HashMap<String, String>, OdiliaError> {
		      Ok(as_accessible(self).await?.get_attributes().await?)
	      }
	      /// See [`atspi_proxies::accessible::AccessibleProxy::name`]
	      /// # Errors
	      /// - If the item is no longer available over the AT-SPI connection.
	      pub async fn name(&self) -> Result<String, OdiliaError> {
		      Ok(as_accessible(self).await?.name().await?)
	      }
	      /// See [`atspi_proxies::accessible::AccessibleProxy::locale`]
	      /// # Errors
	      /// - If the item is no longer available over the AT-SPI connection.
	      pub async fn locale(&self) -> Result<String, OdiliaError> {
		      Ok(as_accessible(self).await?.locale().await?)
	      }
	      /// See [`atspi_proxies::accessible::AccessibleProxy::description`]
	      /// # Errors
	      /// - If the item is no longer available over the AT-SPI connection.
	      pub async fn description(&self) -> Result<String, OdiliaError> {
		      Ok(as_accessible(self).await?.description().await?)
	      }
	      /// See [`atspi_proxies::accessible::AccessibleProxy::get_relation_set`]
	      /// # Errors
	      /// - If the item is no longer available over the AT-SPI connection.
	      /// - The items mentioned are not in the cache.
	      pub async fn get_relation_set(
		      &self,
	      ) -> Result<Vec<(RelationType, Vec<Self>)>, OdiliaError> {
		      let cache = strong_cache(&self.cache)?;
		      let ipc_rs = as_accessible(self).await?.get_relation_set().await?;
		      let mut relations = Vec::new();
		      for (relation, object_pairs) in ipc_rs {
			      let mut cache_keys = Vec::new();
			      for object_pair in object_pairs {
				      let cached = cache.get_ipc(&object_pair.into()).await?;
				      cache_keys.push(cached);
			      }
			      relations.push((relation, cache_keys));
		      }
		      Ok(relations)
	      }
	      /// See [`atspi_proxies::accessible::AccessibleProxy::get_child_at_index`]
	      /// # Errors
	      /// - The items mentioned are not in the cache.
	      pub fn get_child_at_index(&self, idx: i32) -> Result<Self, OdiliaError> {
		      self.get_children()?
			      .get(usize::try_from(idx)?)
			      .ok_or(CacheError::NoItem.into())
			      .cloned()
	      }
	*/
}

/*
impl CacheItem {
	pub async fn add_selection(
		&self,
		start_offset: i32,
		end_offset: i32,
	) -> Result<bool, OdiliaError> {
		Ok(as_text(self).await?.add_selection(start_offset, end_offset).await?)
	}
	pub async fn get_attribute_run(
		&self,
		offset: i32,
		include_defaults: bool,
	) -> Result<(std::collections::HashMap<String, String>, i32, i32), OdiliaError> {
		Ok(as_text(self)
			.await?
			.get_attribute_run(offset, include_defaults)
			.await?)
	}
	pub async fn get_attribute_value(
		&self,
		offset: i32,
		attribute_name: &str,
	) -> Result<String, OdiliaError> {
		Ok(as_text(self)
			.await?
			.get_attribute_value(offset, attribute_name)
			.await?)
	}
	pub async fn get_text_attributes(
		&self,
		offset: i32,
	) -> Result<(std::collections::HashMap<String, String>, i32, i32), OdiliaError> {
		Ok(as_text(self).await?.get_attributes(offset).await?)
	}
	pub async fn get_bounded_ranges(
		&self,
		x: i32,
		y: i32,
		width: i32,
		height: i32,
		coord_type: CoordType,
		x_clip_type: ClipType,
		y_clip_type: ClipType,
	) -> Result<Vec<(i32, i32, String, zbus::zvariant::OwnedValue)>, OdiliaError> {
		Ok(as_text(self)
			.await?
			.get_bounded_ranges(
				x,
				y,
				width,
				height,
				coord_type,
				x_clip_type,
				y_clip_type,
			)
			.await?)
	}
	pub async fn get_character_at_offset(&self, offset: i32) -> Result<i32, OdiliaError> {
		Ok(as_text(self).await?.get_character_at_offset(offset).await?)
	}
	pub async fn get_character_extents(
		&self,
		offset: i32,
		coord_type: CoordType,
	) -> Result<(i32, i32, i32, i32), OdiliaError> {
		Ok(as_text(self).await?.get_character_extents(offset, coord_type).await?)
	}
	pub async fn get_default_attribute_set(
		&self,
	) -> Result<std::collections::HashMap<String, String>, OdiliaError> {
		Ok(as_text(self).await?.get_default_attribute_set().await?)
	}
	pub async fn get_default_attributes(
		&self,
	) -> Result<std::collections::HashMap<String, String>, OdiliaError> {
		Ok(as_text(self).await?.get_default_attributes().await?)
	}
	pub async fn get_nselections(&self) -> Result<i32, OdiliaError> {
		Ok(as_text(self).await?.get_nselections().await?)
	}
	pub async fn get_offset_at_point(
		&self,
		x: i32,
		y: i32,
		coord_type: CoordType,
	) -> Result<i32, OdiliaError> {
		Ok(as_text(self).await?.get_offset_at_point(x, y, coord_type).await?)
	}
	pub async fn get_range_extents(
		&self,
		start_offset: i32,
		end_offset: i32,
		coord_type: CoordType,
	) -> Result<(i32, i32, i32, i32), OdiliaError> {
		Ok(as_text(self)
			.await?
			.get_range_extents(start_offset, end_offset, coord_type)
			.await?)
	}
	pub async fn get_selection(&self, selection_num: i32) -> Result<(i32, i32), OdiliaError> {
		Ok(as_text(self).await?.get_selection(selection_num).await?)
	}
	pub async fn get_string_at_offset(
		&self,
		offset: usize,
		granularity: Granularity,
	) -> Result<(String, usize, usize), OdiliaError> {
		// optimisations that don't call out to DBus.
		if granularity == Granularity::Paragraph {
			return Ok((self.text.clone(), 0, self.text.len()));
		} else if granularity == Granularity::Char {
			let range = offset..=offset;
			return Ok((
				self.text
					.get(range)
					.ok_or(CacheError::TextBoundsError)?
					.to_string(),
				offset,
				offset + 1,
			));
		} else if granularity == Granularity::Word {
			return Ok(self
				.text
				// [char]
				.split_whitespace()
				// [(word, start, end)]
				.filter_map(|word| {
					let start = self
						.text
						// [(idx, char)]
						.char_indices()
						// [(idx, char)]: uses pointer arithmatic to find start index
						.find(|&(idx, _)| {
							idx == word.as_ptr() as usize
								- self.text.as_ptr() as usize
						})
						// [idx]
						.map(|(idx, _)| idx)?;
					// calculate based on start
					let end = start + word.len();
					// if the offset if within bounds
					if offset >= start && offset <= end {
						Some((word.to_string(), start, end))
					} else {
						None
					}
				})
				// get "all" words that match; there should be only one result
				.collect::<Vec<_>>()
				.first()
				// if there's no matching word (out of bounds)
				.ok_or_else(|| OdiliaError::Generic("Out of bounds".to_string()))?
				// clone the reference into a value
				.clone());
		}
		// any other variations, in particular, Granularity::Line, will need to call out to DBus. It's just too complex to calculate, get updates for bounding boxes, etc.
		// this variation does NOT get a semantic line. It gets a visual line.
		let dbus_version = as_text(self)
			.await?
			.get_string_at_offset(offset.try_into()?, granularity)
			.await?;
		Ok((dbus_version.0, dbus_version.1.try_into()?, dbus_version.2.try_into()?))
	}
	pub fn get_text(
		&self,
		start_offset: usize,
		end_offset: usize,
	) -> Result<String, OdiliaError> {
		self.text
			.get(start_offset..end_offset)
			.map(std::borrow::ToOwned::to_owned)
			.ok_or(OdiliaError::Generic("Type is None, not Some".to_string()))
	}
	pub fn get_all_text(&self) -> Result<String, OdiliaError> {
		let length_of_string = self.character_count();
		self.get_text(0, length_of_string)
	}
	pub async fn get_text_after_offset(
		&self,
		offset: i32,
		type_: u32,
	) -> Result<(String, i32, i32), OdiliaError> {
		Ok(as_text(self).await?.get_text_after_offset(offset, type_).await?)
	}
	pub async fn get_text_at_offset(
		&self,
		offset: i32,
		type_: u32,
	) -> Result<(String, i32, i32), OdiliaError> {
		Ok(as_text(self).await?.get_text_at_offset(offset, type_).await?)
	}
	pub async fn get_text_before_offset(
		&self,
		offset: i32,
		type_: u32,
	) -> Result<(String, i32, i32), OdiliaError> {
		Ok(as_text(self).await?.get_text_before_offset(offset, type_).await?)
	}
	pub async fn remove_selection(&self, selection_num: i32) -> Result<bool, OdiliaError> {
		Ok(as_text(self).await?.remove_selection(selection_num).await?)
	}
	pub async fn scroll_substring_to(
		&self,
		start_offset: i32,
		end_offset: i32,
		type_: u32,
	) -> Result<bool, OdiliaError> {
		Ok(as_text(self)
			.await?
			.scroll_substring_to(start_offset, end_offset, type_)
			.await?)
	}
	pub async fn scroll_substring_to_point(
		&self,
		start_offset: i32,
		end_offset: i32,
		type_: u32,
		x: i32,
		y: i32,
	) -> Result<bool, OdiliaError> {
		Ok(as_text(self)
			.await?
			.scroll_substring_to_point(start_offset, end_offset, type_, x, y)
			.await?)
	}
	pub async fn set_caret_offset(&self, offset: i32) -> Result<bool, OdiliaError> {
		Ok(as_text(self).await?.set_caret_offset(offset).await?)
	}
	pub async fn set_selection(
		&self,
		selection_num: i32,
		start_offset: i32,
		end_offset: i32,
	) -> Result<bool, OdiliaError> {
		Ok(as_text(self)
			.await?
			.set_selection(selection_num, start_offset, end_offset)
			.await?)
	}
	/// Get the live caret offset from the system
	/// # Errors
	/// - Fails of the [`self.object_ref`] referes to an invalid item on the bus
	/// - An IPC error from `zbus` it detected.
	pub async fn caret_offset(&self) -> Result<i32, OdiliaError> {
		Ok(as_text(self).await?.caret_offset().await?)
	}
	#[must_use]
	pub fn character_count(&self) -> usize {
		self.text.len()
	}
}
*/

/// An internal cache used within Odilia.
///
/// This contains (mostly) all accessibles in the entire accessibility tree, and
/// they are referenced by their IDs. If you are having issues with incorrect or
/// invalid accessibles trying to be accessed, this is code is probably the issue.
#[derive(Clone)]
pub struct Cache {
	pub tree: ThreadSafeCache,
	pub id_lookup: IdLookupTable,
	pub connection: zbus::Connection,
}

impl std::fmt::Debug for Cache {
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

impl CacheExt for Arc<Cache> {
	#[tracing::instrument(level = "trace", ret)]
	async fn get_ipc(&self, id: &CacheKey) -> Result<CacheItem, OdiliaError> {
		if let Some(ci) = self.get(id) {
			return Ok(ci);
		}
		let acc = id.clone().into_accessible(&self.connection).await?;
		accessible_to_cache_item(&acc).await
	}
	async fn item_from_event<T: EventProperties + Sync>(
		&self,
		ev: &T,
	) -> OdiliaResult<CacheItem> {
		let a11y_prim = AccessiblePrimitive::from_event(ev);
		accessible_to_cache_item(&a11y_prim.into_accessible(&self.connection).await?).await
	}
}

/// A method of performing I/O side-effects outside the cache itself.
/// This is made an explicit trait such that we can either:
///
/// 1. Call out to `DBus` (production), or
/// 2. Use a fixed set of items (testing).
pub trait CacheSideEffect {
	/// Lookup a given [`CacheKey`] that was not found in the cache..
	fn lookup_external(
		&self,
		key: &CacheKey,
	) -> impl Future<Output = OdiliaResult<CacheItem>> + Send;
}

pub struct CacheEffectDBus {
	connection: zbus::Connection,
}
impl CacheSideEffect for CacheEffectDBus {
	async fn lookup_external(&self, key: &CacheKey) -> OdiliaResult<CacheItem> {
		let accessible = AccessibleProxy::builder(&self.connection)
			.destination(key.sender.clone())?
			.cache_properties(CacheProperties::No)
			.path(key.id.clone())?
			.build()
			.await?;
		accessible_to_cache_item(&accessible).await
	}
}

pub struct CacheEffectFixedList {
	items: HashMap<CacheKey, CacheItem>,
}
impl CacheSideEffect for CacheEffectFixedList {
	async fn lookup_external(&self, key: &CacheKey) -> OdiliaResult<CacheItem> {
		Ok(self.items
			.get(key)
			.ok_or::<OdiliaError>(CacheError::NoItem.into())?
			.clone())
	}
}

// N.B.: we are using std RwLockes internally here, within the cache hashmap
// entries. When adding async methods, take care not to hold these mutexes
// across .await points.
impl Cache {
	/// create a new, fresh cache
	#[must_use]
	#[tracing::instrument(level = "debug", ret, skip_all)]
	pub fn new(conn: zbus::Connection) -> Self {
		Self {
			tree: Arc::new(RwLock::new(Arena::with_capacity(10_000))),
			id_lookup: Arc::new(DashMap::with_capacity_and_hasher(
				10_000,
				FxBuildHasher::default(),
			)),
			connection: conn,
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
			return;
		};
		let mut cache = self.tree.write();
		// Remove the item from the tree structure.
		node_id.remove(&mut cache);
	}

	/*
		/// Get a single item from the cache, this only gets a reference to an item, not the item itself.
		/// You will need to either get a read or a write lock on any item returned from this function.
		/// It also may return `None` if a value is not matched to the key.
		#[must_use]
		#[tracing::instrument(level = "trace", ret)]
		pub fn get_ref(&self, id: &CacheKey) -> Option<&Node<CacheItem>> {
			let node_id = self.id_lookup.get(id)?;
	    let cache = self.tree.read()
		.expect("Unable to lock RwLock!");
	    cache.get(*node_id)
		}
	*/
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
			tracing::trace!("The lookup table does not contian this item: {:?}", id);
			return Ok(false);
		};
		let mut cache = self.tree.write();
		let Some(node) = cache.get_mut(*node_id) else {
			tracing::trace!("The tree cache does not contain this item: {:?}", node_id);
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
	#[tracing::instrument(level = "debug", ret, err)]
	pub async fn get_or_create(
		&self,
		key: &AccessiblePrimitive,
		connection: &zbus::Connection,
		cache: Arc<Cache>,
	) -> OdiliaResult<CacheItem> {
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
			let accessible = AccessibleProxy::builder(connection)
				.destination(item.sender)?
				.cache_properties(CacheProperties::No)
				.path(item.id)?
				.build()
				.await?;
			let start = std::time::Instant::now();
			let cache_item = accessible_to_cache_item(&accessible).await?;
			if first_ci.is_none() {
				first_ci = Some(cache_item.clone());
			}
			let end = std::time::Instant::now();
			let diff = end - start;
			tracing::debug!("Time to create cache item: {:?}", diff);
			if let Err(OdiliaError::Cache(CacheError::MoreData(items))) =
				self.add(cache_item)
			{
				stack.extend(items);
			}
		}
		// return that same cache item
		// SAFETY: this is okay because we always have one item in the stack, and guarantee that any
		// errors along the way to setting the value cause an early return.
		#[allow(clippy::unwrap_used)]
		Ok(first_ci.unwrap())
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
