use zbus::zvariant::ObjectPath;

use crate::{
	state::ScreenReaderState,
	cache::CacheItem,
};
use atspi::events::Event;
use std::sync::Arc;

pub fn get_id_from_path<'a>(path: &str) -> Option<i32> {
	tracing::debug!("Attempting to get ID for: {}", path);
	if let Some(id) = path.split('/').next_back() {
		if let Ok(uid) = id.parse::<i32>() {
			return Some(uid);
		} else if (id == "root") {
			return Some(0);
		}
	}
	None
}

pub async fn load_complete(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
    let dest = event.sender()?.unwrap();
    let cache = state
        .build_cache(dest, ObjectPath::try_from("/org/a11y/atspi/cache".to_string())?)
        .await?;
    let entire_cache = cache.get_items().await?;
		let cache_items: Vec<CacheItem> = entire_cache.into_iter()
			.map(|item| {
        // defined in xml/Cache.xml
        let sender = item.object.0.clone();
        let path = item.object.1.to_string();
				let app_path = item.app.1.clone();
				let parent_path = item.parent.1.clone();
				let object_id = get_id_from_path(&path).expect("There should always be an accessible ID");
				let app_id = get_id_from_path(&app_path).expect("There should always be an accessible ID");
				let parent_id = get_id_from_path(&parent_path).expect("There should always be an accessible ID");
				CacheItem {
					object: object_id,
					app: app_id,
					parent: parent_id,
					index: item.index,
					children: item.children,
					ifaces: item.ifaces,
					role: item.role,
					states: item.states
				}})
			.collect();
    let write_by_id = &state.cache.by_id_write;
    let mut write_by_id = write_by_id.lock().await;
		cache_items.into_iter()
			.for_each(|cache_item| {
				write_by_id.insert(cache_item.object, cache_item);
			});
    write_by_id.refresh();
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
