use crate::CacheItem;
use serde::{Serialize, Deserialize};
use odilia_common::cache::{CacheKey, AccessiblePrimitive};
use odilia_common::errors::{CacheError, OdiliaError};
use parking_lot::RwLock;
use std::{sync::{Arc, Weak}, hash::{Hasher, Hash}};

/// A composition of an accessible ID and (possibly) a reference
/// to its `CacheItem`, if the item has not been dropped from the cache yet.
/// TODO if desirable, we could make one direction strong references (e.g. have
/// the parent be an Arc, xor have the children be Arcs). Might even be possible to have both.
/// BUT - is it even desirable to keep an item pinned in an Arc from its
/// releatives after it has been removed from the cache?
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CacheRef {
	/// A key to find the cache item in the cache.
	pub key: CacheKey,
	/// An active reference to an item in cache.
	/// This will have to be de-referenced using `Weak::upgrade`.
	#[serde(skip)]
	pub item: Weak<RwLock<CacheItem>>,
}
impl TryFrom<CacheRef> for CacheItem {
	type Error = OdiliaError;

	fn try_from(cache_ref: CacheRef) -> Result<CacheItem, OdiliaError> {
		Ok(Weak::upgrade(&cache_ref.item)
			.ok_or(CacheError::NoItem)?
			.read()
			.clone())
	}
}
impl Hash for CacheRef {
	fn hash<H>(&self, hasher: &mut H) where H: Hasher {
		self.key.hash(hasher)
	}
}
impl PartialEq for CacheRef {
	fn eq(&self, other: &CacheRef) -> bool {
		self.key == other.key
	}
}
impl Eq for CacheRef {}

impl CacheRef {
	/// Create a new cache reference, which by itself will only populate the `item` field with an empty `Weak`.
	#[must_use]
	pub fn new(key: AccessiblePrimitive) -> Self {
		Self { key, item: Weak::new() }
	}

	/// Clone the underlying [`CacheItem`].
	#[must_use]
	pub fn clone_inner(&self) -> Option<CacheItem> {
		Some(self.item.upgrade().as_ref()?.read().clone())
	}
}

