use serde::{Deserialize, Serialize};
use std::path::PathBuf;

///structure used for all the configurable options related to logging
#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct LogSettings {
	///the logging level this session should output at
	/// see the tracing documentation for more information, in the log filters section
	/// typical values here include info, warn, debug and trace
	/// however, one can also include specific modules for which logging should be shown at a different warning level
	pub level: String,
	///the place where odilia should output its logs
	/// the values possible include tty, file and syslog
	pub logger: LoggingKind,
}
impl Default for LogSettings {
	fn default() -> Self {
		let xdg_dirs = xdg::BaseDirectories::with_prefix("odilia").expect(
			"unable to find the odilia config directory according to the xdg dirs specification",
		);
		let log_path = xdg_dirs
			.place_state_file("odilia.log")
			.expect("unable to place log file");

		Self { level: "info".to_owned(), logger: LoggingKind::File(log_path) }
	}
}

///the place where odilia should output its logs
#[derive(Serialize, Deserialize, Debug)]
pub enum LoggingKind {
	///a file where the log messages should be written
	/// the path can be both absolute and relative to the current working directory
	/// warning: the path must be accessible permission wise from the user where odilia was launched
	File(PathBuf),
	///logs are being sent to the terminal directly
	Tty,
	///the logs are sent to systemd-journald, as long as the target architecture supports it
	/// if that's not the case, this option does nothing
	Syslog,
}
