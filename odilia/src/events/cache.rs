use crate::ScreenReaderState;
use atspi::events::{AddAccessibleEvent, CacheEvents, RemoveAccessibleEvent};
use odilia_cache::AccessiblePrimitive;

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
	let atspi_cache_item = event.to_owned().into_item();
	state.get_or_create_atspi_cache_item_to_cache(atspi_cache_item)
		.await?;
	Ok(())
}
pub async fn remove_accessible(
	state: &ScreenReaderState,
	event: &RemoveAccessibleEvent,
) -> eyre::Result<()> {
	let accessible_prim: AccessiblePrimitive = AccessiblePrimitive::from_event(event)?;
	state.cache.remove(&accessible_prim);
	Ok(())
}
