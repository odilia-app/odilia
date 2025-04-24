//! `odilia-input-method-keyboard`
//!
//! Control the Odilia screen reader with your keyboard.
//! This crate uses the `evdev` kernel interface to work anywhere with physical keyboard access (virtual keyboards are not supported).
//! To use it, you must be a part of your system's `input`, `evdev`, or `plugdev` group (depending on distributions).
#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::print_stdout,
	clippy::print_stderr,
	missing_docs,
	rustdoc::all,
	clippy::missing_docs_in_private_items
)]

use nix::unistd::Uid;
use odilia_common::{events::ScreenReaderEvent as OdiliaEvent, modes::ScreenReaderMode as Mode};
use odilia_input_server_keyboard::{callback, ComboSets, State};
use rdev::grab;
use std::{
	env,
	io::Write,
	os::unix::net::UnixStream,
	path::PathBuf,
	sync::mpsc::{sync_channel, Receiver},
	thread,
};

/// Finds PID and Socket files and returns their respective [`PathBuf`]s.
/// The first return value is the PID file.
/// The second return value is the Socket file.
fn get_file_paths() -> (PathBuf, PathBuf) {
	match env::var("XDG_RUNTIME_DIR") {
		Ok(val) => {
			tracing::info!(
                "XDG_RUNTIME_DIR Variable is present, using it's value as default file path."
            );

			let pid_file_path = format!("{val}/odilias.pid");
			let sock_file_path = format!("{val}/odilia.sock");

			(pid_file_path.into(), sock_file_path.into())
		}
		Err(e) => {
			tracing::warn!(error=%e, "XDG_RUNTIME_DIR Variable is not set, falling back to hardcoded path");

			let pid_file_path = format!("/run/user/{}/odilias.pid", Uid::current());
			let sock_file_path = format!("/run/user/{}/odilia.sock", Uid::current());

			(pid_file_path.into(), sock_file_path.into())
		}
	}
}

/// Takes a [`Receiver`] and blocks forever waiting on results from it.
/// When it receives an event, it sends it over the unix socket to notify Odilia.
fn handle_events_to_socket(rx: &Receiver<OdiliaEvent>) -> Result<(), std::io::Error> {
	let (_pid_path, sock_path) = get_file_paths();
	tracing::debug!(?sock_path, "This is the socket path we recieved");
	let mut stream = UnixStream::connect(&sock_path)?;
	for event in rx {
		let val = serde_json::to_string(&event)
			.expect("Should be able to serialize any event!");
		stream.write_all(val.as_bytes())?;
	}
	Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// syncronous, bounded channel
	// NOTE: this will _block the input thread_ if events are not removed from it often.
	// This _should_ never be a problem, because two threads are running, but you never know.
	let (ev_tx, ev_rx) = sync_channel::<OdiliaEvent>(255);
	let combos = ComboSets::default();
	let state = State {
		mode: Mode::Focus,
		activation_key_pressed: false,
		// no allocations below 10-key rollover
		pressed: Vec::with_capacity(10),
		combos,
		tx: ev_tx,
	};
	let _ = thread::spawn(move || {
		// This will block.
		if let Err(error) = grab(callback, state) {
			tracing::error!("Error grabbing keyboard: {error:?}");
		}
	});
	handle_events_to_socket(&ev_rx)?;
	Ok(())
}
