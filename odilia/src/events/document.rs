use crate::state::ScreenReaderState;
use atspi::{
	events::GenericEvent, identify::document::DocumentEvents,
	identify::document::LoadCompleteEvent, accessible::AccessibleProxy,
};
use odilia_cache::{CacheItem, AccessiblePrimitive};
use odilia_common::errors::AccessiblePrimitiveConversionError;

use std::sync::Arc;

pub async fn load_complete(
	state: &ScreenReaderState,
	event: &LoadCompleteEvent,
) -> eyre::Result<()> {
	let sender = event.sender()?.unwrap();
	let cache = state.build_cache(sender.clone()).await?;
	// TODO: this should be streamed, rather than waiting for the entire vec to fill up.
	let entire_cache = cache.get_items().await?;
	for item in entire_cache {
		let odilia_cache_item = CacheItem::from_atspi_cache_item(item, Arc::clone(&state.cache), state.atspi.connection()).await?;
		state.cache.add(odilia_cache_item).await;
	}
	tracing::debug!("Add an entire document to cache.");
	Ok(())
}

pub async fn dispatch(state: &ScreenReaderState, event: &DocumentEvents) -> eyre::Result<()> {
	// Dispatch based on member
	match event {
		DocumentEvents::LoadComplete(load_complete_event) => {
			load_complete(state, load_complete_event).await?
		}
		other_member => {
			tracing::debug!("Ignoring event with unknown member: {:#?}", other_member)
		}
	}
	Ok(())
}
