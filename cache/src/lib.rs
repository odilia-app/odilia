#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	unsafe_code
)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_arguments)]

mod convertable;
pub use convertable::Convertable;
mod accessible_ext;
pub use accessible_ext::AccessibleExt;

use std::{
	collections::HashMap,
	fmt::Debug,
	future::Future,
	sync::{Arc, RwLock, Weak},
};

use atspi_common::{
	ClipType, CoordType, EventProperties, Granularity, InterfaceSet, RelationType, Role,
	StateSet,
};
use atspi_proxies::{accessible::AccessibleProxy, text::TextProxy};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
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
type InnerCache = DashMap<CacheKey, Arc<RwLock<CacheItem>>, FxBuildHasher>;
type ThreadSafeCache = Arc<InnerCache>;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A struct representing an accessible. To get any information from the cache other than the stored information like role, interfaces, and states, you will need to instantiate an [`atspi_proxies::accessible::AccessibleProxy`] or other `*Proxy` type from atspi to query further info.
pub struct CacheItem {
	/// The accessible object (within the application)    (so)
	pub object: AccessiblePrimitive,
	/// The application (root object(?)      (so)
	pub app: AccessiblePrimitive,
	/// The parent object.  (so)
	pub parent: CacheRef,
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
	/// The children (ids) of the accessible
	pub children: Vec<CacheRef>,

	#[serde(skip)]
	pub cache: Weak<Cache>,
}
impl CacheItem {
	/// Return a *reference* to a parent. This is *much* cheaper than getting the parent element outright via [`Self::parent`].
	/// # Errors
	/// This method will return a [`CacheError::NoItem`] if no item is found within the cache.
	#[tracing::instrument(level = "trace", ret, err)]
	pub fn parent_ref(&mut self) -> OdiliaResult<Arc<std::sync::RwLock<CacheItem>>> {
		let parent_ref = Weak::upgrade(&self.parent.item);
		if let Some(p) = parent_ref {
			Ok(p)
		} else {
			let cache = strong_cache(&self.cache)?;
			let arc_mut_parent = cache
				.get_ref(&self.parent.key.clone())
				.ok_or(CacheError::NoItem)?;
			self.parent.item = Arc::downgrade(&arc_mut_parent);
			Ok(arc_mut_parent)
		}
	}
	/// Creates a `CacheItem` from an [`atspi::Event`] type.
	/// # Errors
	/// This can fail under three possible conditions:
	///
	/// 1. We are unable to convert information from the event into an [`AccessiblePrimitive`] hashmap key. This should never happen.
	/// 2. We are unable to convert the [`AccessiblePrimitive`] to an [`atspi_proxies::accessible::AccessibleProxy`].
	/// 3. The `accessible_to_cache_item` function fails for any reason. This also shouldn't happen.
	#[tracing::instrument(level = "trace", skip_all, ret, err)]
	pub async fn from_atspi_event<T: EventProperties>(
		event: &T,
		cache: Arc<Cache>,
		connection: &zbus::Connection,
	) -> OdiliaResult<Self> {
		let a11y_prim = AccessiblePrimitive::from_event(event);
		accessible_to_cache_item(
			&a11y_prim.into_accessible(connection).await?,
			Arc::downgrade(&cache),
		)
		.await
	}
	/// Convert an [`atspi::CacheItem`] into a [`crate::CacheItem`].
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
	pub async fn from_atspi_cache_item(
		atspi_cache_item: atspi_common::CacheItem,
		cache: Weak<Cache>,
		connection: &zbus::Connection,
	) -> OdiliaResult<Self> {
		let children: Vec<CacheRef> =
			AccessiblePrimitive::from(atspi_cache_item.object.clone())
				.into_accessible(connection)
				.await?
				.get_children()
				.await?
				.into_iter()
				.map(|child_object_pair| CacheRef::new(child_object_pair.into()))
				.collect();
		Ok(Self {
			object: atspi_cache_item.object.into(),
			app: atspi_cache_item.app.into(),
			parent: CacheRef::new(atspi_cache_item.parent.into()),
			index: atspi_cache_item.index.try_into().ok(),
			children_num: atspi_cache_item.children.try_into().ok(),
			interfaces: atspi_cache_item.ifaces,
			role: atspi_cache_item.role,
			states: atspi_cache_item.states,
			text: atspi_cache_item.name,
			cache,
			children,
		})
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
		cache: Weak<Cache>,
		connection: &zbus::Connection,
	) -> OdiliaResult<Self> {
		let index: i32 = AccessiblePrimitive::from(atspi_cache_item.object.clone())
			.into_accessible(connection)
			.await?
			.get_index_in_parent()
			.await?;
		Ok(Self {
			object: atspi_cache_item.object.into(),
			app: atspi_cache_item.app.into(),
			parent: CacheRef::new(atspi_cache_item.parent.into()),
			index: index.try_into().ok(),
			children_num: Some(atspi_cache_item.children.len()),
			interfaces: atspi_cache_item.ifaces,
			role: atspi_cache_item.role,
			states: atspi_cache_item.states,
			text: atspi_cache_item.name,
			cache,
			children: atspi_cache_item
				.children
				.into_iter()
				.map(|or| CacheRef::new(or.into()))
				.collect(),
		})
	}
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
/// the parent be an Arc, or have the children be Arcs). Might even be possible to have both.
/// BUT - is it even desirable to keep an item pinned in an Arc from its
/// relatives after it has been removed from the cache?
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheRef {
	pub key: CacheKey,
	#[serde(skip)]
	item: Weak<RwLock<CacheItem>>,
}

impl CacheRef {
	#[must_use]
	#[tracing::instrument(level = "trace", skip_all, ret)]
	pub fn new(key: AccessiblePrimitive) -> Self {
		Self { key, item: Weak::new() }
	}

