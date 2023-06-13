use crate::state::ScreenReaderState;
use atspi_common::events::{
	document::{DocumentEvents, LoadCompleteEvent},
	GenericEvent,
};
use odilia_common::errors::OdiliaError;

pub async fn load_complete(
	state: &ScreenReaderState,
	event: &LoadCompleteEvent,
) -> Result<(), OdiliaError> {
	let sender = event.sender();
	let cache = state.build_cache(sender).await?;
	// TODO: this should be streamed, rather than waiting for the entire vec to fill up.
	let entire_cache = cache.get_items().await?;
	for item in entire_cache {
		state.get_or_create_atspi_cache_item_to_cache(item).await?;
	}
	tracing::debug!("Add an entire document to cache.");
	Ok(())
}

pub async fn dispatch(state: &ScreenReaderState, event: &DocumentEvents) -> eyre::Result<()> {
	// Dispatch based on member
	match event {
		DocumentEvents::LoadComplete(load_complete_event) => {
			load_complete(state, load_complete_event).await?;
		}
		other_member => {
			tracing::debug!("Ignoring event with unknown member: {:#?}", other_member);
		}
	}
	Ok(())
}
