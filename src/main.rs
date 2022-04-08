mod args;
mod logging;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    logging::init();
    let _args = args::parse();
    tracing::info!("Hello, world!");
    Ok(())
}
