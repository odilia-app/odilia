mod object;
mod document;

use std::collections::HashMap;

use futures::stream::StreamExt;
use speech_dispatcher::Priority;
use tokio::sync::mpsc::{Receiver, Sender};

use atspi::{
    accessible::Role,
    accessible_plus::{AccessiblePlus, MatcherArgs},
    collection::MatchType,
    convertable::Convertable,
    events::Event,
};
use crate::state;
use odilia_common::{
    events::{Direction, ScreenReaderEvent},
    modes::ScreenReaderMode,
};

pub async fn structural_navigation(dir: Direction, role: Role) -> zbus::Result<()> {
    let curr = state::get_accessible_history(0).await?;
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
    if let Some(next) = curr.get_next(&mt, dir == Direction::Backward).await? {
        let text = next.to_text().await?;
        text.set_caret_offset(0).await?;
    } else {
        state::say(Priority::Text, "No more headings".to_string()).await;
    }
    Ok(())
}

pub async fn sr_event(
    sr_events: &mut Receiver<ScreenReaderEvent>,
    mode_channel: Sender<ScreenReaderMode>,
) -> zbus::Result<()> {
    loop {
        let sr_event = match sr_events.recv().await {
            Some(e) => e,
            _ => continue
        };

        match sr_event {
            ScreenReaderEvent::StructuralNavigation(dir, role) => {
                if let Err(e) = structural_navigation(dir, role).await {
                    tracing::debug!(error = %e, "There was an error with the structural navigation call.");
                }
            },
            ScreenReaderEvent::StopSpeech => tracing::trace!("Stopping speech!"),
            ScreenReaderEvent::ChangeMode(ScreenReaderMode { name }) => {
                tracing::debug!("Changing mode to {:?}", name);
                let _ = mode_channel.send(ScreenReaderMode { name }).await;
            }
            _ => {}
        };
    }
}

#[tracing::instrument(level = "debug")]
pub async fn process() {
    let events = state::get_event_stream().await;
    pin_utils::pin_mut!(events);
    loop {
        match events.next().await {
            Some(Ok(event)) => {
                if let Err(e) = dispatch(event).await {
                    tracing::error!(error = %e, "Could not handle event");
                }
            },
            _ => tracing::debug!("Event is none"),
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
            "Document" => document::dispatch(event).await?,
            interface => tracing::debug!(interface, "Ignoring event with unknown interface"),
        }
    }
    Ok(())
}
