mod args;
mod events;
mod logging;
mod state;
use state::ScreenReaderState;
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::init();
    let _args = args::parse();
    let state = ScreenReaderState::new().await?;
    state.register_event("Object:StateChanged:Focused").await?;
    state.register_event("Object:TextCaretMoved").await?;
    let event_stream = state.atspi.event_stream();
    let (tx, mut rx) = channel(8); // limit is insanely high, but could be useful in rare
                                      // circumstances
    let incomming_future = events::process(&tx, event_stream);
    let handling_future = events::event_listener(&mut rx, &state);
    tokio::join!(incomming_future, handling_future);
    Ok(())
}
