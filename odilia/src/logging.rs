//!Logging with the [`tracing`] crate.
//!
//! Not much here yet, but this will get more complex if we decide to add other layers for error
//! reporting, tokio-console, etc.

use std::io;

use odilia_common::{
	errors::OdiliaError,
	settings::{log::LoggingKind, ApplicationConfig},
};
use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;
use tracing_tree::{time::Uptime, HierarchicalLayer};

/// Initialise the logging stack
/// this requires an application configuration structure, so configuration must be initialized before logging is
pub fn init(config: &ApplicationConfig) -> Result<(), OdiliaError> {
	let tree = HierarchicalLayer::new(4)
		.with_bracketed_fields(true)
		.with_targets(true)
		.with_deferred_spans(true)
		.with_span_retrace(true)
		.with_indent_lines(true)
		.with_ansi(false)
		.with_wraparound(4)
		.with_timer(Uptime::default());
	//this requires boxing because the types returned by this match block would be incompatible otherwise, since we return different layers, or modifications to a layer depending on what we get from the configuration. It is possible to do it otherwise, hopefully, but for now this  would do
	let final_layer = match &config.log.logger {
		LoggingKind::File(path) => {
			tracing::info!("creating log file '{}'", path.display());
			let file = std::fs::File::create(path)?;
			tree.with_writer(file).boxed()
		}
		LoggingKind::Tty => tree.with_writer(io::stdout).with_ansi(true).boxed(),
		LoggingKind::Syslog => tracing_journald::Layer::new()?
			.with_syslog_identifier("odilia".to_owned())
			.boxed(),
	};
	let trace_sub = { tracing_subscriber::Registry::default() };
	trace_sub.with(ErrorLayer::default()).with(final_layer).init();
	Ok(())
}
