mod args;
mod events;
mod logging;
mod state;
use state::ScreenReaderState;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::init();
    let _args = args::parse();
    let state = ScreenReaderState::new().await?;
    state.register_event("Object:StateChanged:Focused").await?;
    state.register_event("Object:TextCaretMoved").await?;
    let incomming_future = events::process(&state);
    tokio::join!(incomming_future);
    Ok(())
}
