mod object;

use speech_dispatcher::Priority;
use odilia_common::{
    events::ScreenReaderEvent,
    modes::ScreenReaderMode,
};
use futures::{
    stream::{
      StreamExt,
    },
    Future,
};
use tokio::sync::mpsc::{
  Sender,
  Receiver,
};

use atspi::{
  events::Event,
  accessible::Role,
  accessible::AccessibleProxy,
  accessible_plus::AccessiblePlus,
  convertable::Convertable,
};
use crate::state;
use crate::state::{
  ScreenReaderState
};

pub async fn match_heading(acc: AccessibleProxy<'_>) -> zbus::Result<bool> {
  Ok(acc.get_role().await? == Role::Heading)
}

pub async fn sr_event(sr_events: &mut Receiver<ScreenReaderEvent>, mode_channel: Sender<ScreenReaderMode>) -> zbus::Result<()>{
    println!("Waiting for sr event.");
    while let Some(sr_event) = sr_events.recv().await {
        let event_result = match sr_event {
            ScreenReaderEvent::StructuralNavigation(dir,role) => {
              let curr = state::get_accessible_history(0).await?;
              if let Some(next) = curr.get_next(match_heading, false).await? {
                let text = next.to_text().await?;
                text.set_caret_offset(0).await?;
              } else {
                state::say(Priority::Text, "No more headings".to_string()).await;
              }
            },
            ScreenReaderEvent::StopSpeech => println!("Stop speech!"),
            ScreenReaderEvent::ChangeMode(ScreenReaderMode{ name }) => {
              tracing::debug!("Change mode to {:?}", name);
              let _ = mode_channel.send(ScreenReaderMode{ name }).await;
            },
            _ => {}
        };
    }
    Ok(())
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
