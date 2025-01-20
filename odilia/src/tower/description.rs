use crate::{tower::from_state::TryFromState, OdiliaError, ScreenReaderState};
use atspi::EventProperties;
use core::{fmt::Debug, future::Future, ops::Deref};
use std::sync::Arc;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Description(Option<String>);

impl From<String> for Description {
	fn from(s: String) -> Description {
		if s.is_empty() {
			Description(None)
		} else {
			Description(Some(s))
		}
	}
}

impl Deref for Description {
	type Target = Option<String>;
	fn deref(&self) -> &Option<String> {
		&self.0
	}
}

async fn try_from_state<E>(
	state: Arc<ScreenReaderState>,
	event: E,
) -> Result<Description, OdiliaError>
where
	E: EventProperties + Debug,
{
	state.get_or_create_event_object_to_cache::<E>(&event)
		.await?
		.description()
		.await
		.map(Description::from)
}

try_from_state_event_fn!(try_from_state, Description, Debug);
