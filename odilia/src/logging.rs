//!Logging with the [`tracing`] crate.
//!
//! Not much here yet, but this will get more complex if we decide to add other layers for error
//! reporting, tokio-console, etc.

use std::env;

use eyre::Context;
use odilia_common::settings::{log::LoggingKind, ApplicationConfig};
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::*, EnvFilter};
use tracing_tree::HierarchicalLayer;

/// Initialise the logging stack
/// this requires an application configuration structure, so configuration must be initialized before logging is
pub fn init(config: &ApplicationConfig) -> eyre::Result<()> {
	let env_filter = match env::var("APP_LOG").or_else(|_| env::var("RUST_LOG")) {
		Ok(s) => EnvFilter::from(s),
		_ => EnvFilter::from(&config.log.level),
	};
	//this requires boxing because the types returned by this match block would be incompatible otherwise, since we return different layers depending on what we get from the configuration. It is possible to do it otherwise, hopefully, but for now this and a forced dereference at the end would do
	let output_layer = match &config.log.logger {
		LoggingKind::File(path) => {
			let file = std::fs::File::create(path).with_context(|| {
				format!("creating log file '{}'", path.display())
			})?;
			let fmt =
				tracing_subscriber::fmt::layer().with_ansi(false).with_writer(file);
			fmt.boxed()
		}
		LoggingKind::Tty => tracing_subscriber::fmt::layer()
			.with_ansi(true)
			.with_target(true)
			.boxed(),
		LoggingKind::Syslog => tracing_journald::layer()?.boxed(),
	};
	let subscriber = tracing_subscriber::Registry::default()
		.with(env_filter)
		.with(output_layer)
		.with(ErrorLayer::default())
		.with(HierarchicalLayer::new(4)
			.with_bracketed_fields(true)
			.with_targets(true)
			.with_deferred_spans(true)
			.with_span_retrace(true)
			.with_indent_lines(true));
	tracing::subscriber::set_global_default(subscriber)
		.wrap_err("unable to init default logging layer")?;
	LogTracer::init().wrap_err("unable to init tracing log layer")?;
	Ok(())
}
