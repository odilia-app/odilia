mod object;

use futures::stream::StreamExt;
use odilia_common::{
    events::{Direction, ScreenReaderEvent},
    modes::ScreenReaderMode,
};
use speech_dispatcher::Priority;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::state;
use atspi::{
    accessible_plus::{AccessiblePlus, MatcherArgs},
    collection::MatchType,
    convertable::Convertable,
    events::Event,
};
use std::collections::HashMap;

pub async fn sr_event(
    sr_events: &mut Receiver<ScreenReaderEvent>,
    mode_channel: Sender<ScreenReaderMode>,
) -> zbus::Result<()> {
    println!("Waiting for sr event.");
    while let Some(sr_event) = sr_events.recv().await {
        let _event_result = match sr_event {
            ScreenReaderEvent::StructuralNavigation(dir, role) => {
                let curr = state::get_accessible_history(0).await?;
                let direction = match dir {
                    Direction::Forward => false,
                    Direction::Backward => true,
                };
                let roles = vec![role];
                let attributes = HashMap::new();
                let interfaces = Vec::new();
                let mt: MatcherArgs = (
                    roles,
                    MatchType::Invalid,
                    attributes,
                    MatchType::Invalid,
                    interfaces,
                    MatchType::Invalid,
                );
                if let Some(next) = curr.get_next(&mt, direction).await? {
                    let text = next.to_text().await?;
                    text.set_caret_offset(0).await?;
                } else {
                    state::say(Priority::Text, "No more headings".to_string()).await;
                }
            }
            ScreenReaderEvent::StopSpeech => println!("Stop speech!"),
            ScreenReaderEvent::ChangeMode(ScreenReaderMode { name }) => {
                tracing::debug!("Change mode to {:?}", name);
                let _ = mode_channel.send(ScreenReaderMode { name }).await;
            }
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
        match interface
            .rsplit('.')
            .next()
            .expect("Interface name should contain '.'")
        {
            "Object" => object::dispatch(event).await?,
            interface => tracing::debug!(interface, "Ignoring event with unknown interface"),
        }
    }
    Ok(())
}
