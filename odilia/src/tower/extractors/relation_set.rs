use crate::tower::{EventProp, GetProperty, PropertyType};
use crate::OdiliaError;
use atspi::RelationType;
use odilia_cache::{Cache, CacheItem};

pub struct RelationSet;

impl PropertyType for RelationSet {
	type Type = Vec<(RelationType, Vec<CacheItem>)>;
}

impl GetProperty<RelationSet> for CacheItem {
	async fn get_property(&self, cache: &Cache) -> Result<EventProp<RelationSet>, OdiliaError> {
		Ok(EventProp(self.relation_set.unchecked_into_cache_itmes(cache)))
	}
}
