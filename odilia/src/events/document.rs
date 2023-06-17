use crate::state::ScreenReaderState;
use atspi_common::events::{
	document::{DocumentEvents, LoadCompleteEvent},
	GenericEvent,
};
use odilia_common::{
  errors::OdiliaError,
  events::{ScreenReaderEvent, CacheEvent},
};

pub async fn load_complete(
	event: &LoadCompleteEvent,
) -> Result<Vec<ScreenReaderEvent>, OdiliaError> {
  Ok(vec![ScreenReaderEvent::Cache(CacheEvent::LoadAll((event.item.name.to_string(), event.item.path.clone())))])
//	let sender = event.sender();
//	let cache = state.build_cache(sender).await?;
//	// TODO: this should be streamed, rather than waiting for the entire vec to fill up.
//	let entire_cache = cache.get_items().await?;
//	for item in entire_cache {
//		state.get_or_create_atspi_cache_item_to_cache(item).await?;
//	}
//	tracing::debug!("Add an entire document to cache.");
//	Ok(())
}

pub async fn dispatch(state: &ScreenReaderState, event: &DocumentEvents) -> eyre::Result<Vec<ScreenReaderEvent>> {
	// Dispatch based on member
	Ok(match event {
		DocumentEvents::LoadComplete(load_complete_event) => {
			load_complete(load_complete_event).await?
		}
		other_member => {
			tracing::debug!("Ignoring event with unknown member: {:#?}", other_member);
      vec![]
		}
	})
}
