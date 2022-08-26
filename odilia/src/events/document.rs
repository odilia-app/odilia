use zbus::zvariant::ObjectPath;

use atspi::events::Event;
use crate::state::ScreenReaderState;

pub async fn load_complete(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
    let dest = event.sender()?.unwrap();
    let cache = state.build_cache(
        dest, ObjectPath::try_from("/org/a11y/atspi/cache".to_string())?
    ).await?;
    let entire_cache = cache.get_items().await?;
    let write_by_id = &state.cache.by_id_write;
    let mut write_by_id = write_by_id.lock().await;
    for item in entire_cache {
        // defined in xml/Cache.xml
        let path = item.0.1.to_string();
        let dest = item.0.0.to_string();
        if let Some(id) = path.split('/').next_back() {
            if let Ok(uid) = id.parse::<u32>() {
                tracing::trace!(id=uid, path, dest, "Caching item");
                write_by_id.insert(uid, (path, dest));
            }
        }
    }
    write_by_id.refresh();
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

