//! Input handlers for the Odilia screen reader.

use input_server_keyboard::*;

//#![deny(clippy::all, missing_docs)]

mod proxy;

#[cfg(test)]
pub(crate) trait EventFromEventType {
	fn from_event_type(event_type: EventType) -> Event {
		Event { event_type, time: std::time::SystemTime::now(), name: None }
	}
}
#[cfg(test)]
impl EventFromEventType for Event {}

use nix::unistd::Uid;
use odilia_common::{
	events::ScreenReaderEvent as OdiliaEvent,
	events::{ChangeMode, StopSpeech},
	modes::ScreenReaderMode as Mode,
};
use rdev::{grab, Event, EventType, Key};

use std::cmp::Ordering;
use std::env;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;

const ACTIVATION_KEY: Key = Key::CapsLock;


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

fn handle_events_to_socket(rx: Receiver<OdiliaEvent>) {
	let (_pid_path, sock_path) = get_file_paths();
	println!("SOCK PATH: {sock_path:?}");
	let Ok(mut stream) = UnixStream::connect(&sock_path) else {
		panic!("Unable to connect to stream {:?}", sock_path);
	};
	for event in rx.iter() {
		let val = serde_json::to_string(&event)
			.expect("Should be able to serialize any event!");
		stream.write_all(val.as_bytes()).expect("Able to write to stream!");
	}
}

fn main() {
	// syncronous, bounded channel
	// NOTE: this will _block the input thread_ if events are not removed from it often.
	// This _should_ never be a problem, because two threads are running, but you never know.
	let (ev_tx, ev_rx) = sync_channel::<OdiliaEvent>(255);
	let combos = ComboSet::from_iter(
		vec![
			(vec![Key::KeyA].try_into().unwrap(), ChangeMode(Mode::Browse).into()),
			(vec![Key::KeyF].try_into().unwrap(), ChangeMode(Mode::Focus).into()),
			// use Odilia + G to mean "stop speech"; like Emacs
			// this allows us to vastly simplify the key handling code since we don't have to create a
			// virtual keyboard and send control if the user is _actually_ using control vs. using it to
			// stop speech.
			(vec![Key::KeyG].try_into().unwrap(), StopSpeech.into()),
		]
		.into_iter(),
	);
	let state = State {
		mode: Mode::Focus,
		activation_key_pressed: false,
		// no allocations below 10-key rollover
		pressed: Vec::with_capacity(10),
		combos: [(None, combos)].try_into().unwrap(),
		tx: ev_tx,
	};
	let _ = thread::spawn(move || {
		// This will block.
		if let Err(error) = grab(callback, state) {
			println!("Error: {:?}", error)
		}
	});
	handle_events_to_socket(ev_rx);
}
