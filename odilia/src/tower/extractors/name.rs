use crate::tower::{EventProp, GetProperty, PropertyType};
use crate::OdiliaError;
use odilia_cache::{Cache, CacheItem};

pub struct Name;

impl PropertyType for Name {
	type Type = Option<String>;
}

impl GetProperty<Name> for CacheItem {
	async fn get_property(&self, _cache: &Cache) -> Result<EventProp<Name>, OdiliaError> {
		Ok(EventProp(self.name.clone()))
	}
}
