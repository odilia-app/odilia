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
    events::process(&state).await;
    Ok(())
}
