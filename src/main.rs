mod args;
mod events;
mod logging;
mod state;
use state::ScreenReaderState;
use odilia_common::input::{
    KeyBinding,
    Modifiers,
};
use odilia_common::{
    events::{
        ScreenReaderEvent,
    },
    modes::{
        ScreenReaderMode,
    },
    input::{
        Key
    },
};
use odilia_input::{
    events::create_keybind_channel,
    keybinds::add_keybind,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
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
    let mut screen_reader_event_stream = create_keybind_channel();
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
    let state = ScreenReaderState::new().await?;
    state.register_event("Object:StateChanged:Focused").await?;
    state.register_event("Object:TextCaretMoved").await?;
    let atspi_event_future = events::process(&state);
    let odilia_event_future = events::sr_event(&mut screen_reader_event_stream);
    tokio::join!(atspi_event_future, odilia_event_future);
    Ok(())
}
