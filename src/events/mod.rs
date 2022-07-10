mod object;

use futures::stream::StreamExt;

use atspi::events::Event;
use crate::state::ScreenReaderState;

#[tracing::instrument(level = "debug", skip(state))]
pub async fn process(state: &ScreenReaderState) {
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
        if let Err(e) = dispatch(state, event).await {
            tracing::error!(error = %e, "Could not handle event");
        }
    }
}

async fn dispatch(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
        // Dispatch based on interface
        if let Some(interface) = event.interface() {
        match interface.rsplit('.').next().expect("Interface name should contain '.'") {
            "Object" => object::dispatch(state, event).await?,
            interface => tracing::debug!(interface, "Ignoring event with unknown interface"),
    }
    }
        Ok(())
}
