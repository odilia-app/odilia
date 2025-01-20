use crate::{tower::from_state::TryFromState, OdiliaError, ScreenReaderState};
use atspi::{EventProperties, RelationType};
use core::{fmt::Debug, future::Future};
use odilia_cache::CacheItem;
use std::sync::Arc;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct RelationSet(pub Vec<(RelationType, Vec<CacheItem>)>);

impl From<Vec<(RelationType, Vec<CacheItem>)>> for RelationSet {
	fn from(rs: Vec<(RelationType, Vec<CacheItem>)>) -> RelationSet {
		RelationSet(rs)
	}
}

async fn try_from_state<E>(
	state: Arc<ScreenReaderState>,
	event: E,
) -> Result<RelationSet, OdiliaError>
where
	E: EventProperties,
{
	state.get_or_create_event_object_to_cache::<E>(&event)
		.await?
		.get_relation_set()
		.await
		.map(RelationSet::from)
}

try_from_state_event_fn!(try_from_state, RelationSet);
