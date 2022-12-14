mod document;
mod object;

use std::{collections::HashMap, rc::Rc};

use futures::stream::StreamExt;
use tokio::sync::{
    broadcast,
    mpsc::{Receiver, Sender},
};

use crate::state::ScreenReaderState;
use atspi::{
    accessible::Role,
    accessible_ext::{AccessibleExt, MatcherArgs},
    collection::MatchType,
    component::ScrollType,
    convertable::Convertable,
    events::Event,
    InterfaceSet,
};
use ssip_client::Priority;
use odilia_common::{
    events::{Direction, ScreenReaderEvent},
    modes::ScreenReaderMode,
};
use zbus::names::UniqueName;

pub async fn structural_navigation(
    state: &ScreenReaderState,
    dir: Direction,
    role: Role,
) -> zbus::Result<()> {
    let curr = match state.history_item(0).await? {
        Some(acc) => acc,
        None => return Ok(()),
    };
    let roles = vec![role];
    let attributes = HashMap::new();
    let interfaces = InterfaceSet::empty();
    let mt: MatcherArgs =
        (roles, MatchType::Invalid, attributes, MatchType::Invalid, interfaces, MatchType::Invalid);
    if let Some(next) = curr.get_next(&mt, dir == Direction::Backward).await? {
        let comp = next.to_component().await?;
        let texti = next.to_text().await?;
        let focused = comp.grab_focus().await?;
        comp.scroll_to(ScrollType::TopLeft).await?;
        let caret_offset = texti.set_caret_offset(0).await?;
        tracing::debug!("Focused: {}", focused);
        tracing::debug!("Caret offset: {}", caret_offset);
        state
            .update_accessible(
                UniqueName::try_from(next.destination().as_str())?,
                next.path().to_owned(),
            )
            .await;
        let role = next.get_role().await?;
        let len = texti.character_count().await?;
        let text = texti.get_text(0, len).await?;
        state.say(Priority::Text, format!("{text}, {role}")).await;
    } else {
        state.say(Priority::Text, format!("No more {role}s")).await;
    }
    Ok(())
}

pub async fn sr_event(
    state: Rc<ScreenReaderState>,
    sr_events: &mut Receiver<ScreenReaderEvent>,
    shutdown_rx: &mut broadcast::Receiver<i32>,
) -> eyre::Result<()> {
    loop {
        tokio::select! {
            sr_event = sr_events.recv() => {
                tracing::debug!("SR Event received");
                match sr_event {
                    Some(ScreenReaderEvent::StructuralNavigation(dir, role)) => {
                         if let Err(e) = structural_navigation(&state, dir, role).await {
                            tracing::debug!(error = %e, "There was an error with the structural navigation call.");
                        }
                    },
                    Some(ScreenReaderEvent::StopSpeech) => {
                      tracing::debug!("Stopping speech!");
                      let _ = state.stop_speech().await;
                    },
                    Some(ScreenReaderEvent::ChangeGranularity(granularity)) => {
                      tracing::debug!("Changing granularity of read text.");
                      let mut sr_granularity = state.granularity.lock().await;
                      *sr_granularity = granularity;
                    },
                    Some(ScreenReaderEvent::ChangeMode(ScreenReaderMode { name })) => {
                        tracing::debug!("Changing mode to {:?}", name);
                        //let _ = mode_channel.send(ScreenReaderMode { name }).await;
                    }
                    _ => { continue; }
                };
                continue;
            }
            _ = shutdown_rx.recv() => {
                tracing::debug!("sr_event cancelled");
                break;
            }
        }
    }
    Ok(())
}

//#[tracing::instrument(level = "debug"i, skip(state))]
pub async fn receive(
    state: Rc<ScreenReaderState>,
    tx: Sender<Event>,
    shutdown_rx: &mut broadcast::Receiver<i32>,
) {
    let events = state.atspi.event_stream();
    tokio::pin!(events);
    loop {
        tokio::select! {
            event = events.next() => {
                if let Some(Ok(good_event)) = event {
                    if let Err(e) = tx.send(good_event).await {
                        tracing::error!(error = %e, "Error sending atspi event");
                    }
                } else {
                    tracing::debug!("Event is either None or an Error variant.");
                }
                continue;
            }
            _ = shutdown_rx.recv() => {
                tracing::debug!("receive function is done");
                break;
            }
        }
    }
}

//#[tracing::instrument(level = "debug")]
pub async fn process(
    state: Rc<ScreenReaderState>,
    rx: &mut Receiver<Event>,
    shutdown_rx: &mut broadcast::Receiver<i32>,
) {
    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Some(good_event) => {
                        if let Err(e) = dispatch(&state, good_event).await {
                            tracing::error!(error = %e, "Could not handle event");
                        } else {
                            tracing::debug!("Event handled without error");
                        }
                    },
                    None => {
                        tracing::debug!("Event was none.");
                    }
                };
                continue;
            }
            _ = shutdown_rx.recv() => {
                tracing::debug!("process function is done");
                break;
            }
        }
    }
}

async fn dispatch(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
    // Dispatch based on interface
    if let Some(interface) = event.interface() {
        match interface
            .rsplit('.')
            .next()
            .expect("Interface name should contain '.'")
        {
            "Object" => object::dispatch(state, event).await?,
            "Document" => document::dispatch(state, event).await?,
            interface => tracing::debug!(interface, "Ignoring event with unknown interface"),
        }
    }
    Ok(())
}
