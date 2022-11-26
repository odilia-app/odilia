mod args;
mod cache;
mod events;
mod logging;
mod state;

use console_subscriber;

use std::rc::Rc;

use eyre::WrapErr;
use futures::future::FutureExt;
use tokio::{
    sync::mpsc,
    sync::broadcast,
    signal::unix::{
        SignalKind,
        signal,
    },
};

use atspi::accessible::Role;
use crate::state::ScreenReaderState;
use odilia_common::{
    events::{Direction, ScreenReaderEvent},
    modes::ScreenReaderMode,
};
use odilia_input::sr_event_receiver;
use serde_json;

async fn sigterm_signal_watcher(shutdown_tx: broadcast::Sender<i32>) -> eyre::Result<()>{
    let mut c = signal(SignalKind::interrupt())?;
    tracing::debug!("Watching for Ctrl+C");
    c.recv().await;
    tracing::debug!("Asking all processes to stop.");
    let _ = shutdown_tx.send(0);
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    logging::init();
    //console_subscriber::init();
    let _args = args::parse();
    let change_mode = ScreenReaderEvent::ChangeMode(ScreenReaderMode{ name: "Browse".to_string()});
    let sn = ScreenReaderEvent::StructuralNavigation(Direction::Forward, Role::Heading);
    let (shutdown_tx, _) = broadcast::channel(1);
    // Initialize state
    let state = Rc::new(ScreenReaderState::new().await?);

    let (sr_event_tx, mut sr_event_rx) = mpsc::channel(8);
    // this channel must NEVER fill up; it will cause the thread receiving events to deadlock due to a zbus design choice.
    // If you need to make it bigger, then make it bigger, but do NOT let it ever fill up.
    let (atspi_event_tx, mut atspi_event_rx) = mpsc::channel(128);

    // Register events
    state.register_event("Object:StateChanged:Focused").await?;
    state.register_event("Object:TextCaretMoved").await?;
    state.register_event("Document:LoadComplete").await?;

    let mut shutdown_rx_atspi_recv = shutdown_tx.subscribe();
    let atspi_event_receiver = events::receive(Rc::clone(&state), atspi_event_tx, &mut shutdown_rx_atspi_recv).map(|_| Ok::<_, eyre::Report>(()));
    let mut shutdown_rx_atspi_proc_recv = shutdown_tx.subscribe();
    let atspi_event_processor = events::process(Rc::clone(&state), &mut atspi_event_rx, &mut shutdown_rx_atspi_proc_recv).map(|_| Ok::<_, eyre::Report>(()));
    let mut shutdown_rx_odilia_recv = shutdown_tx.subscribe();
    let odilia_event_receiver = sr_event_receiver(sr_event_tx, &mut shutdown_rx_odilia_recv).map(|r| r.wrap_err("Could not process Odilia events"));
    let mut shutdown_rx_odilia_proc_recv = shutdown_tx.subscribe();
    let odilia_event_processor = events::sr_event(Rc::clone(&state), &mut sr_event_rx, &mut shutdown_rx_odilia_proc_recv).map(|r| r.wrap_err("Could not process Odilia event"));
    let signal_receiver = sigterm_signal_watcher(shutdown_tx).map(|r| r.wrap_err("Could not process signal shutdown."));
    tokio::try_join!(signal_receiver, atspi_event_receiver, atspi_event_processor, odilia_event_receiver, odilia_event_processor)?;
    tracing::debug!("All listeners have stopped. Running cleanup code.");
    let _ = state.speaker.cancel_all();
    if let Ok(_) = state.speaker.stop() {
        tracing::debug!("Speech-dispatcher has successfully been stopped.");
    } else {
        tracing::debug!("Speech-dispatched has not been stopped; you may see problems when attempting to use it again.");
    }
    tracing::debug!("Goodbye, Odilia!");
    Ok(())
}
