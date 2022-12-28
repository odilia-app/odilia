use zbus::zvariant::ObjectPath;
use odilia_cache::CacheItem;

use crate::state::ScreenReaderState;
use atspi::events::Event;

pub fn get_id_from_path(path: &str) -> Option<i32> {
	tracing::debug!("Attempting to get ID for: {}", path);
	if let Some(id) = path.split('/').next_back() {
		if let Ok(uid) = id.parse::<i32>() {
			return Some(uid);
		} else if id == "root" {
			return Some(0);
		} else if id == "null" {
			return Some(-1);
		}
	}
	None
}

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
		let path = item.object.1.to_string();
		let app_path = item.app.1.clone();
		let parent_path = item.parent.1.clone();
		let object_id =
			get_id_from_path(&path).expect("There should always be an accessible ID");
		let app_id = get_id_from_path(&app_path)
			.expect("There should always be an accessible ID");
		let parent_id = get_id_from_path(&parent_path)
			.expect("There should always be an accessible ID");
		cache_items.push(CacheItem {
			object: object_id,
			app: app_id,
			parent: parent_id,
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
