//!Logging with the [`tracing`] crate.
//!
//! Not much here yet, but this will get more complex if we decide to add other layers for error
//! reporting, tokio-console, etc.

/// Initialise the logging stack. Right now this just calls [`tracing_subscriber::fmt::init`].
pub fn init() {
    tracing_subscriber::fmt::init();
}
