#![deny(clippy::all)]

#[cfg(test)]
mod tests;

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

#[derive(Eq, PartialEq, Clone)]
#[repr(transparent)]
pub struct KeySet {
	inner: Vec<Key>,
}

fn val_key(k1: &Key) -> u64 {
	match k1 {
		Key::Alt => 1 << 32,
		Key::AltGr => 2 << 32,
		Key::Backspace => 3 << 32,
		Key::CapsLock => 4 << 32,
		Key::ControlLeft => 5 << 32,
		Key::ControlRight => 6 << 32,
		Key::Delete => 7 << 32,
		Key::DownArrow => 8 << 32,
		Key::End => 9 << 32,
		Key::Escape => 10 << 32,
		Key::F1 => 11 << 32,
		Key::F10 => 12 << 32,
		Key::F11 => 13 << 32,
		Key::F12 => 14 << 32,
		Key::F2 => 15 << 32,
		Key::F3 => 16 << 32,
		Key::F4 => 17 << 32,
		Key::F5 => 18 << 32,
		Key::F6 => 19 << 32,
		Key::F7 => 20 << 32,
		Key::F8 => 21 << 32,
		Key::F9 => 22 << 32,
		Key::Home => 23 << 32,
		Key::LeftArrow => 24 << 32,
		Key::MetaLeft => 25 << 32,
		Key::MetaRight => 26 << 32,
		Key::PageDown => 27 << 32,
		Key::PageUp => 28 << 32,
		Key::Return => 29 << 32,
		Key::RightArrow => 30 << 32,
		Key::ShiftLeft => 31 << 32,
		Key::ShiftRight => 32 << 32,
		Key::Space => 33 << 32,
		Key::Tab => 34 << 32,
		Key::UpArrow => 35 << 32,
		Key::PrintScreen => 36 << 32,
		Key::ScrollLock => 37 << 32,
		Key::Pause => 38 << 32,
		Key::NumLock => 39 << 32,
		Key::BackQuote => 40 << 32,
		Key::Num1 => 41 << 32,
		Key::Num2 => 42 << 32,
		Key::Num3 => 43 << 32,
		Key::Num4 => 44 << 32,
		Key::Num5 => 45 << 32,
		Key::Num6 => 46 << 32,
		Key::Num7 => 47 << 32,
		Key::Num8 => 48 << 32,
		Key::Num9 => 49 << 32,
		Key::Num0 => 50 << 32,
		Key::Minus => 51 << 32,
		Key::Equal => 52 << 32,
		Key::KeyQ => 53 << 32,
		Key::KeyW => 54 << 32,
		Key::KeyE => 55 << 32,
		Key::KeyR => 56 << 32,
		Key::KeyT => 57 << 32,
		Key::KeyY => 58 << 32,
		Key::KeyU => 59 << 32,
		Key::KeyI => 60 << 32,
		Key::KeyO => 61 << 32,
		Key::KeyP => 62 << 32,
		Key::LeftBracket => 63 << 32,
		Key::RightBracket => 64 << 32,
		Key::KeyA => 65 << 32,
		Key::KeyS => 66 << 32,
		Key::KeyD => 67 << 32,
		Key::KeyF => 68 << 32,
		Key::KeyG => 69 << 32,
		Key::KeyH => 70 << 32,
		Key::KeyJ => 71 << 32,
		Key::KeyK => 72 << 32,
		Key::KeyL => 73 << 32,
		Key::SemiColon => 74 << 32,
		Key::Quote => 75 << 32,
		Key::BackSlash => 76 << 32,
		Key::IntlBackslash => 77 << 32,
		Key::KeyZ => 78 << 32,
		Key::KeyX => 79 << 32,
		Key::KeyC => 80 << 32,
		Key::KeyV => 81 << 32,
		Key::KeyB => 82 << 32,
		Key::KeyN => 83 << 32,
		Key::KeyM => 84 << 32,
		Key::Comma => 85 << 32,
		Key::Dot => 86 << 32,
		Key::Slash => 87 << 32,
		Key::Insert => 88 << 32,
		Key::KpReturn => 89 << 32,
		Key::KpMinus => 90 << 32,
		Key::KpPlus => 91 << 32,
		Key::KpMultiply => 92 << 32,
		Key::KpDivide => 93 << 32,
		Key::Kp0 => 94 << 32,
		Key::Kp1 => 95 << 32,
		Key::Kp2 => 96 << 32,
		Key::Kp3 => 97 << 32,
		Key::Kp4 => 98 << 32,
		Key::Kp5 => 99 << 32,
		Key::Kp6 => 100 << 32,
		Key::Kp7 => 101 << 32,
		Key::Kp8 => 102 << 32,
		Key::Kp9 => 103 << 32,
		Key::KpDelete => 104 << 32,
		Key::Function => 105 << 32,
		Key::Unknown(bits_u32) => (*bits_u32).into(),
	}
}

