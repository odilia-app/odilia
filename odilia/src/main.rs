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
    events::create_keybind_channel,
    keybinds::{add_keybind, update_sr_mode},
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::init();
    let _args = args::parse();

    // Initialize state
    state::init_state().await?;

    // Add directional structural nav keys
    const S_NAV_BINDINGS: &[(Key, Role)] = &[
        (Key::Other('h'), Role::Heading),
        (Key::Other('b'), Role::PushButton),
        (Key::Other('k'), Role::Link),
        (Key::Other('l'), Role::List),
        (Key::Other('i'), Role::ListItem),
    ];

    for (key, role) in S_NAV_BINDINGS.iter().copied() {
        let forward_kb = KeyBinding::new(Some(key)).mode(Some(ScreenReaderMode {
            name: "BrowseMode".to_string(),
        }));
        let backward_kb = forward_kb.clone().mods(Modifiers::SHIFT);

        add_keybind(
            forward_kb,
            ScreenReaderEvent::StructuralNavigation(Direction::Forward, role),
        )
        .await;
        add_keybind(
            backward_kb,
            ScreenReaderEvent::StructuralNavigation(Direction::Backward, role),
        )
        .await;
    }

    // Misc keybindings
    let noop_caps = KeyBinding::default().mods(Modifiers::ODILIA).notify(false);
    add_keybind(noop_caps, ScreenReaderEvent::Noop).await;
    let ctrl = KeyBinding::default()
        .mods(Modifiers::CONTROL)
        .consume(false);
    add_keybind(ctrl, ScreenReaderEvent::StopSpeech).await;
    let browse_mode = KeyBinding::new(Some(Key::Other('b'))).mods(Modifiers::ODILIA);
    add_keybind(
        browse_mode,
        ScreenReaderEvent::ChangeMode(ScreenReaderMode {
            name: "BrowseMode".to_string(),
        }),
    )
    .await;

    // Register events
    state::register_event("Object:StateChanged:Focused").await?;
    state::register_event("Object:TextCaretMoved").await?;
    state::register_event("Document:LoadComplete").await?;

    // Create and run tasks
    let (mode_change_tx, mode_change_rx) = channel(8); // should maybe be 1? I don't know how it works
    let screen_reader_event_stream = create_keybind_channel();

    let atspi_event_future = tokio::spawn(events::process()).map(|r| r.wrap_err("Could not process at-spi events"));
    let odilia_event_future = events::sr_event(screen_reader_event_stream, mode_change_tx).map(|r| r.wrap_err("Could not process Odilia events"));
    let update_mode_future = tokio::spawn(update_sr_mode(mode_change_rx)).map(|r| r.wrap_err("Could not update mode"));
    tokio::try_join!(atspi_event_future, odilia_event_future, update_mode_future)?;
    Ok(())
}