	#[must_use]
	pub fn clone_inner(&self) -> Option<CacheItem> {
		Some(self.item.upgrade().as_ref()?.read().ok()?.clone())
	}
}

impl From<AccessiblePrimitive> for CacheRef {
	#[tracing::instrument(level = "trace", ret)]
	fn from(value: AccessiblePrimitive) -> Self {
		Self::new(value)
	}
}

#[inline]
#[tracing::instrument(level = "trace", ret, err)]
async fn as_accessible(cache_item: &CacheItem) -> OdiliaResult<AccessibleProxy<'_>> {
	let cache = strong_cache(&cache_item.cache)?;
	Ok(cache_item.object.clone().into_accessible(&cache.connection).await?)
}
#[inline]
#[tracing::instrument(level = "trace", ret, err)]
async fn as_text(cache_item: &CacheItem) -> OdiliaResult<TextProxy<'_>> {
	let cache = strong_cache(&cache_item.cache)?;
	Ok(cache_item.object.clone().into_text(&cache.connection).await?)
}

#[inline]
#[tracing::instrument(level = "trace", ret, err)]
fn strong_cache(weak_cache: &Weak<Cache>) -> OdiliaResult<Arc<Cache>> {
	Weak::upgrade(weak_cache).ok_or(OdiliaError::Cache(CacheError::NotAvailable))
}

impl CacheItem {
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
		let parent_item = self
			.parent
			.clone_inner()
			.or_else(|| self.cache.upgrade()?.get(&self.parent.key));
		parent_item.ok_or(CacheError::NoItem.into())
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
}

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

/// An internal cache used within Odilia.
///
/// This contains (mostly) all accessibles in the entire accessibility tree, and
/// they are referenced by their IDs. If you are having issues with incorrect or
/// invalid accessibles trying to be accessed, this is code is probably the issue.
#[derive(Clone)]
pub struct Cache {
	pub by_id: ThreadSafeCache,
	pub connection: zbus::Connection,
}

