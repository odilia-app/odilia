mod args;
mod events;
mod logging;
mod state;
mod structural;
use state::ScreenReaderState;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::init();
    let _args = args::parse();
    let mut state= ScreenReaderState::new().await?;
    state.register_event("Object:StateChanged:Focused").await?;
    state.register_event("Object:TextCaretMoved").await?;
    events::process(&mut state).await;
    Ok(())
}
