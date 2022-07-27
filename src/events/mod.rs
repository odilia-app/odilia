mod object;

use odilia_common::{
    events::ScreenReaderEvent,
    modes::ScreenReaderMode,
};
use futures::stream::{
    StreamExt
};
use tokio::sync::mpsc::Receiver;

use atspi::events::Event;
use crate::state::ScreenReaderState;

pub async fn sr_event(sr_events: &mut Receiver<ScreenReaderEvent>) {
    println!("Waiting for sr event.");
    while let Some(sr_event) = sr_events.recv().await {
        match sr_event {
            ScreenReaderEvent::StopSpeech => println!("Stop speech!"),
            ScreenReaderEvent::ChangeMode(ScreenReaderMode{ name }) => println!("Change mode to {:?}", name),
            _ => {}
        }
    }
}

#[tracing::instrument(level = "debug", skip(state))]
pub async fn process(state: &ScreenReaderState) {
    let events = state.atspi.event_stream();
    pin_utils::pin_mut!(events);
    while let Some(Ok(event)) = events.next().await {
        if let Err(e) = dispatch(state, event).await {
            tracing::error!(error = %e, "Could not handle event");
        } else {
            tracing::debug!("Event handled without error");
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
