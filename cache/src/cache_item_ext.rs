use crate::{Cache, AccessiblePrimitiveHostExt};
use async_trait::async_trait;
use std::sync::{Weak, Arc};
use std::collections::HashMap;
use zbus::Connection;
use atspi_proxies::{text::TextProxy, accessible::AccessibleProxy};
use atspi_client::{convertable::Convertable, text_ext::TextExt};
use atspi_common::{GenericEvent, Granularity, CoordType, ClipType, RelationType};
use odilia_common::{result::OdiliaResult, cache::{CacheItem, CacheRef, AccessiblePrimitive}, errors::{OdiliaError, CacheError}};

#[inline]
async fn as_accessible<'a>(cache_item: &CacheItem, conn: &Connection) -> OdiliaResult<AccessibleProxy<'a>> {
	Ok(cache_item.object.clone().into_accessible(&conn).await?)
}
#[inline]
async fn as_text<'a>(cache_item: &CacheItem, conn: &Connection) -> OdiliaResult<TextProxy<'a>> {
	Ok(cache_item.object.clone().into_text(&conn).await?)
}

#[async_trait]
pub trait CacheItemHostExt {
	fn parent_ref(
		&mut self,
		conn: &Cache,
	) -> OdiliaResult<Arc<std::sync::RwLock<CacheItem>>>;
	async fn from_atspi_event<'a, T: GenericEvent<'a> + Sync>(
		event: &T,
		conn: &Connection,
	) -> OdiliaResult<Self> where Self: Sized;
	async fn from_atspi_cache_item(
		atspi_cache_item: atspi_common::CacheItem,
		conn: &Connection,
	) -> OdiliaResult<Self> where Self: Sized;
}

#[async_trait]
impl CacheItemHostExt for CacheItem {
	/// Return a *reference* to a parent. This is *much* cheaper than getting the parent element outright via [`Self::parent`].
	/// # Errors
	/// This method will return a [`CacheError::NoItem`] if no item is found within the cache.
	fn parent_ref(&mut self, cache: &Cache) -> OdiliaResult<Arc<std::sync::RwLock<CacheItem>>> {
		let parent_ref = Weak::upgrade(&self.parent.item);
		if let Some(p) = parent_ref {
			Ok(p)
		} else {
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
	async fn from_atspi_event<'a, T: GenericEvent<'a> + Sync>(
		event: &T,
		conn: &Connection,
	) -> OdiliaResult<Self> where Self: Sized {
		let a11y_prim = AccessiblePrimitive::from_event(event)?;
		accessible_to_cache_item(&a11y_prim.into_accessible(conn).await?).await
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
	async fn from_atspi_cache_item(
		atspi_cache_item: atspi_common::CacheItem,
		conn: &Connection,
	) -> OdiliaResult<Self> where Self: Sized {
		let children: Vec<CacheRef> =
			AccessiblePrimitive::try_from(atspi_cache_item.object.clone())?
				.into_accessible(conn)
				.await?
				.get_children()
				.await?
				.into_iter()
				.map(|child_object_pair| CacheRef::new(child_object_pair.into()))
				.collect();
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
			children,
		})
	}
}

#[async_trait]
pub trait AccessibleHostExt {
	type Error: std::error::Error;

	fn text(&self) -> String;
	fn get_application(&self, cache: &Cache) -> Result<Self, Self::Error> where Self: Sized;
	fn parent(&self, cache: &Cache) -> Result<Self, Self::Error> where Self: Sized;
	fn get_children(&self, cache: &Cache) -> Result<Vec<Self>, Self::Error> where Self: Sized;
	async fn get_attributes(&self, conn: &Connection) -> Result<HashMap<String, String>, Self::Error>;
	async fn name(&self, conn: &Connection) -> Result<String, Self::Error>;
	async fn locale(&self, conn: &Connection) -> Result<String, Self::Error>;
	async fn description(&self, conn: &Connection) -> Result<String, Self::Error>;
	async fn get_relation_set(&self, conn: &Connection, cache: &Cache) -> Result<Vec<(RelationType, Vec<Self>)>, Self::Error> where Self: Sized;
	async fn get_role_name(&self, conn: &Connection) -> Result<String, Self::Error>;
	fn get_child_at_index(&self, idx: i32, cache: &Cache) -> Result<Self, Self::Error> where Self: Sized;
	async fn get_localized_role_name(&self, conn: &Connection) -> Result<String, Self::Error>;
}

#[async_trait]
impl AccessibleHostExt for CacheItem {
	type Error = OdiliaError;

