use crate::{tower::from_state::TryFromState, OdiliaError, ScreenReaderState};
use atspi_common::EventProperties;
use derived_deref::{Deref, DerefMut};
use odilia_cache::CacheItem;
use odilia_common::cache::AccessiblePrimitive;
use refinement::Predicate;
use std::fmt::Debug;
use std::{future::Future, marker::PhantomData, sync::Arc};
use zbus::{names::UniqueName, zvariant::ObjectPath};

pub type CacheEvent<E> = EventPredicate<E, Always>;
pub type ActiveAppEvent<E> = EventPredicate<E, ActiveApplication>;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct InnerEvent<E: EventProperties + Debug> {
	#[target]
	pub inner: E,
	pub item: CacheItem,
}
impl<E> InnerEvent<E>
where
	E: EventProperties + Debug,
{
	fn new(inner: E, item: CacheItem) -> Self {
		Self { inner, item }
	}
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct EventPredicate<E: EventProperties + Debug, P: Predicate<(E, Arc<ScreenReaderState>)>> {
	#[target]
	pub inner: E,
	pub item: CacheItem,
	_marker: PhantomData<P>,
}
impl<E, P> EventPredicate<E, P>
where
	E: EventProperties + Debug + Clone,
	P: Predicate<(E, Arc<ScreenReaderState>)>,
{
	pub fn from_cache_event(ce: InnerEvent<E>, state: Arc<ScreenReaderState>) -> Option<Self> {
		if P::test(&(ce.inner.clone(), state)) {
			return Some(Self { inner: ce.inner, item: ce.item, _marker: PhantomData });
		}
		None
	}
}

#[derive(Debug)]
pub struct Always;
impl<E> Predicate<(E, Arc<ScreenReaderState>)> for Always {
	fn test(_: &(E, Arc<ScreenReaderState>)) -> bool {
		true
	}
}

#[derive(Debug)]
pub struct ActiveApplication;
impl<E> Predicate<(E, Arc<ScreenReaderState>)> for ActiveApplication
where
	E: EventProperties,
{
	fn test((ev, state): &(E, Arc<ScreenReaderState>)) -> bool {
		let Some(last_focused) = state.history_item(0) else {
			return false;
		};
		last_focused == ev.object_ref().into()
	}
}

impl<E> TryFromState<Arc<ScreenReaderState>, E> for InnerEvent<E>
where
	E: EventProperties + Debug + Clone,
{
	type Error = OdiliaError;
	type Future = impl Future<Output = Result<Self, Self::Error>>;
	#[tracing::instrument(skip(state), ret)]
	fn try_from_state(state: Arc<ScreenReaderState>, event: E) -> Self::Future {
		async move {
			let a11y = AccessiblePrimitive::from_event(&event);
			let proxy = a11y.into_accessible(state.connection()).await?;
			let cache_item =
				state.cache.get_or_create(&proxy, Arc::clone(&state.cache)).await?;
			Ok(InnerEvent::new(event, cache_item))
		}
	}
}

impl<E, P> TryFromState<Arc<ScreenReaderState>, E> for EventPredicate<E, P>
where
	E: EventProperties + Debug + Clone,
	P: Predicate<(E, Arc<ScreenReaderState>)> + Debug,
{
	type Error = OdiliaError;
	type Future = impl Future<Output = Result<Self, Self::Error>>;
	#[tracing::instrument(skip(state), ret)]
	fn try_from_state(state: Arc<ScreenReaderState>, event: E) -> Self::Future {
		async move {
			let a11y = AccessiblePrimitive::from_event(&event);
			let proxy = a11y.into_accessible(state.connection()).await?;
			let cache_item =
				state.cache.get_or_create(&proxy, Arc::clone(&state.cache)).await?;
			let cache_event = InnerEvent::new(event.clone(), cache_item);
			EventPredicate::from_cache_event(cache_event, state).ok_or(
				OdiliaError::PredicateFailure(format!(
					"Predicate cache event {} failed for event {:?}",
					std::any::type_name::<P>(),
					event
				)),
			)
		}
	}
}

impl<E, P> EventProperties for EventPredicate<E, P>
where
	E: EventProperties + Debug,
	P: Predicate<(E, Arc<ScreenReaderState>)>,
{
	fn path(&self) -> ObjectPath<'_> {
		self.inner.path()
	}
	fn sender(&self) -> UniqueName<'_> {
		self.inner.sender()
	}
}
