use nix::unistd::Uid;
use odilia_common::{
	events::{ChangeMode, ScreenReaderEvent as OdiliaEvent, StopSpeech},
	modes::ScreenReaderMode as Mode,
};
use odilia_input_server_keyboard::*;
use rdev::{grab, Key};
use std::{
	env,
	io::Write,
	os::unix::net::UnixStream,
	path::PathBuf,
	sync::mpsc::{sync_channel, Receiver},
	thread,
};
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
			println!("Error: {:?}", error)
		}
	});
	handle_events_to_socket(ev_rx);
}
