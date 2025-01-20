use crate::{tower::EventProp, tower::EventProperty, OdiliaError, ScreenReaderState};
use atspi::EventProperties;
use std::sync::Arc;

pub struct Name;

impl EventProperty for Name {
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
			.name()
			.await
			.map(|s| if s.is_empty() { None } else { Some(s) })
			.map(EventProp)
	}
}
