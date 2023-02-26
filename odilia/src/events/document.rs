use crate::state::ScreenReaderState;
use atspi::{
	events::GenericEvent, identify::document::DocumentEvents,
	identify::document::LoadCompleteEvent,
};
use odilia_cache::CacheItem;

pub async fn load_complete(
	state: &ScreenReaderState,
	event: &LoadCompleteEvent,
) -> eyre::Result<()> {
	let sender = event.sender()?.unwrap();
	let cache = state.build_cache(sender.clone()).await?;
	// TODO: this should be streamed, rather than waiting for the entire vec to fill up.
	let entire_cache = cache.get_items().await?;
	for item in entire_cache {
		state.cache.add(CacheItem {
			object: item
				.object
				.try_into()
				.expect("Could not create AccessiblePrimitive from parts"),
			app: item.app.try_into().expect(
				"Could not create AccessiblePrimitive from parts for application",
			),
			parent: item.parent.try_into().expect(
				"Could not create AccessiblePrimitive from parts for parent",
			),
			index: item.index,
			children: item.children,
			ifaces: item.ifaces,
			role: item.role,
			states: item.states,
			text: item.name.clone(),
		}).await;
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
