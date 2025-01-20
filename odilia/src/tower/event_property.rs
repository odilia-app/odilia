use crate::{tower::from_state::TryFromState, OdiliaError, ScreenReaderState};
use atspi::EventProperties;
use core::future::Future;
use std::sync::Arc;

pub trait EventProperty: Sized {
	type Output;
	async fn from_state<E>(
		state: Arc<ScreenReaderState>,
		event: E,
	) -> Result<EventProp<Self>, OdiliaError>
	where
		E: EventProperties;
}

impl<E, T> TryFromState<Arc<ScreenReaderState>, E> for EventProp<T>
where
	T: EventProperty,
	E: EventProperties,
{
	type Error = OdiliaError;
	type Future = impl Future<Output = Result<EventProp<T>, Self::Error>>;
	fn try_from_state(s: Arc<ScreenReaderState>, e: E) -> Self::Future {
		T::from_state(s, e)
	}
}

#[repr(transparent)]
pub struct EventProp<EP: EventProperty>(pub EP::Output);
