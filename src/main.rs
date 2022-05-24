mod args;
mod logging;
mod state;
use state::ScreenReaderState;

use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::init();
    let _args = args::parse();
    let state = ScreenReaderState::new().await?;
    state.register_event("Object:StateChanged:Focused").await?;
    process_events(&state).await;
    Ok(())
}

#[tracing::instrument(level = "debug", skip(state))]
async fn process_events(state: &ScreenReaderState) {
    let events = state.atspi.event_stream();
    pin_utils::pin_mut!(events);
    while let Some(res) = events.next().await {
        let event = match res {
            Ok(e) => e,
            Err(e) => {
                tracing::error!(error = %e, "Error receiving atspi event");
                continue;
            },
        };
        tracing::debug!(kind = %event.kind(), "Got event");
    }
}
