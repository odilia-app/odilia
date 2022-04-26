use eyre::WrapErr;

pub struct ScreenReaderState {
    pub atspi: atspi::Connection,
}

impl ScreenReaderState {
    pub async fn new() -> eyre::Result<Self> {
        let atspi = atspi::Connection::open()
            .await
            .context("Could not connect to at-spi bus")?;
        Ok(Self { atspi })
    }
}
