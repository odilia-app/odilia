#![cfg(feature = "integration_tests")]

use std::{
	process::Command,
	sync::mpsc::{sync_channel, Receiver},
	thread,
	time::Duration,
};

use odilia_common::{
	events::{ScreenReaderEvent as OdiliaEvent, StopSpeech},
	modes::ScreenReaderMode as Mode,
};
use odilia_input_server_keyboard::{callback, ComboSets, State};
use rdev::grab;

/// Arguments to [`ydotool`]:
///
/// 1. "key": press keys by keycode
/// 2. press capslock
/// 3. press g
/// 4. release g
/// 5. release capslock
const SEQUENCE_OF_KEYS: [&str; 5] = ["key", "58:1", "34:1", "34:0", "58:0"];

fn handle_events_to_socket(rx: Receiver<OdiliaEvent>) {
	assert_eq!(
		rx.recv_timeout(Duration::from_secs(1)),
		Ok(StopSpeech.into()),
		"unexpected event recieved from the keyboard state machine, this is a bug (or your setup is misconfigured)"
	);
}

#[test]
fn test_key_capture() {
	let (ev_tx, ev_rx) = sync_channel::<OdiliaEvent>(5);
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
      panic!("Error grabbing keyboard: {error:?}");
			tracing::error!("Error grabbing keyboard: {error:?}");
		}
	});
	thread::sleep(Duration::from_millis(500));
	let _cmd = Command::new("ydotool")
		.args(SEQUENCE_OF_KEYS)
		.output()
		.expect("Unable to find `ydotool`");

	handle_events_to_socket(ev_rx);
}
