mod args;
mod logging;
mod state;
use state::ScreenReaderState;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::init();
    let _args = args::parse();
    let _state = ScreenReaderState::new().await?;
    tracing::info!("Hello, world!");
    Ok(())
}
