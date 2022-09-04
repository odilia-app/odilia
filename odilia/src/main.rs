mod args;
mod cache;
mod events;
mod logging;
mod state;

use std::rc::Rc;

use eyre::WrapErr;
use futures::future::FutureExt;
use tokio::sync::mpsc::channel;

use atspi::accessible::Role;
use crate::state::ScreenReaderState;
use odilia_common::{
    events::{Direction, ScreenReaderEvent},
    input::{Key, KeyBinding, Modifiers},
    modes::ScreenReaderMode,
};
use odilia_input::sr_event_receiver;
use serde_json;

use std::process::exit;

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    logging::init();
    let _args = args::parse();
    let change_mode = ScreenReaderEvent::ChangeMode(ScreenReaderMode{ name: "Browse".to_string()});
    let sn = ScreenReaderEvent::StructuralNavigation(Direction::Forward, Role::Heading);
    tracing::debug!("CM {:?}", serde_json::to_string(&change_mode).unwrap());
    tracing::debug!("SN {:?}", serde_json::to_string(&sn).unwrap());
    // Initialize state
    let state = Rc::new(ScreenReaderState::new().await?);

    let (sr_event_tx, mut sr_event_rx) = channel(8);
    // this channel must NEVER fill up; it will cause the thread receiving events to deadlock due to a zbus design choice.
    // If you need to make it bigger, then make it bigger, but do NOT let it ever fill up.
    let (atspi_event_tx, mut atspi_event_rx) = channel(64);

    // Register events
    state.register_event("Object:StateChanged:Focused").await?;
    state.register_event("Object:TextCaretMoved").await?;
    state.register_event("Document:LoadComplete").await?;

    let atspi_event_receiver = events::receive(Rc::clone(&state), atspi_event_tx).map(|_| Ok::<_, eyre::Report>(()));
    let atspi_event_processor = events::process(Rc::clone(&state), &mut atspi_event_rx).map(|_| Ok::<_, eyre::Report>(()));
    let odilia_event_receiver = sr_event_receiver(sr_event_tx).map(|r| r.wrap_err("Could not process Odilia events"));
    let odilia_event_processor = events::sr_event(Rc::clone(&state), &mut sr_event_rx).map(|r| r.wrap_err("Could not process Odilia event"));
    tokio::try_join!(atspi_event_receiver, atspi_event_processor, odilia_event_receiver, odilia_event_processor)?;
    Ok(())
}
