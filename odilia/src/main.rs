mod args;
mod events;
mod logging;
mod state;
use atspi::accessible::Role;
use odilia_common::{
    events::{Direction, ScreenReaderEvent},
    input::{Key, KeyBinding, Modifiers},
    modes::ScreenReaderMode,
};
use odilia_input::{
    keybinds::{add_keybind, update_sr_mode},
};
use std::{collections::HashMap, process::exit};
use tokio::sync::mpsc::channel;

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

    state::register_event("Object:StateChanged:Focused").await?;
    state::register_event("Object:TextCaretMoved").await?;
    state::register_event("Document:LoadComplete").await?;
    let atspi_event_future = events::process();
    //let odilia_event_future = events::sr_event(&mut screen_reader_event_stream, mode_change_tx);
    let update_mode_future = update_sr_mode(&mut mode_change_rx);
    let _ = tokio::join!(atspi_event_future,  update_mode_future);
    Ok(())
}
