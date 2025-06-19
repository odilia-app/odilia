use std::collections::VecDeque;

use odilia_cache::{CacheActor, CacheItem, CacheRequest, CacheResponse};

use crate::{
	tower::{EventProp, GetProperty, PropertyType},
	OdiliaError,
};

/// A property type that returns a list of elements which contians all elements of a subtree rooted
/// at the [`CacheItem`] passed in.
///
/// This collection contains the original `CacheItem`.
pub struct Subtree;

impl PropertyType for Subtree {
	type Type = Vec<CacheItem>;
}

impl GetProperty<Subtree> for CacheItem {
	async fn get_property(
		&self,
		cache: &CacheActor,
	) -> Result<EventProp<Subtree>, OdiliaError> {
		let mut subtree = vec![];
		let mut stack = VecDeque::new();
		stack.push_front(self.clone());
		while let Some(item) = stack.pop_front() {
			subtree.push(item.clone());
			let resp =
				cache.request(CacheRequest::Children(item.object.clone())).await?;
			let chs = match resp {
				CacheResponse::Children(chs) => chs,
				e => {
					tracing::error!("Inappropriate response from cache for `Children` request: {e:?}");
					return Err(format!("Inappropriate response from cache for `Realtion` request: {e:?}").into());
				}
			};
			for ch in chs.0 {
				// Only allow one copy of any circular reference.
				if subtree.iter().any(|i| ch.object == i.object) {
					continue;
				}
				stack.push_front(ch);
			}
		}
		Ok(EventProp(subtree))
	}
}
