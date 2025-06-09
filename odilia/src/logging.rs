//!Logging with the [`tracing`] crate.
//!
//! Not much here yet, but this will get more complex if we decide to add other layers for error
//! reporting, tokio-console, etc.

use std::io;

use odilia_common::{
	errors::OdiliaError,
	settings::{log::LoggingKind, ApplicationConfig},
};
use tracing_subscriber::{
	filter::LevelFilter,
	fmt::{time::Uptime, Layer},
	prelude::*,
};

/// Initialise the logging stack
/// this requires an application configuration structure, so configuration must be initialized before logging is
pub fn init(config: &ApplicationConfig) -> Result<(), OdiliaError> {
	let tree = Layer::new()
		.with_target(true)
		.with_level(true)
		.with_line_number(true)
		.with_ansi(false)
		.with_timer(Uptime::default());
	//this requires boxing because the types returned by this match block would be incompatible otherwise, since we return different layers, or modifications to a layer depending on what we get from the configuration. It is possible to do it otherwise, hopefully, but for now this  would do
	let final_layer = match &config.log.logger {
		LoggingKind::File(path) => {
			let file = std::fs::File::create(path)?;
			tree.with_writer(file).boxed()
		}
		LoggingKind::Tty => tree.with_writer(io::stdout).with_ansi(true).boxed(),
		LoggingKind::Syslog => tracing_journald::Layer::new()?
			.with_syslog_identifier("odilia".to_owned())
			.boxed(),
	};
	let level_filter = LevelFilter::from_level(config.log.level.into());
	tracing_subscriber::Registry::default()
		.with(level_filter)
		.with(final_layer)
		.init();
	Ok(())
}
