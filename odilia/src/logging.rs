//!Logging with the [`tracing`] crate.
//!
//! Not much here yet, but this will get more complex if we decide to add other layers for error
//! reporting, tokio-console, etc.

use std::{env, io};

use eyre::Context;
use odilia_common::settings::{log::LoggingKind, ApplicationConfig};
use tracing_error::ErrorLayer;
use tracing_subscriber::{prelude::*, EnvFilter};
use tracing_tree::HierarchicalLayer;

/// Initialise the logging stack
/// this requires an application configuration structure, so configuration must be initialized before logging is
pub fn init(config: &ApplicationConfig) -> eyre::Result<()> {
	let env_filter = match env::var("APP_LOG").or_else(|_| env::var("RUST_LOG")) {
		Ok(s) => EnvFilter::from(s),
		_ => EnvFilter::from(&config.log.level),
	};
	let tree = HierarchicalLayer::new(4)
		.with_bracketed_fields(true)
		.with_targets(true)
		.with_deferred_spans(true)
		.with_span_retrace(true)
		.with_indent_lines(true)
		.with_ansi(false)
		.with_wraparound(4);
	//this requires boxing because the types returned by this match block would be incompatible otherwise, since we return different layers, or modifications to a layer depending on what we get from the configuration. It is possible to do it otherwise, hopefully, but for now this  would do
	let final_layer = match &config.log.logger {
		LoggingKind::File(path) => {
			let file = std::fs::File::create(path).with_context(|| {
				format!("creating log file '{}'", path.display())
			})?;
			tree.with_writer(file).boxed()
		}
		LoggingKind::Tty => tree.with_writer(io::stdout).with_ansi(true).boxed(),
		LoggingKind::Syslog => tracing_journald::Layer::new()?
			.with_syslog_identifier("odilia".to_owned())
			.boxed(),
	};
	tracing_subscriber::Registry::default()
		.with(env_filter)
		.with(ErrorLayer::default())
		.with(final_layer)
		.init();
	Ok(())
}
