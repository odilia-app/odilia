use zbus::{
	zvariant::ObjectPath,
	names::UniqueName,
};
use odilia_cache::{CacheItem, AccessiblePrimitive};

use crate::state::ScreenReaderState;
use atspi::{
	accessible::AccessibleProxy,
	events::GenericEvent,
	identify::document::DocumentEvents,
	identify::document::LoadCompleteEvent,
};

pub async fn load_complete(state: &ScreenReaderState, event: &LoadCompleteEvent) -> eyre::Result<()> {
	println!("LOAD COMPLETE STARTS");
	let sender = event.sender()?.unwrap();
	println!("GOT SENDER");
	let accessible: AccessibleProxy = AccessiblePrimitive::from_event(event).unwrap().into_accessible(state.connection()).await?;
	println!("Turned into accessible");
	let app: AccessiblePrimitive = accessible.get_application().await?.try_into()?;
	println!("Became a11y_prim");
	let application_proxy = app.into_accessible(state.connection()).await?;
	println!("Became proxy");
	let name_of_dest = application_proxy.destination().to_string();
	println!("Got dest");
	let cache = state.build_cache(
		UniqueName::try_from(sender.clone())?).await?;
	println!("Built cache");
	let entire_cache = cache.get_items().await?;
	println!("Cache items got!");
	let mut cache_items = Vec::new();
	for item in entire_cache {
		println!("NEW CACHE ITEM: {:#?}", item.object.1);
		cache_items.push(CacheItem {
			object: item.object.try_into().unwrap(),
			app: item.app.try_into().unwrap(),
			parent: item.parent.try_into().unwrap(),
			index: item.index,
			children: item.children,
			ifaces: item.ifaces,
			role: item.role,
			states: item.states,
			text: item.name.clone(),
		});
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