	fn text(&self) -> String {
		self.text.clone()
	}
	fn get_application(&self, cache: &Cache) -> Result<Self, Self::Error> {
		cache.get(&self.app).ok_or(CacheError::NoItem.into())
	}
	fn parent(&self, cache: &Cache) -> Result<Self, Self::Error> {
		let parent_item = self
			.parent
			.clone_inner()
			.or_else(|| cache.get(&self.parent.key));
		parent_item.ok_or(CacheError::NoItem.into())
	}
	/// Get a `Vec` of children with the same type as `Self`.
	/// # Errors
	/// 1. Will return an `Err` variant if `self.cache` does not reference an active cache. This should never happen, but it is technically possible.
	/// 2. Any children keys' values are not found in the cache itself.
	fn get_children(&self, cache: &Cache) -> OdiliaResult<Vec<Self>> {
		let children = self
			.children
			.iter()
			.map(|child_ref| {
				child_ref
					.clone_inner()
					.or_else(|| cache.get(&child_ref.key))
					.ok_or(CacheError::NoItem)
			})
			.collect::<Result<Vec<_>, _>>()?;
		Ok(children)
	}
	async fn get_attributes(&self, conn: &Connection) -> Result<HashMap<String, String>, Self::Error> {
		Ok(as_accessible(self, conn).await?.get_attributes().await?)
	}
	async fn name(&self, conn: &Connection) -> Result<String, Self::Error> {
		Ok(as_accessible(self, conn).await?.name().await?)
	}
	async fn locale(&self, conn: &Connection) -> Result<String, Self::Error> {
		Ok(as_accessible(self, conn).await?.locale().await?)
	}
	async fn description(&self, conn: &Connection) -> Result<String, Self::Error> {
		Ok(as_accessible(self, conn).await?.description().await?)
	}
	async fn get_relation_set(&self, conn: &Connection, cache: &Cache) -> Result<Vec<(RelationType, Vec<Self>)>, Self::Error> {
		as_accessible(self, conn)
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
	async fn get_role_name(&self, conn: &Connection) -> Result<String, Self::Error> {
		Ok(as_accessible(self, conn).await?.get_role_name().await?)
	}
	fn get_child_at_index(&self, idx: i32, cache: &Cache) -> Result<Self, Self::Error> {
		<Self as AccessibleHostExt>::get_children(self, cache)?
			.get(usize::try_from(idx)?)
			.ok_or(CacheError::NoItem.into())
			.cloned()
	}
	async fn get_localized_role_name(&self, conn: &Connection) -> Result<String, Self::Error> {
		Ok(as_accessible(self, conn).await?.get_localized_role_name().await?)
	}
}

#[async_trait]
pub trait TextHostExt {
	type Error: std::error::Error;

	async fn add_selection(
		&self,
		start_offset: i32,
		end_offset: i32,
		conn: &Connection,
	) -> Result<bool, Self::Error>;
	async fn get_attribute_run(
		&self,
		offset: i32,
		include_defaults: bool,
		conn: &Connection,
	) -> Result<(std::collections::HashMap<String, String>, i32, i32), Self::Error>;
	async fn get_attribute_value(
		&self,
		offset: i32,
		attribute_name: &str,
		conn: &Connection,
	) -> Result<String, Self::Error>;
	async fn get_attributes(
		&self,
		offset: i32,
		conn: &Connection,
	) -> Result<(std::collections::HashMap<String, String>, i32, i32), Self::Error>;
	async fn get_bounded_ranges(
		&self,
		x: i32,
		y: i32,
		width: i32,
		height: i32,
		coord_type: CoordType,
		x_clip_type: ClipType,
		y_clip_type: ClipType,
		conn: &Connection,
	) -> Result<Vec<(i32, i32, String, zbus::zvariant::OwnedValue)>, Self::Error>;
	async fn get_character_extents(
		&self,
		offset: i32,
		coord_type: CoordType,
		conn: &Connection,
	) -> Result<(i32, i32, i32, i32), Self::Error>;
	async fn get_default_attribute_set(
		&self,
		conn: &Connection,
	) -> Result<std::collections::HashMap<String, String>, Self::Error>;
	async fn get_default_attributes(
		&self,
		conn: &Connection,
	) -> Result<std::collections::HashMap<String, String>, Self::Error>;
	async fn get_nselections(&self, conn: &Connection) -> Result<i32, Self::Error>;
	async fn get_offset_at_point(
		&self,
		x: i32,
		y: i32,
		coord_type: CoordType,
		conn: &Connection,
	) -> Result<i32, Self::Error>;
	async fn get_range_extents(
		&self,
		start_offset: i32,
		end_offset: i32,
		coord_type: CoordType,
		conn: &Connection,
	) -> Result<(i32, i32, i32, i32), Self::Error>;
	async fn get_selection(&self, selection_num: i32, conn: &Connection) -> Result<(i32, i32), Self::Error>;
	async fn get_string_at_offset(
		&self,
		offset: i32,
		granularity: Granularity,
		conn: &Connection,
	) -> Result<(String, i32, i32), Self::Error>;
	fn get_text(
		&self,
		start_offset: i32,
		end_offset: i32,
	) -> Result<String, Self::Error>;
	async fn get_text_after_offset(
		&self,
		offset: i32,
		type_: u32,
		conn: &Connection,
	) -> Result<(String, i32, i32), Self::Error>;
	async fn get_text_at_offset(
		&self,
		offset: i32,
		type_: u32,
		conn: &Connection,
	) -> Result<(String, i32, i32), Self::Error>;
	async fn get_text_before_offset(
		&self,
		offset: i32,
		type_: u32,
		conn: &Connection,
	) -> Result<(String, i32, i32), Self::Error>;
	async fn remove_selection(&self, selection_num: i32, conn: &Connection) -> Result<bool, Self::Error>;
	async fn scroll_substring_to(
		&self,
		start_offset: i32,
		end_offset: i32,
		type_: u32,
		conn: &Connection,
	) -> Result<bool, Self::Error>;
	async fn scroll_substring_to_point(
		&self,
		start_offset: i32,
		end_offset: i32,
		type_: u32,
		x: i32,
		y: i32,
		conn: &Connection,
	) -> Result<bool, Self::Error>;
	async fn set_caret_offset(&self, offset: i32, conn: &Connection) -> Result<bool, Self::Error>;
	async fn set_selection(
		&self,
		selection_num: i32,
		start_offset: i32,
		end_offset: i32,
		conn: &Connection,
	) -> Result<bool, Self::Error>;
	async fn caret_offset(&self, conn: &Connection) -> Result<i32, Self::Error>;
	fn character_count(&self) -> Result<i32, Self::Error>;
}

#[async_trait]
impl TextHostExt for CacheItem {
	type Error = OdiliaError;

