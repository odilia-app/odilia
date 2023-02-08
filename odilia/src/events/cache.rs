use crate::ScreenReaderState;
use atspi::events::{
  GenericEvent,
	CacheEvents,
	AddAccessibleEvent,
	RemoveAccessibleEvent,
};
use odilia_cache::atspi_cache_item_to_odilia_cache_item;

pub async fn dispatch(state: &ScreenReaderState, event: &CacheEvents) -> eyre::Result<()> {
	match event {
		CacheEvents::Add(add_event) => add_accessible(state, add_event).await?,
		CacheEvents::Remove(rem_event) => remove_accessible(state, rem_event).await?,
	}
	Ok(())
}

pub async fn add_accessible(state: &ScreenReaderState, event: &AddAccessibleEvent) -> eyre::Result<()> {
	let cache_item = atspi_cache_item_to_odilia_cache_item(state.atspi.connection(), event.to_owned().into_item()).await?;
	state.cache.add(cache_item).await;
	Ok(())
}
pub async fn remove_accessible(state: &ScreenReaderState, event: &RemoveAccessibleEvent) -> eyre::Result<()> {
	let id = event.path().expect("Could not get path for remove accessible event; this should never happen.").try_into()?;
	state.cache.remove(&id).await;
	Ok(())
}
