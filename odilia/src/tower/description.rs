use crate::{tower::EventProp, tower::EventProperty, OdiliaError, ScreenReaderState};
use atspi::EventProperties;
use std::sync::Arc;

pub struct Description;

impl EventProperty for Description {
	type Output = Option<String>;
	async fn from_state<E>(
		state: Arc<ScreenReaderState>,
		event: E,
	) -> Result<EventProp<Self>, OdiliaError>
	where
		E: EventProperties,
	{
		state.get_or_create_event_object_to_cache::<E>(&event)
			.await?
			.description()
			.await
			.map(|s| if s.is_empty() { None } else { Some(s) })
			.map(EventProp)
	}
}
