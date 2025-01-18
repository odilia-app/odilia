use crate::{tower::from_state::TryFromState, OdiliaError, ScreenReaderState};
use atspi::{EventProperties, RelationType};
use core::{fmt::Debug, future::Future, ops::Deref};
use odilia_cache::CacheItem;
use std::sync::Arc;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct RelationSet(Vec<(RelationType, Vec<CacheItem>)>);

impl From<Vec<(RelationType, Vec<CacheItem>)>> for RelationSet {
	fn from(rs: Vec<(RelationType, Vec<CacheItem>)>) -> RelationSet {
		RelationSet(rs)
	}
}

impl Deref for RelationSet {
	type Target = Vec<(RelationType, Vec<CacheItem>)>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<E> TryFromState<Arc<ScreenReaderState>, E> for RelationSet
where
	E: EventProperties + Debug,
{
	type Error = OdiliaError;
	type Future = impl Future<Output = Result<Self, Self::Error>>;
	fn try_from_state(state: Arc<ScreenReaderState>, event: E) -> Self::Future {
		async move {
			state.get_or_create_event_object_to_cache::<E>(&event)
				.await?
				.get_relation_set()
				.await
				.map(RelationSet::from)
		}
	}
}
