use core::{future::Future, pin::Pin};
use std::sync::Arc;

use atspi::{Event, EventProperties};
use odilia_cache::{CacheActor, CacheItem};

use crate::{tower::from_state::TryFromState, OdiliaError, ScreenReaderState};

/// Define a representation for a property type.
/// Often, this differs from the internal representation.
///
/// For example: while descriptions and labels are stored in the cache as [`String`]s, there is a
/// semantic distinction in an empty string (""), therefore type [`PropertyType::Type`] parameter
/// would be `Option<String>` (or some equivelant type).
///
/// In general, we recommend using semantically useful types wherever possible.
pub trait PropertyType {
	/// The type the property is transformed into.
	type Type;
}

/// A helper trait which is (usually) implemented on [`CacheItem`], generically over the associated
/// type `P`.
/// `P` must implement [`PropertyType`], which defines the representation returned from
/// [`GetProperty::get_property`] in the `EventProp<P>` generic.
///
/// [`TryFromState`] is auto-implemented for any [`EventProp<P>`] and [`GetProperty<P>`] for
/// [`CacheItem`].
/// In practical terms, this means you can use it like an extractor:
///
/// ```
/// // in `main.rs`
/// use odilia::tower::{EventProp, Name};
/// async fn handle_event(
///     EventProp(name): EventProp<Name>,
/// ) {
///     todo!()
/// }
/// ```
pub trait GetProperty<P: PropertyType>: Sized {
	fn get_property(
		&self,
		cache: &CacheActor,
	) -> impl Future<Output = Result<EventProp<P>, OdiliaError>> + Send;
}

impl<E, T> TryFromState<Arc<ScreenReaderState>, E> for EventProp<T>
where
	CacheItem: GetProperty<T>,
	T: PropertyType,
	E: EventProperties + Into<Event> + Send + Sync + 'static,
	atspi::Event: From<E>,
{
	type Error = OdiliaError;
	type Future = Pin<
		Box<dyn Future<Output = Result<EventProp<T>, Self::Error>> + Send + 'static>,
	>;
	fn try_from_state(state: Arc<ScreenReaderState>, event: E) -> Self::Future {
		Box::pin(async move {
			let ci = state.cache_from_event(event.into()).await?;
			<CacheItem as GetProperty<T>>::get_property(&ci, &state.cache_actor).await
		})
	}
}

#[repr(transparent)]
pub struct EventProp<P: PropertyType>(pub P::Type);
