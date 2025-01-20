use crate::{tower::from_state::TryFromState, OdiliaError, ScreenReaderState};
use atspi::EventProperties;
use core::{fmt::Debug, future::Future, ops::Deref};
use std::sync::Arc;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Name(Option<String>);

impl From<String> for Name {
	fn from(s: String) -> Name {
		if s.is_empty() {
			Name(None)
		} else {
			Name(Some(s))
		}
	}
}

impl Deref for Name {
	type Target = Option<String>;
	fn deref(&self) -> &Option<String> {
		&self.0
	}
}
async fn try_from_state<E>(state: Arc<ScreenReaderState>, event: E) -> Result<Name, OdiliaError>
where
	E: EventProperties,
{
	state.get_or_create_event_object_to_cache::<E>(&event)
		.await?
		.name()
		.await
		.map(Name::from)
}

try_from_state_event_fn!(try_from_state, Name);
