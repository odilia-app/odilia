mod args;
mod logging;
mod state;
use state::ScreenReaderState;

use futures::stream::StreamExt;

use atspi::events::Event;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::init();
    let _args = args::parse();
    let state = ScreenReaderState::new().await?;
    state.register_event("Object:StateChanged:Focused").await?;
    process_events(&state).await?;
    Ok(())
}

#[tracing::instrument(level = "debug", skip(state))]
async fn process_events(state: &ScreenReaderState) -> eyre::Result<()> {
    let events = state.atspi.event_stream();
    pin_utils::pin_mut!(events);
    while let Some(res) = events.next().await {
        let event = match res {
            Ok(e) => e,
            Err(e) => {
                tracing::error!(error = %e, "Error receiving atspi event");
                continue;
            }
        };
        tracing::debug!(kind = %event.kind(), "Got event");
        // Dispatch based on interface
        if let Some(interface) = event.interface() {
        match interface.rsplit('.').next().expect("Interface name should contain '.'") {
            "Object" => process_object_event(state, event).await?,
            interface => tracing::debug!(interface, "Ignoring event with unknown interface"),
    }
    }
    }
    Ok(())
}

async fn process_object_event(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
    // Dispatch based on member
    if let Some(member) = event.member() {
    match member.as_str() {
        "StateChanged" => process_state_changed_event(state, event).await?,
            member => tracing::debug!(member, "Ignoring event with unknown member"),
    }
    }
Ok(())
}

async fn process_state_changed_event(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
    // Dispatch based on kind
    match event.kind() {
        "focused" => process_focused_event(state, event).await?,
            kind => tracing::debug!(kind, "Ignoring event with unknown kind"),
    }
    Ok(())
}

async fn process_focused_event(state: &ScreenReaderState, event: Event) -> zbus::Result<()> {
    // Speak the newly focused object
    Ok(())
}
