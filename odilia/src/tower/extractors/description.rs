use crate::tower::{EventProp, GetProperty, PropertyType};
use crate::OdiliaError;
use odilia_cache::{Cache, CacheItem};

pub struct Description;

impl PropertyType for Description {
	type Type = Option<String>;
}

impl GetProperty<Description> for CacheItem {
	async fn get_property(
		&self,
		_cache: &Cache,
	) -> Result<EventProp<Description>, OdiliaError> {
		Ok(EventProp(self.description.clone()))
	}
}
