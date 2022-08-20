use crate::state;
use atspi::events::Event;

pub async fn load_complete(event: Event) -> eyre::Result<()> {
    let path = event.path().unwrap();
    let dest = event.sender()?.unwrap();
    let cache = state::get_cache(
        dest, path
    ).await?;
    let entire_cache = cache.get_items().await?;
    tracing::debug!("LOAD COMPLETE: {:?}", entire_cache);
    Ok(())
}

pub async fn dispatch(event: Event) -> eyre::Result<()> {
    // Dispatch based on member
    if let Some(member) = event.member() {
        match member.as_str() {
            "LoadComplete" => load_complete(event).await?,
            member => tracing::debug!(member, "Ignoring event with unknown member"),
        }
    }
    Ok(())
}

