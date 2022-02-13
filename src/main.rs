mod logging;

#[tokio::main]
async fn main() {
    logging::init();
    tracing::info!("Hello, world!");
}
