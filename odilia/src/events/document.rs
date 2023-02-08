use zbus::{
	names::UniqueName,
};
use odilia_cache::{
  CacheItem,
  atspi_cache_item_to_odilia_cache_item,
};

use crate::state::ScreenReaderState;
use atspi::{
  AccessibleId,
	events::GenericEvent,
	identify::document::DocumentEvents,
	identify::document::LoadCompleteEvent,
};

pub async fn load_complete(state: &ScreenReaderState, event: &LoadCompleteEvent) -> eyre::Result<()> {
	let sender = event.sender()?.unwrap();
	let cache = state.build_cache(
		UniqueName::try_from(sender.clone())?).await?;
	let entire_cache = cache.get_items().await?;
	let mut cache_items = Vec::new();
	for item in entire_cache {
    let odilia_cache_item = atspi_cache_item_to_odilia_cache_item(state.atspi.connection(), item).await?;
		cache_items.push(odilia_cache_item);
	}
	state.cache.add_all(cache_items).await;
	tracing::debug!("Add an entire document to cache.");
	Ok(())
}

pub async fn dispatch(state: &ScreenReaderState, event: &DocumentEvents) -> eyre::Result<()> {
	// Dispatch based on member
	match event {
		DocumentEvents::LoadComplete(load_complete_event) => load_complete(state, load_complete_event).await?,
		other_member => tracing::debug!("Ignoring event with unknown member: {:#?}", other_member),
	}
	Ok(())
}
