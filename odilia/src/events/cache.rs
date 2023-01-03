use crate::ScreenReaderState;
use odilia_cache::AccessiblePrimitive;
use atspi::events::{
	CacheEvent,
	AddAccessibleEvent,
	RemoveAccessibleEvent,
};

pub async fn dispatch(state: &ScreenReaderState, event: &CacheEvent) -> eyre::Result<()> {
	match event {
		CacheEvent::Add(add_event) => add_accessible(state, add_event).await?,
		CacheEvent::Remove(rem_event) => remove_accessible(state, rem_event).await?,
	}
	Ok(())
}

pub async fn add_accessible(state: &ScreenReaderState, event: &AddAccessibleEvent) -> eyre::Result<()> {
	let cache_item = event.to_owned().into_item().try_into()?;
	state.cache.add(cache_item).await;
	Ok(())
}
pub async fn remove_accessible(state: &ScreenReaderState, event: &RemoveAccessibleEvent) -> eyre::Result<()> {
	let accessible_prim: AccessiblePrimitive = event.to_owned().into_accessible().try_into()?;
	state.cache.remove(&accessible_prim.id).await;
	Ok(())
}
