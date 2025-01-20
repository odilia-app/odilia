use crate::{tower::EventProp, tower::EventProperty, OdiliaError, ScreenReaderState};
use atspi::{EventProperties, RelationType};
use odilia_cache::CacheItem;
use std::sync::Arc;

pub struct RelationSet;

impl EventProperty for RelationSet {
	type Output = Vec<(RelationType, Vec<CacheItem>)>;
	async fn from_state<E>(
		state: Arc<ScreenReaderState>,
		event: E,
	) -> Result<EventProp<Self>, OdiliaError>
	where
		E: EventProperties,
	{
		state.get_or_create_event_object_to_cache::<E>(&event)
			.await?
			.get_relation_set()
			.await
			.map(EventProp)
	}
}
