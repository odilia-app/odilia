use zbus::zvariant::ObjectPath;
use odilia_cache::CacheItem;

use crate::state::ScreenReaderState;
use atspi::events::Event;

pub async fn load_complete(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
	let sender = event.sender()?.unwrap();
	let cache = state
		.build_cache(
			sender.clone(),
			ObjectPath::try_from("/org/a11y/atspi/cache".to_string())?,
		)
		.await?;
	let entire_cache = cache.get_items().await?;
	let mut cache_items = Vec::new();
	for item in entire_cache {
		cache_items.push(CacheItem {
			object: item.object.try_into().unwrap(),
			app: item.app.try_into().unwrap(),
			parent: item.parent.try_into().unwrap(),
			index: item.index,
			children: item.children,
			ifaces: item.ifaces.into(),
			role: item.role.into(),
			states: item.states.into(),
			text: item.name.clone(),
		});
	}
	state.cache.add_all(cache_items).await;
	tracing::debug!("Add an entire document to cache.");
	Ok(())
}

pub async fn dispatch(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
	// Dispatch based on member
	if let Some(member) = event.member() {
		match member.as_str() {
			"LoadComplete" => load_complete(state, event).await?,
			member => tracing::debug!(member, "Ignoring event with unknown member"),
		}
	}
	Ok(())
}
