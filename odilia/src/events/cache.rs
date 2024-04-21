use crate::ScreenReaderState;
use atspi::events::{
	AddAccessibleEvent, CacheEvents, LegacyAddAccessibleEvent, RemoveAccessibleEvent,
};
use odilia_cache::AccessiblePrimitive;

#[tracing::instrument(level = "debug", skip(state), ret, err)]
pub async fn dispatch(state: &ScreenReaderState, event: &CacheEvents) -> eyre::Result<()> {
	match event {
		CacheEvents::Add(add_event) => add_accessible(state, add_event).await?,
		CacheEvents::LegacyAdd(ladd_event) => {
			legacy_add_accessible(state, ladd_event).await?
		}
		CacheEvents::Remove(rem_event) => remove_accessible(state, rem_event)?,
	}
	Ok(())
}

#[tracing::instrument(level = "debug", skip(state), ret, err)]
pub async fn add_accessible(
	state: &ScreenReaderState,
	event: &AddAccessibleEvent,
) -> eyre::Result<()> {
	state.get_or_create_atspi_cache_item_to_cache(event.node_added.clone())
		.await?;
	Ok(())
}

#[tracing::instrument(level = "debug", skip(state), ret, err)]
pub async fn legacy_add_accessible(
	state: &ScreenReaderState,
	event: &LegacyAddAccessibleEvent,
) -> eyre::Result<()> {
	state.get_or_create_atspi_legacy_cache_item_to_cache(event.node_added.clone())
		.await?;
	Ok(())
}

#[tracing::instrument(level = "debug", skip(state), ret, err)]
pub fn remove_accessible(
	state: &ScreenReaderState,
	event: &RemoveAccessibleEvent,
) -> eyre::Result<()> {
	let accessible_prim: AccessiblePrimitive = AccessiblePrimitive::from_event(event)?;
	state.cache.remove(&accessible_prim);
	Ok(())
}
