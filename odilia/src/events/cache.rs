use crate::ScreenReaderState;
use atspi::events::{AddAccessibleEvent, CacheEvents, RemoveAccessibleEvent};
use odilia_cache::AccessiblePrimitive;

pub async fn dispatch(state: &ScreenReaderState, event: &CacheEvents) -> eyre::Result<()> {
	match event {
		CacheEvents::Add(add_event) => add_accessible(state, add_event).await?,
		CacheEvents::Remove(rem_event) => remove_accessible(state, rem_event)?,
	}
	Ok(())
}

pub async fn add_accessible(
	state: &ScreenReaderState,
	event: &AddAccessibleEvent,
) -> eyre::Result<()> {
	state.get_or_create_atspi_cache_item_to_cache(event.node_added.clone())
		.await?;
	Ok(())
}
pub fn remove_accessible(
	state: &ScreenReaderState,
	event: &RemoveAccessibleEvent,
) -> eyre::Result<()> {
	let accessible_prim: AccessiblePrimitive = AccessiblePrimitive::from_event(event)?;
	state.cache.remove(&accessible_prim);
	Ok(())
}
