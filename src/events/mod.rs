mod object;

use futures::stream::{
    Stream,
    StreamExt
};

use atspi::events::Event;
use crate::state::ScreenReaderState;
use tokio::sync::mpsc::{
    Sender,
    Receiver
};

#[tracing::instrument(level = "debug", skip(state, rx))]
pub async fn event_listener(rx: &mut Receiver<Event>, state: &ScreenReaderState) {
    while let Some(event) = rx.recv().await {
        if let Err(e) = dispatch(state, event).await {
            tracing::error!(error = %e, "Could not handle event");
        } else {
            tracing::debug!("Event handled without error");
        }
    }
}

#[tracing::instrument(level = "debug", skip(tx, events))]
pub async fn process(tx: &Sender<Event>, events: impl Stream<Item=zbus::Result<Event>>) {
    //let events = state.atspi.event_stream();
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
        if let Err(_) = tx.send(event).await {
            tracing::error!("Receiver dropped.");
            return;
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
