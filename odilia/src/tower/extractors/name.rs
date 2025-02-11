use crate::tower::{EventProp, GetProperty, PropertyType};
use crate::OdiliaError;
use odilia_cache::CacheItem;

pub struct Name;

impl PropertyType for Name {
	type Type = Option<String>;
}

impl GetProperty<Name> for CacheItem {
	async fn get_property(&self) -> Result<EventProp<Name>, OdiliaError> {
		self.description()
			.await
			.map(|s| if s.is_empty() { None } else { Some(s) })
			.map(EventProp)
	}
}
