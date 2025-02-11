use crate::tower::{EventProp, GetProperty, PropertyType};
use crate::OdiliaError;
use odilia_cache::CacheItem;

pub struct Description;

impl PropertyType for Description {
	type Type = Option<String>;
}

impl GetProperty<Description> for CacheItem {
	async fn get_property(&self) -> Result<EventProp<Description>, OdiliaError> {
		self.description()
			.await
			.map(|s| if s.is_empty() { None } else { Some(s) })
			.map(EventProp)
	}
}
