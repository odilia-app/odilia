use crate::{Cache, AccessiblePrimitiveHostExt};
use async_trait::async_trait;
use std::sync::{Weak, Arc};
use parking_lot::RwLock;
use std::collections::HashMap;
use zbus::Connection;
use atspi_proxies::{text::TextProxy, accessible::AccessibleProxy};
use atspi_client::{convertable::Convertable, text_ext::TextExt};
use atspi_common::{GenericEvent, Granularity, CoordType, ClipType, RelationType};
use odilia_common::{OdiliaResult, cache::{CacheItem, CacheRef, AccessiblePrimitive}, errors::{OdiliaError, CacheError}};

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
