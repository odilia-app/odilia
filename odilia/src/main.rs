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

use std::process::exit;

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    logging::init();
    let (sr_event_tx, mut sr_event_rx) = channel(8);
    let _args = args::parse();
    // Initialize state
    let state = Rc::new(ScreenReaderState::new().await?);

    // Register events
    state.register_event("Object:StateChanged:Focused").await?;
    state.register_event("Object:TextCaretMoved").await?;
    state.register_event("Document:LoadComplete").await?;

    let atspi_event_future = events::process(Rc::clone(&state)).map(|_| Ok::<_, eyre::Report>(()));
    let odilia_event_future = events::sr_event(Rc::clone(&state), sr_event_rx).map(|r| r.wrap_err("Could not process Odilia events"));
    tokio::try_join!(atspi_event_future, odilia_event_future)?;
    Ok(())
}