	async fn add_selection(
		&self,
		start_offset: i32,
		end_offset: i32,
	  conn: &Connection) -> Result<bool, Self::Error> {
		Ok(as_text(self, conn).await?.add_selection(start_offset, end_offset).await?)
	}
	async fn get_attribute_run(
		&self,
		offset: i32,
		include_defaults: bool,
	  conn: &Connection) -> Result<(std::collections::HashMap<String, String>, i32, i32), Self::Error> {
		Ok(as_text(self, conn)
			.await?
			.get_attribute_run(offset, include_defaults)
			.await?)
	}
	async fn get_attribute_value(
		&self,
		offset: i32,
		attribute_name: &str,
	  conn: &Connection) -> Result<String, Self::Error> {
		Ok(as_text(self, conn)
			.await?
			.get_attribute_value(offset, attribute_name)
			.await?)
	}
	async fn get_attributes(
		&self,
		offset: i32,
	  conn: &Connection) -> Result<(std::collections::HashMap<String, String>, i32, i32), Self::Error> {
		Ok(as_text(self, conn).await?.get_attributes(offset).await?)
	}
	async fn get_bounded_ranges(
		&self,
		x: i32,
		y: i32,
		width: i32,
		height: i32,
		coord_type: CoordType,
		x_clip_type: ClipType,
		y_clip_type: ClipType,
	  conn: &Connection) -> Result<Vec<(i32, i32, String, zbus::zvariant::OwnedValue)>, Self::Error> {
		Ok(as_text(self, conn)
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
	async fn get_character_extents(
		&self,
		offset: i32,
		coord_type: CoordType,
	  conn: &Connection) -> Result<(i32, i32, i32, i32), Self::Error> {
		Ok(as_text(self, conn).await?.get_character_extents(offset, coord_type).await?)
	}
	async fn get_default_attribute_set(
		&self,
	  conn: &Connection) -> Result<std::collections::HashMap<String, String>, Self::Error> {
		Ok(as_text(self, conn).await?.get_default_attribute_set().await?)
	}
	async fn get_default_attributes(
		&self,
	  conn: &Connection) -> Result<std::collections::HashMap<String, String>, Self::Error> {
		Ok(as_text(self, conn).await?.get_default_attributes().await?)
	}
	async fn get_nselections(&self, conn: &Connection) -> Result<i32, Self::Error> {
		Ok(as_text(self, conn).await?.get_nselections().await?)
	}
	async fn get_offset_at_point(
		&self,
		x: i32,
		y: i32,
		coord_type: CoordType,
	  conn: &Connection) -> Result<i32, Self::Error> {
		Ok(as_text(self, conn).await?.get_offset_at_point(x, y, coord_type).await?)
	}
	async fn get_range_extents(
		&self,
		start_offset: i32,
		end_offset: i32,
		coord_type: CoordType,
	  conn: &Connection) -> Result<(i32, i32, i32, i32), Self::Error> {
		Ok(as_text(self, conn)
			.await?
			.get_range_extents(start_offset, end_offset, coord_type)
			.await?)
	}
	async fn get_selection(&self, selection_num: i32, conn: &Connection) -> Result<(i32, i32), Self::Error> {
		Ok(as_text(self, conn).await?.get_selection(selection_num).await?)
	}
	async fn get_string_at_offset(
		&self,
		offset: i32,
		granularity: Granularity,
	  conn: &Connection) -> Result<(String, i32, i32), Self::Error> {
		let uoffset = usize::try_from(offset)?;
		// optimisations that don't call out to DBus.
		if granularity == Granularity::Paragraph {
			return Ok((self.text.clone(), 0, self.text.len().try_into()?));
		} else if granularity == Granularity::Char {
			let range = uoffset..=uoffset;
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
				// [(idx, char)]
				.enumerate()
				// [(word, start, end)]
				.filter_map(|(_, word)| {
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
					let i_start = i32::try_from(start).ok()?;
					let i_end = i32::try_from(end).ok()?;
					// if the offset if within bounds
					if uoffset >= start && uoffset <= end {
						Some((word.to_string(), i_start, i_end))
					} else {
						None
					}
				})
				// get "all" words that match; there should be only one result
				.collect::<Vec<_>>()
				// get the first result
				.get(0)
				// if there's no matching word (out of bounds)
				.ok_or_else(|| OdiliaError::Generic("Out of bounds".to_string()))?
				// clone the reference into a value
				.clone());
		}
		// any other variations, in particular, Granularity::Line, will need to call out to DBus. It's just too complex to calculate, get updates for bounding boxes, etc.
		// this variation does NOT get a semantic line. It gets a visual line.
		Ok(as_text(self, conn).await?.get_string_at_offset(offset, granularity).await?)
	}
	fn get_text(
		&self,
		start_offset: i32,
		end_offset: i32,
	) -> Result<String, Self::Error> {
		self.text
			.get(usize::try_from(start_offset)?..usize::try_from(end_offset)?)
			.map(std::borrow::ToOwned::to_owned)
			.ok_or(OdiliaError::Generic("Type is None, not Some".to_string()))
	}
	async fn get_text_after_offset(
		&self,
		offset: i32,
		type_: u32,
	  conn: &Connection) -> Result<(String, i32, i32), Self::Error> {
		Ok(as_text(self, conn).await?.get_text_after_offset(offset, type_).await?)
	}
	async fn get_text_at_offset(
		&self,
		offset: i32,
		type_: u32,
	  conn: &Connection) -> Result<(String, i32, i32), Self::Error> {
		Ok(as_text(self, conn).await?.get_text_at_offset(offset, type_).await?)
	}
	async fn get_text_before_offset(
		&self,
		offset: i32,
		type_: u32,
	  conn: &Connection) -> Result<(String, i32, i32), Self::Error> {
		Ok(as_text(self, conn).await?.get_text_before_offset(offset, type_).await?)
	}
	async fn remove_selection(&self, selection_num: i32, conn: &Connection) -> Result<bool, Self::Error> {
		Ok(as_text(self, conn).await?.remove_selection(selection_num).await?)
	}
	async fn scroll_substring_to(
		&self,
		start_offset: i32,
		end_offset: i32,
		type_: u32,
	  conn: &Connection) -> Result<bool, Self::Error> {
		Ok(as_text(self, conn)
			.await?
			.scroll_substring_to(start_offset, end_offset, type_)
			.await?)
	}
	async fn scroll_substring_to_point(
		&self,
		start_offset: i32,
		end_offset: i32,
		type_: u32,
		x: i32,
		y: i32,
	  conn: &Connection) -> Result<bool, Self::Error> {
		Ok(as_text(self, conn)
			.await?
			.scroll_substring_to_point(start_offset, end_offset, type_, x, y)
			.await?)
	}
	async fn set_caret_offset(&self, offset: i32, conn: &Connection) -> Result<bool, Self::Error> {
		Ok(as_text(self, conn).await?.set_caret_offset(offset).await?)
	}
	async fn set_selection(
		&self,
		selection_num: i32,
		start_offset: i32,
		end_offset: i32,
	  conn: &Connection) -> Result<bool, Self::Error> {
		Ok(as_text(self, conn)
			.await?
			.set_selection(selection_num, start_offset, end_offset)
			.await?)
	}
	async fn caret_offset(&self, conn: &Connection) -> Result<i32, Self::Error> {
		Ok(as_text(self, conn).await?.caret_offset().await?)
	}
	fn character_count(&self) -> Result<i32, Self::Error> {
		Ok(i32::try_from(self.text.len())?)
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
		children: children.into_iter().map(|k| CacheRef::new(k.into())).collect(),
	})
}
