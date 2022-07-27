mod object;

use odilia_common::{
    events::ScreenReaderEvent,
    modes::ScreenReaderMode,
};
use futures::stream::{
    StreamExt
};
use tokio::sync::mpsc::{
  Sender,
  Receiver,
};

use atspi::events::Event;
use crate::state;
use crate::state::{
  ScreenReaderState
};

pub async fn sr_event(sr_events: &mut Receiver<ScreenReaderEvent>, mode_channel: Sender<ScreenReaderMode>) {
    println!("Waiting for sr event.");
    while let Some(sr_event) = sr_events.recv().await {
        match sr_event {
            ScreenReaderEvent::Next(ele_type) => {
              tracing::debug!("Look for the next {:?}", ele_type);
            },
            ScreenReaderEvent::StopSpeech => println!("Stop speech!"),
            ScreenReaderEvent::ChangeMode(ScreenReaderMode{ name }) => {
              tracing::debug!("Change mode to {:?}", name);
              let _ = mode_channel.send(ScreenReaderMode{ name }).await;
            },
            _ => {}
        }
    }
}

#[tracing::instrument(level = "debug")]
pub async fn process() {
    let events = state::get_event_stream().await;
    pin_utils::pin_mut!(events);
    while let Some(Ok(event)) = events.next().await {
        if let Err(e) = dispatch(event).await {
            tracing::error!(error = %e, "Could not handle event");
        } else {
            tracing::debug!("Event handled without error");
        }
    }
}

async fn dispatch(event: Event) -> eyre::Result<()> {
        // Dispatch based on interface
        if let Some(interface) = event.interface() {
        match interface.rsplit('.').next().expect("Interface name should contain '.'") {
            "Object" => object::dispatch(event).await?,
            interface => tracing::debug!(interface, "Ignoring event with unknown interface"),
    }
    }
        Ok(())
}
