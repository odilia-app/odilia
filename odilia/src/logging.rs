//!Logging with the [`tracing`] crate.
//!
//! Not much here yet, but this will get more complex if we decide to add other layers for error
//! reporting, tokio-console, etc.

use std::env;

use odilia_common::settings::ApplicationConfig;
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::*, EnvFilter};
use tracing_tree::HierarchicalLayer;

/// Initialise the logging stack
/// this requires an application configuration structure, so configuration must be initialized before logging is
pub fn init(config:&ApplicationConfig) {
	let env_filter = match env::var("ODILIA_LOG").or_else(|_| env::var("RUST_LOG")) {
		Ok(s) => EnvFilter::from(s),
		Err(env::VarError::NotPresent) => EnvFilter::from(&config.log.level),
		Err(e) => {
			eprintln!("Warning: Failed to read log filter from ODILIA_LOG or RUST_LOG: {e}");
			EnvFilter::from(&config.log.level)
		}
	};
	let subscriber = tracing_subscriber::Registry::default()
		.with(env_filter)
		.with(ErrorLayer::default())
		.with(HierarchicalLayer::new(4)
			.with_ansi(true)
			.with_bracketed_fields(true)
			.with_targets(true)
			.with_deferred_spans(true)
			.with_span_retrace(true)
			.with_indent_lines(true));
	if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
		eprintln!("Warning: Failed to set log handler: {e}");
	}
	if let Err(e) = LogTracer::init() {
		tracing::warn!(error = %e, "Failed to install log facade");
	}
}
