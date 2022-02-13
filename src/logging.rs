//! Not much here yet, but this will get more complex if we decide to add other layers for error
//! reporting, tokio-console, etc.

pub fn init() {
    tracing_subscriber::fmt::init();
}
