use crate::ScreenReaderState;
use atspi::events::{AddAccessibleEvent, CacheEvents, RemoveAccessibleEvent};
use odilia_cache::{AccessiblePrimitive, CacheItem};
use std::sync::Arc;

pub async fn dispatch(state: &ScreenReaderState, event: &CacheEvents) -> eyre::Result<()> {
	match event {
		CacheEvents::Add(add_event) => add_accessible(state, add_event).await?,
		CacheEvents::Remove(rem_event) => remove_accessible(state, rem_event).await?,
	}
	Ok(())
}

pub async fn add_accessible(
	state: &ScreenReaderState,
	event: &AddAccessibleEvent,
) -> eyre::Result<()> {
	let atspi_cache_item = event
		.to_owned()
		.into_item();
	let odilia_cache_item = CacheItem::from_atspi_cache_item(
		atspi_cache_item,
		Arc::clone(&state.cache),
		state.atspi.connection()
	).await?;
	state.cache.add(odilia_cache_item).await;
	Ok(())
}
pub async fn remove_accessible(
	state: &ScreenReaderState,
	event: &RemoveAccessibleEvent,
) -> eyre::Result<()> {
	let accessible_prim: AccessiblePrimitive = event.to_owned().into_accessible().try_into()?;
	state.cache.remove(&accessible_prim).await;
	Ok(())
}