impl PartialOrd for KeySet {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(&other))
	}
}
impl Ord for KeySet {
	fn cmp(&self, other: &Self) -> Ordering {
		// https://doc.rust-lang.org/std/cmp/trait.Ord.html#lexicographical-comparison
		self.inner.iter().map(val_key).cmp(other.inner.iter().map(val_key))
	}
}
impl std::fmt::Debug for KeySet {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.inner.fmt(fmt)
	}
}
impl KeySet {
	fn insert(&mut self, t: Key) -> Result<(), ()> {
		if t == ACTIVATION_KEY || self.inner.contains(&t) {
			Err(())
		} else {
			self.inner.push(t);
			Ok(())
		}
	}
	fn new() -> Self {
		KeySet { inner: Vec::new() }
	}
	fn from_dedup(v: Vec<Key>) -> Self {
		let mut this = Self::new();
		for item in v {
			// ignore when it's broken
			let _ = this.insert(item);
		}
		this
	}
}
impl<const N: usize> TryFrom<[Key; N]> for KeySet {
	type Error = ();
	fn try_from(items: [Key; N]) -> Result<Self, Self::Error> {
		let mut this = KeySet::new();
		for item in items {
			this.insert(item)?;
		}
		Ok(this)
	}
}
impl TryFrom<Vec<Key>> for KeySet {
	type Error = ();
	fn try_from(items: Vec<Key>) -> Result<Self, Self::Error> {
		let mut this = KeySet::new();
		for item in items {
			this.insert(item)?;
		}
		Ok(this)
	}
}
impl PartialEq<Vec<Key>> for KeySet {
	fn eq(&self, other: &Vec<Key>) -> bool {
		&self.inner == other
	}
}

impl PartialEq<Vec<Key>> for &KeySet {
	fn eq(&self, other: &Vec<Key>) -> bool {
		&self.inner == other
	}
}
impl IntoIterator for KeySet {
	type IntoIter = <Vec<Key> as IntoIterator>::IntoIter;
	type Item = Key;
	fn into_iter(self) -> Self::IntoIter {
		self.inner.into_iter()
	}
}

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

#[derive(Debug, PartialEq, Eq)]
pub enum ComboError {
	SamePrefix { original: KeySet, new: KeySet },
	Identical(KeySet),
}

#[derive(Clone, Eq, PartialEq)]
pub struct ComboSet {
	inner: Vec<(KeySet, OdiliaEvent)>,
}
impl std::fmt::Debug for ComboSet {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.inner.fmt(fmt)
	}
}
impl ComboSet {
	fn keys<'a>(&'a self) -> impl Iterator<Item = &'a KeySet> {
		self.inner.iter().map(|x| &x.0)
	}
	fn insert(&mut self, keys: KeySet, ev: OdiliaEvent) -> Result<(), ComboError> {
		for keyset in self.keys() {
			if *keyset == keys {
				return Err(ComboError::Identical(keys));
			}
			if keyset.inner.starts_with(&keys.inner) {
				return Err(ComboError::SamePrefix {
					original: keyset.clone(),
					new: keys,
				});
			}
			if keys.inner.starts_with(&keyset.inner) {
				return Err(ComboError::SamePrefix {
					original: keyset.clone(),
					new: keys,
				});
			}
		}
		self.inner.push((keys, ev));
		Ok(())
	}
	fn new() -> Self {
		Self { inner: Vec::new() }
	}
	fn from_iter<I>(iter: I) -> Self
	where
		I: Iterator<Item = (KeySet, OdiliaEvent)>,
	{
		let mut this = Self::new();
		for item in iter {
			let _ = this.insert(item.0, item.1);
		}
		this
	}
}
impl IntoIterator for ComboSet {
	type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;
	type Item = (KeySet, OdiliaEvent);
	fn into_iter(self) -> Self::IntoIter {
		self.inner.into_iter()
	}
}

