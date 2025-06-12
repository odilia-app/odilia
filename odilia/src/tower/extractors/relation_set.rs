use odilia_cache::{CacheActor, CacheItem, CacheRequest, CacheResponse, ConstRelationType};

use crate::{
	tower::{EventProp, GetProperty, PropertyType},
	OdiliaError,
};

pub struct RelationSet<T: ConstRelationType>(pub T::InnerStore);

impl<T: ConstRelationType> PropertyType for RelationSet<T> {
	type Type = T::InnerStore;
}

impl<T: ConstRelationType<InnerStore = Vec<CacheItem>>> GetProperty<RelationSet<T>> for CacheItem {
	async fn get_property(
		&self,
		cache: &CacheActor,
	) -> Result<EventProp<RelationSet<T>>, OdiliaError> {
		let resp = cache
			.request(CacheRequest::Relation(self.object.clone(), T::RELATION_TYPE))
			.await?;
		let rel = match resp {
			CacheResponse::Relations(rel) => rel,
			e => {
				tracing::error!("Inappropriate response from cache for `Relation` request: {e:?}");
				return Err(format!("Inappropriate response from cache for `Realtion` request: {e:?}").into());
			}
		};
		Ok(EventProp(rel.1))
	}
}
