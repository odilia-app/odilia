use odilia_cache::{CacheActor, CacheItem, CacheRequest, CacheResponse};

use crate::{
	tower::{EventProp, GetProperty, PropertyType},
	OdiliaError,
};

pub struct Children;

impl PropertyType for Children {
	type Type = Vec<CacheItem>;
}

impl GetProperty<Children> for CacheItem {
	async fn get_property(
		&self,
		cache: &CacheActor,
	) -> Result<EventProp<Children>, OdiliaError> {
		let resp = cache.request(CacheRequest::Children(self.object.clone())).await?;
		let chs = match resp {
			CacheResponse::Children(chs) => chs,
			e => {
				tracing::error!("Inappropriate response from cache for `Relation` request: {e:?}");
				return Err(format!("Inappropriate response from cache for `Realtion` request: {e:?}").into());
			}
		};
		Ok(EventProp(chs.0))
	}
}