impl<'a> IntoIterator for &'a ComboSet {
	type IntoIter = std::slice::Iter<'a, (KeySet, OdiliaEvent)>;
	type Item = &'a (KeySet, OdiliaEvent);
	fn into_iter(self) -> Self::IntoIter {
		self.inner.iter()
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum SetError {
	IdenticalCombo {
		mode: Option<Mode>,
		set: KeySet,
	},
	SamePrefixCombo {
		original: (Option<Mode>, KeySet),
		attempted: (Option<Mode>, KeySet),
	},
	/// Returned when attempting to add a keybinding with an empty keyset.
	UnpressableKey,
	/// Returned when attempting to add a keybinding with a mode which can not be reached from the
	/// existing set of keybindings.
	UnreachableMode(Mode),
}

#[derive(Clone, PartialEq, Eq)]
pub struct ComboSets {
	inner: Vec<(Option<Mode>, ComboSet)>,
}
impl std::fmt::Debug for ComboSets {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.inner.fmt(fmt)
	}
}
impl ComboSets {
	fn insert(&mut self, mode: Option<Mode>, cs: ComboSet) -> Result<(), SetError> {
		if mode.is_some() && !self.inner.iter().map(|x| x.0).any(|m| m == mode) {
			return Err(SetError::UnreachableMode(mode.unwrap()));
		}
		if cs.inner
			.iter()
			.map(|set| set.0.inner.iter())
			.any(|combo| combo.count() == 0)
		{
			return Err(SetError::UnpressableKey);
		}
		for (combo_mode, combo_sets) in self.inner.iter() {
			if mode.is_none() || *combo_mode == mode || combo_mode.is_none() {
				for combo1 in combo_sets {
					for combo2 in &cs {
						if combo1.0 == combo2.0 {
							return Err(SetError::IdenticalCombo {
								mode,
								set: combo2.0.clone(),
							});
						} else if combo1
							.0
							.inner
							.starts_with(&combo2.0.inner)
						{
							return Err(SetError::SamePrefixCombo {
								original: (
									*combo_mode,
									combo1.0.clone(),
								),
								attempted: (mode, combo2.0.clone()),
							});
						} else if combo2
							.0
							.inner
							.starts_with(&combo1.0.inner)
						{
							return Err(SetError::SamePrefixCombo {
								original: (
									*combo_mode,
									combo1.0.clone(),
								),
								attempted: (mode, combo2.0.clone()),
							});
						}
					}
				}
			} else {
			}
		}
		self.inner.push((mode, cs));
		Ok(())
	}
	fn new() -> Self {
		Self { inner: Vec::new() }
	}
	fn from_iter<I>(iter: I) -> Self
	where
		I: Iterator<Item = (Option<Mode>, ComboSet)>,
	{
		let mut this = Self::new();
		for item in iter {
			let _ = this.insert(item.0, item.1);
		}
		this
	}
}
impl<const N: usize> TryFrom<[(Option<Mode>, ComboSet); N]> for ComboSets {
	type Error = SetError;
	fn try_from(items: [(Option<Mode>, ComboSet); N]) -> Result<Self, Self::Error> {
		let mut this = ComboSets::new();
		for item in items {
			this.insert(item.0, item.1)?;
		}
		Ok(this)
	}
}
impl IntoIterator for ComboSets {
	type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;
	type Item = (Option<Mode>, ComboSet);
	fn into_iter(self) -> Self::IntoIter {
		self.inner.into_iter()
	}
}

impl<'a> IntoIterator for &'a ComboSets {
	type IntoIter = std::slice::Iter<'a, (Option<Mode>, ComboSet)>;
	type Item = &'a (Option<Mode>, ComboSet);
	fn into_iter(self) -> Self::IntoIter {
		self.inner.iter()
	}
}

#[derive(Debug)]
pub struct State {
	pub(crate) activation_key_pressed: bool,
	pub(crate) mode: Mode,
	pub(crate) pressed: Vec<Key>,
	pub(crate) combos: ComboSets,
	pub(crate) tx: SyncSender<OdiliaEvent>,
}
impl State {
	#[cfg(test)]
	/// For testing purposes only: create an "unbounded" (100,000-sized) buffer for accepting the
	/// OdiliaEvents that may be triggered.
	fn new_unbounded() -> (Self, Receiver<OdiliaEvent>) {
		let (tx, rx) = sync_channel(100_000);
		(
			Self {
				activation_key_pressed: false,
				mode: Mode::Focus,
				pressed: Vec::new(),
				combos: ComboSets::new(),
				tx,
			},
			rx,
		)
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
		pressed: Vec::new(),
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

pub(crate) fn callback(event: Event, state: &mut State) -> Option<Event> {
	println!("My callback {:?}", event);
	match (event.event_type, state.activation_key_pressed) {
		// if capslock is pressed while activation is disabled
		(EventType::KeyPress(ACTIVATION_KEY), false) => {
			// enable it
			state.activation_key_pressed = true;
			println!("Cancelling CL!");
			// swallow the event
			None
		}
		// if capslock is released while activation is disabled (happens if capslock was pressed before
		// start, but released after the daemon began intercepting keys)
		(EventType::KeyRelease(ACTIVATION_KEY), false) => {
			// passthrough the event; if you don't, the application the user was focused on will act like
			// capslock is perpetually held
			Some(event)
		}
		// if capslock is pressed while activation is enabled (usually the result of holding down
		// the key)
		(EventType::KeyPress(ACTIVATION_KEY), true) => {
			// swallow the event
			None
		}
		// if capslock is released while activate is enabled
		(EventType::KeyRelease(ACTIVATION_KEY), true) => {
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
			for (mode, combos) in &state.combos {
				if *mode != Some(state.mode) && mode.is_some() {
					continue;
				}
				for combo in combos {
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
