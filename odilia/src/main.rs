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
use odilia_input::{
    keybinds::{add_keybind, update_sr_mode},
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::init();
    let (mode_change_tx, mut mode_change_rx) = channel(8); // should maybe be 1? I don't know how it works
    let _args = args::parse();
    let init = state::init_state().await;
    if !init {
        eprintln!("Unable to initialize state. Fatal error.");
        exit(1);
    }
    let atspi_event_future = events::process();
    //let odilia_event_future = events::sr_event(&mut screen_reader_event_stream, mode_change_tx);
    let update_mode_future = update_sr_mode(&mut mode_change_rx);
    let _ = tokio::join!(atspi_event_future,  update_mode_future);
    // Create and run tasks
    let (mode_change_tx, mode_change_rx) = channel(8); // should maybe be 1? I don't know how it works
    let screen_reader_event_stream = create_keybind_channel();

    let atspi_event_future = tokio::spawn(events::process()).map(|r| r.wrap_err("Could not process at-spi events"));
    //let odilia_event_future = events::sr_event(screen_reader_event_stream, mode_change_tx).map(|r| r.wrap_err("Could not process Odilia events"));
    let update_mode_future = tokio::spawn(update_sr_mode(mode_change_rx)).map(|r| r.wrap_err("Could not update mode"));
    tokio::try_join!(atspi_event_future, odilia_event_future, update_mode_future)?;
    Ok(())
}