impl std::fmt::Debug for Cache {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&format!("Cache {{ by_id: ...{} items..., .. }}", self.by_id.len()))
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
		accessible_to_cache_item(&acc, Arc::downgrade(self)).await
	}
	async fn item_from_event<T: EventProperties + Sync>(
		&self,
		ev: &T,
	) -> OdiliaResult<CacheItem> {
		let a11y_prim = AccessiblePrimitive::from_event(ev);
		accessible_to_cache_item(
			&a11y_prim.into_accessible(&self.connection).await?,
			Arc::downgrade(self),
		)
		.await
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
	#[tracing::instrument(level = "trace", ret, err)]
	pub fn add(&self, cache_item: CacheItem) -> OdiliaResult<()> {
		let id = cache_item.object.clone();
		self.add_ref(id, &Arc::new(RwLock::new(cache_item)))
	}

	/// Add an item via a reference instead of creating the reference.
	/// # Errors
	/// Can error if [`Cache::populate_references`] errors. The insertion is guarenteed to succeed.
	#[tracing::instrument(level = "trace", ret, err)]
	pub fn add_ref(
		&self,
		id: CacheKey,
		cache_item: &Arc<RwLock<CacheItem>>,
	) -> OdiliaResult<()> {
		self.by_id.insert(id, Arc::clone(cache_item));
		Self::populate_references(&self.by_id, cache_item)
	}

	/// Remove a single cache item. This function can not fail.
	#[tracing::instrument(level = "trace", ret)]
	pub fn remove(&self, id: &CacheKey) {
		self.by_id.remove(id);
	}

	/// Get a single item from the cache, this only gets a reference to an item, not the item itself.
	/// You will need to either get a read or a write lock on any item returned from this function.
	/// It also may return `None` if a value is not matched to the key.
	#[must_use]
	#[tracing::instrument(level = "trace", ret)]
	pub fn get_ref(&self, id: &CacheKey) -> Option<Arc<RwLock<CacheItem>>> {
		self.by_id.get(id).as_deref().cloned()
	}

	/// Get a single item from the cache.
	///
	/// This will allow you to get the item without holding any locks to it,
	/// at the cost of (1) a clone and (2) no guarantees that the data is kept up-to-date.
	#[must_use]
	#[tracing::instrument(level = "trace", ret)]
	pub fn get(&self, id: &CacheKey) -> Option<CacheItem> {
		Some(self.by_id.get(id).as_deref()?.read().ok()?.clone())
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
	#[tracing::instrument(level = "trace", ret, err)]
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
	#[tracing::instrument(level = "trace", ret)]
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
	/// An [`odilia_common::errors::OdiliaError::PoisoningError`] may be returned if a write lock can not be acquired on the `CacheItem` being modified.
	#[tracing::instrument(level = "trace", skip(modify), ret, err)]
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
		let mut cache_item = entry.write()?;
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
	#[tracing::instrument(level = "debug", ret, err)]
	pub async fn get_or_create(
		&self,
		accessible: &AccessibleProxy<'_>,
		cache: Arc<Cache>,
	) -> OdiliaResult<CacheItem> {
		// if the item already exists in the cache, return it
		let primitive = accessible.try_into()?;
		if let Some(cache_item) = self.get(&primitive) {
			return Ok(cache_item);
		}
		// otherwise, build a cache item
		let start = std::time::Instant::now();
		let cache_item =
			accessible_to_cache_item(accessible, Arc::downgrade(&cache)).await?;
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
	#[tracing::instrument(level = "trace", ret, err)]
	pub fn populate_references(
		cache: &ThreadSafeCache,
		item_ref: &Arc<RwLock<CacheItem>>,
	) -> Result<(), OdiliaError> {
		let item_wk_ref = Arc::downgrade(item_ref);
		let mut item = item_ref.write()?;
		let item_key = item.object.clone();

		let parent_key = item.parent.key.clone();
		let parent_ref_opt = cache.get(&parent_key);

		// Update this item's children references
		for child_ref in &mut item.children {
			if let Some(child_arc) = cache.get(&child_ref.key).as_ref() {
				child_ref.item = Arc::downgrade(child_arc);
				child_arc.write()?.parent.item = Weak::clone(&item_wk_ref);
			}
		}

		// TODO: Should there be errors for the non let bindings?
		if let Some(parent_ref) = parent_ref_opt {
			item.parent.item = Arc::downgrade(&parent_ref);
			if let Some(ix) = item.index {
				if let Some(cache_ref) = parent_ref
					.write()?
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
#[tracing::instrument(level = "trace", ret, err)]
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
		app: app.into(),
		parent: CacheRef::new(parent.into()),
		index: index.try_into().ok(),
		children_num: children_num.try_into().ok(),
		interfaces,
		role,
		states,
		text,
		children: children.into_iter().map(|k| CacheRef::new(k.into())).collect(),
		cache,
	})
}
