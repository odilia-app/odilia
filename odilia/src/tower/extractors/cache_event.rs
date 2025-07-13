use std::{fmt::Debug, future::Future, marker::PhantomData, ops::Deref, pin::Pin, sync::Arc};

use atspi::{Event, EventProperties};
use odilia_cache::CacheItem;
use zbus::{names::UniqueName, zvariant::ObjectPath};

use crate::{
	tower::{from_state::TryFromState, predicate::CONTAINER_ROLES, Predicate},
	OdiliaError, ScreenReaderState,
};

pub type CacheEvent<E> = EventPredicate<E, Always>;
pub type NonContainerEvent<E> = EventPredicate<E, NotContainer>;
pub type ActiveAppEvent<E> = EventPredicate<E, ActiveApplication>;

#[derive(Debug, Clone)]
pub struct InnerEvent<E: EventProperties + Debug> {
	pub inner: E,
	pub item: CacheItem,
}

//impl<E: EventProperties + Debug> Deref for InnerEvent<E> {
//	type Target = E;
//	fn deref(&self) -> &Self::Target {
//		&self.inner
//	}
//}

impl<E> InnerEvent<E>
where
	E: EventProperties + Debug,
{
	fn new(inner: E, item: CacheItem) -> Self {
		Self { inner, item }
	}
}

#[derive(Debug, Clone)]
pub struct EventPredicate<
	E: EventProperties + Debug,
	P: Predicate<(CacheItem, Arc<ScreenReaderState>)>,
> {
	pub inner: E,
	pub item: CacheItem,
	_marker: PhantomData<P>,
}

impl<E: EventProperties + Debug, P: Predicate<(CacheItem, Arc<ScreenReaderState>)>> Deref
	for EventPredicate<E, P>
{
	type Target = E;
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}
impl<E, P> EventPredicate<E, P>
where
	E: EventProperties + Debug + Clone,
	P: Predicate<(CacheItem, Arc<ScreenReaderState>)>,
{
	pub fn from_cache_event(ce: InnerEvent<E>, state: Arc<ScreenReaderState>) -> Option<Self> {
		if P::test(&(ce.item.clone(), state)) {
			return Some(Self { inner: ce.inner, item: ce.item, _marker: PhantomData });
		}
		None
	}
}

#[derive(Debug)]
pub struct NotContainer;
impl Predicate<(CacheItem, Arc<ScreenReaderState>)> for NotContainer {
	fn test((ci, _): &(CacheItem, Arc<ScreenReaderState>)) -> bool {
		!CONTAINER_ROLES.contains(&ci.role)
	}
}

#[derive(Debug)]
pub struct Always;
impl Predicate<(CacheItem, Arc<ScreenReaderState>)> for Always {
	fn test(_: &(CacheItem, Arc<ScreenReaderState>)) -> bool {
		true
	}
}

#[derive(Debug)]
pub struct ActiveApplication;
impl Predicate<(CacheItem, Arc<ScreenReaderState>)> for ActiveApplication {
	fn test((ci, state): &(CacheItem, Arc<ScreenReaderState>)) -> bool {
		let Some(last_focused) = state.history_item(0) else {
			return false;
		};
		last_focused == ci.app
	}
}

impl<E> TryFromState<Arc<ScreenReaderState>, E> for InnerEvent<E>
where
	E: EventProperties + Into<Event> + Debug + Clone + Send + Sync + Unpin + 'static,
{
	type Error = OdiliaError;
	type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>>;
	#[tracing::instrument(skip(state), ret)]
	fn try_from_state(state: Arc<ScreenReaderState>, event: E) -> Self::Future {
		Box::pin(async move {
			let cache_item = state.cache_from_event(event.clone().into()).await?;
			Ok(InnerEvent::new(event, cache_item))
		})
	}
}

impl<E, P> TryFromState<Arc<ScreenReaderState>, E> for EventPredicate<E, P>
where
	E: EventProperties + Into<Event> + Debug + Clone + Send + Sync + 'static,
	P: Predicate<(CacheItem, Arc<ScreenReaderState>)> + Debug,
{
	type Error = OdiliaError;
	type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>> + Send + 'static>>;
	#[tracing::instrument(skip(state), ret)]
	fn try_from_state(state: Arc<ScreenReaderState>, event: E) -> Self::Future {
		Box::pin(async move {
			let event_any: Event = event.clone().into();
			let cache_item = state.cache_from_event(event_any).await?;
			let cache_event = InnerEvent::new(event.clone(), cache_item);
			EventPredicate::from_cache_event(cache_event, state).ok_or(
				OdiliaError::PredicateFailure(format!(
					"Predicate cache event {} failed for event {:?}",
					std::any::type_name::<P>(),
					event
				)),
			)
		})
	}
}

impl<E, P> EventProperties for EventPredicate<E, P>
where
	E: EventProperties + Debug,
	P: Predicate<(CacheItem, Arc<ScreenReaderState>)>,
{
	fn path(&self) -> ObjectPath<'_> {
		self.inner.path()
	}
	fn sender(&self) -> UniqueName<'_> {
		self.inner.sender()
	}
}

impl<E, P> From<EventPredicate<E, P>> for Event
where
	E: Into<Event> + Debug + EventProperties,
	P: Predicate<(CacheItem, Arc<ScreenReaderState>)>,
{
	fn from(pred: EventPredicate<E, P>) -> Self {
		pred.inner.into()
	}
}
