mod args;
mod cache;
mod events;
mod logging;
mod state;

use eyre::WrapErr;
use futures::future::FutureExt;
use tokio::sync::mpsc::channel;

use atspi::accessible::Role;
use odilia_common::{
    events::{Direction, ScreenReaderEvent},
    input::{Key, KeyBinding, Modifiers},
    modes::ScreenReaderMode,
};
use odilia_input::sr_event_receiver;

use std::process::exit;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::init();
    let (sr_event_tx, mut sr_event_rx) = channel(8);
    let _args = args::parse();
    let init = state::init_state().await;
    if init.is_err() {
        eprintln!("Unable to initialize state. Fatal error.");
        exit(1);
    }
    let atspi_event_future = tokio::spawn(events::process()).map(|r| r.wrap_err("Could not process at-spi events"));
    let odilia_event_receiver = tokio::spawn(sr_event_receiver(sr_event_tx)).map(|r| r.wrap_err("Could not set up event receiver"));
    let odilia_event_future = tokio::spawn(events::sr_event(sr_event_rx)).map(|r| r.wrap_err("Could not process Odilia events"));
    tokio::try_join!(atspi_event_future, odilia_event_receiver, odilia_event_future)?;
    Ok(())
}
