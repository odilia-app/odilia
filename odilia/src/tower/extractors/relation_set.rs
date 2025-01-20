use crate::tower::{EventProp, GetProperty, PropertyType};
use crate::OdiliaError;
use atspi::RelationType;
use odilia_cache::CacheItem;

pub struct RelationSet;

impl PropertyType for RelationSet {
	type Type = Vec<(RelationType, Vec<CacheItem>)>;
}

impl GetProperty<RelationSet> for CacheItem {
	async fn get_property(&self) -> Result<EventProp<RelationSet>, OdiliaError> {
		self.get_relation_set().await.map(EventProp)
	}
}
