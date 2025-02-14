use nix::unistd::Uid;
use odilia_common::{
	events::ScreenReaderEvent as OdiliaEvent,
	events::{ChangeMode, Disable, Enable, StopSpeech, StructuralNavigation},
	modes::ScreenReaderMode as Mode,
};
use rdev::{grab, Event, EventType, Key};

use std::env;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;

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

#[derive(Default)]
pub struct ComboSet {
  mode: Option<Mode>,
	combos: Vec<(Vec<Key>, OdiliaEvent)>,
}

pub struct State {
	activation_key_pressed: bool,
  mode: Mode,
	pressed: Vec<Key>,
  combos: Vec<ComboSet>,
	tx: SyncSender<OdiliaEvent>,
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

fn handle_modechange_from_socket(rx: Receiver<Mode>) {
	let (_pid_path, sock_path) = get_file_paths();
	println!("SOCK PATH: {sock_path:?}");
	let Ok(mut stream) = UnixStream::connect(&sock_path) else {
		panic!("Unable to connect to stream {:?}", sock_path);
	};
	for mode in rx.iter() {
		let val = serde_json::to_string(&mode)
			.expect("Should be able to serialize any event!");
    println!("{val:?}");
	}
}

fn main() {
	// syncronous, bounded channel
	// NOTE: this will _block the input thread_ if events are not removed from it often.
	// This _should_ never be a problem, because two threads are running, but you never know.
	let (ev_tx, ev_rx) = sync_channel::<OdiliaEvent>(255);
	let combos = vec![
		(vec![Key::KeyA], ChangeMode(Mode::Browse).into()),
		(vec![Key::KeyF], ChangeMode(Mode::Focus).into()),
		// use Odilia + G to mean "stop speech"; like Emacs
		// this allows us to vastly simplify the key handling code since we don't have to create a
		// virtual keyboard and send control if the user is _actually_ using control vs. using it to
		// stop speech.
		(vec![Key::KeyG], StopSpeech.into()),
	];
	let state = State {
    mode: Mode::Focus,
		activation_key_pressed: false,
		pressed: Vec::new(),
		combos: vec![
        ComboSet { combos, mode: None },
    ],
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

fn callback(event: Event, state: &mut State) -> Option<Event> {
	println!("My callback {:?}", event);
	match (event.event_type, state.activation_key_pressed) {
		// if capslock is pressed while activation is disabled
		(EventType::KeyPress(Key::CapsLock), false) => {
			// enable it
			state.activation_key_pressed = true;
			println!("Cancelling CL!");
			// swallow the event
			None
		}
		// if capslock is released while activation is disabled (should never happen)
		(EventType::KeyRelease(Key::CapsLock), false) => {
			// swallow the event
			None
		}
		// if capslock is pressed while activation is enabled (usually the result of holding down
		// the key)
		(EventType::KeyPress(Key::CapsLock), true) => {
			// swallow the event
			None
		}
		// if capslock is released while activate is enabled
		(EventType::KeyRelease(Key::CapsLock), true) => {
			// disable activate state
			state.activation_key_pressed = false;
			println!("Cancelling CL! Dropping activation feature.");
			// and swallow event
			None
		}
		// if a key press is made while activation is enabled
		(EventType::KeyPress(other), true) => {
			// if the key is already pressed (i.e., it's been held down)
			let None = state.pressed.iter().position(|key| *key == other) else {
				// swallow the event immediately, do not pass go
				return None;
			};
			// otherwise, add it to the list of held keys
			state.pressed.push(other);
			// look in the combos
      for combo_set in &state.combos {
          if combo_set.mode != Some(state.mode) && 
            combo_set.mode.is_some() {
            continue;
          }
          for combo in &combo_set.combos {
            println!("Combo: {combo:?}");
            println!("Pressed {:?}", state.pressed);
            // if a combo matches the held keys (must be in right order)
            if combo.0 == state.pressed {
              // print out the command
              println!("Combo found for {:?}", combo.1);
              state.tx.send(combo.1.clone()).expect(
                "To be able to send the combo over the channel",
              );
            }
          }
      }
			// swallow the event
			None
		}
		// if a key release is made while activation mode is on
		(EventType::KeyRelease(other), true) => {
			// if it's previously been pressed
			if let Some(idx) = state.pressed.iter().position(|key| *key == other) {
				// remove it from the list of held keys
				state.pressed.remove(idx);
				// and swallow the event
				None
				// otherwise, it was a key held from before the activation was enabled
			} else {
				// pass this through to the other layers, as applications need to be notified about
				// letting go of the key
				Some(event)
			}
		}
		// all other cases (having to do with the mouse): pass through
		_ => Some(event),
	}
}
