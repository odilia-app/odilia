//!Logging with the [`tracing`] crate.
//!
//! Not much here yet, but this will get more complex if we decide to add other layers for error
//! reporting, tokio-console, etc.

use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, prelude::*};

/// Initialise the logging stack. Right now this just calls [`tracing_subscriber::fmt::init`].
pub fn init() {
    let subscriber = tracing_subscriber::Registry::default()
        .with(fmt::layer())
        .with(ErrorLayer::default());
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Warning: Failed to set log handler: {}", e);
    }
    if let Err(e) = color_eyre::install() {
        tracing::warn!(error = %e, "Failed to install error / panic handler");
    }
}
