mod args;
mod events;
mod logging;
mod state;
use tokio::sync::mpsc::channel;
use std::{
  process::exit,
  collections::HashMap,
};
use odilia_common::{
    events::{
        ScreenReaderEvent,
        Direction,
    },
    modes::{
        ScreenReaderMode,
    },
    input::{
      KeyBinding,
      Modifiers,
      Key
    },
};
use odilia_input::{
    events::create_keybind_channel,
    keybinds::{
      add_keybind,
      update_sr_mode,
    },
};
use atspi::accessible::Role;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let mut s_nav = HashMap::new();
    s_nav.insert(Some(Key::Other('h')), Role::Heading);
    s_nav.insert(Some(Key::Other('b')), Role::PushButton);
    s_nav.insert(Some(Key::Other('k')), Role::Link);
    s_nav.insert(Some(Key::Other('l')), Role::List);
    s_nav.insert(Some(Key::Other('i')), Role::ListItem);
    let ctrl = KeyBinding {
        key: None,
        mods: Modifiers::CONTROL,
        repeat: 1,
        consume: false,
        mode: None,
        notify: true
    };
    let noop_caps = KeyBinding {
        key: None,
        mods: Modifiers::ODILIA,
        repeat: 1,
        consume: true,
        mode: None,
        notify: false
    };
    let browse_mode = KeyBinding {
        key: Some(Key::Other('b')),
        mods: Modifiers::ODILIA,
        repeat: 1,
        consume: true,
        mode: None,
        notify: true
    };
    logging::init();
    let (mode_change_tx,mut mode_change_rx) = channel(8); // should maybe be 1? I don't know how it works
    let mut screen_reader_event_stream = create_keybind_channel();
    for (key,role) in s_nav.into_iter() {
      let kb = KeyBinding {
        key: key,
        mods: Modifiers::NONE,
        repeat: 1,
        consume: true,
        mode: Some(ScreenReaderMode{ name: "BrowseMode".to_string()}),
        notify: true
      };
      let bkb = KeyBinding {
        key: key,
        mods: Modifiers::SHIFT,
        repeat: 1,
        consume: true,
        mode: Some(ScreenReaderMode{ name: "BrowseMode".to_string()}),
        notify: true
      };
      add_keybind(kb, ScreenReaderEvent::StructuralNavigation(Direction::Forward, role)).await;
      add_keybind(bkb, ScreenReaderEvent::StructuralNavigation(Direction::Backward, role)).await;
    }
    add_keybind(ctrl, ScreenReaderEvent::StopSpeech).await;
    add_keybind(
        browse_mode,
        ScreenReaderEvent::ChangeMode(
            ScreenReaderMode {name: "BrowseMode".to_string()}
    )).await;
    add_keybind(
        noop_caps,
        ScreenReaderEvent::Noop
    ).await;
    let _args = args::parse();
    let init = state::init_state().await;
    if !init {
      eprintln!("Unable to initialize state. Fatal error.");
      exit(1);
    } 
    
    state::register_event("Object:StateChanged:Focused").await?;
    state::register_event("Object:TextCaretMoved").await?;
    let atspi_event_future = events::process();
    let odilia_event_future = events::sr_event(&mut screen_reader_event_stream, mode_change_tx);
    let update_mode_future = update_sr_mode(&mut mode_change_rx);
    let _ = tokio::join!(atspi_event_future, odilia_event_future, update_mode_future);
    Ok(())
}
