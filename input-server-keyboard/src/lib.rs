//! `odilia-input-server-keyboard`
//!
//! Library to handle state mechanics for keyboard controll of the Odilia screen reader.
//! Uses the `evdev` kernel interface to interrupt keys as necessary;
//! this allows Odilia to work anywhere: X11, Wayland, and TTY.

#![deny(
	clippy::all,
	clippy::pedantic,
	missing_docs,
	clippy::perf,
	clippy::complexity,
	clippy::style,
	rustdoc::all,
	clippy::print_stdout,
	clippy::print_stderr
)]

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "proptest"))]
mod proptests;

use odilia_common::{
	atspi::Role,
	events::{
		ChangeMode, Direction, Quit, ScreenReaderEvent as OdiliaEvent, StopSpeech,
		StructuralNavigation,
	},
	modes::ScreenReaderMode as Mode,
};
use rdev::{Event, EventType, Key};

use std::cmp::Ordering;
use std::sync::mpsc::SyncSender;

/// The fixed activation key for all keybindings.
pub const ACTIVATION_KEY: Key = Key::CapsLock;

/// A set of keys to be used as the combination for a binding.
#[derive(Eq, PartialEq, Clone, Default)]
#[repr(transparent)]
pub struct KeySet {
	inner: Vec<Key>,
}

#[allow(clippy::too_many_lines, clippy::trivially_copy_pass_by_ref)]
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

/// An error in creating (or modifying) a [`KeySet`].
#[derive(Debug, PartialEq, Eq)]
pub enum KeySetError {
	/// Attempted to add the [`ACTIVATION_KEY`].
	ActivationKey,
	/// Attempted to add a key twice.
	AlreadyContains(rdev::Key),
}

impl PartialOrd for KeySet {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
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
	/// Add a new key to the set.
	///
	/// # Errors
	///
	/// Returns an `Err` variant if either:
	///
	/// - Equal to [`ACTIVATION_KEY`], or
	/// - Already contained in the set of keys.
	///
	/// ```
	/// use rdev::Key;
	/// use odilia_input_server_keyboard::KeySet;
	/// let mut ks = KeySet::new();
	/// assert!(ks.insert(Key::ShiftLeft).is_ok());
	/// assert!(ks.insert(Key::KeyA).is_ok());
	/// assert!(ks.insert(Key::KeyL).is_ok());
	/// assert!(ks.insert(Key::KeyA).is_err());
	/// assert!(ks.insert(Key::CapsLock).is_err());
	/// ```
	pub fn insert(&mut self, key: Key) -> Result<(), KeySetError> {
		if key == ACTIVATION_KEY {
			Err(KeySetError::ActivationKey)
		} else if self.inner.contains(&key) {
			Err(KeySetError::AlreadyContains(key))
		} else {
			self.inner.push(key);
			Ok(())
		}
	}
	/// Creates a new, empty `KeySet`.
	#[must_use]
	pub fn new() -> Self {
		KeySet { inner: Vec::new() }
	}
	/// Create a `KeySet` from a list of keys.
	///
	/// # Errors
	///
	/// See [`Self::insert`].
	pub fn try_from_iter<I>(mut iter: I) -> Result<Self, KeySetError>
	where
		I: Iterator<Item = Key>,
	{
		let mut this = Self::new();
		iter.try_for_each(|item| this.insert(item))?;
		Ok(this)
	}
}
impl<const N: usize> TryFrom<[Key; N]> for KeySet {
	type Error = KeySetError;
	fn try_from(items: [Key; N]) -> Result<Self, Self::Error> {
		let mut this = KeySet::new();
		items.into_iter().try_for_each(|item| this.insert(item))?;
		Ok(this)
	}
}
impl TryFrom<Vec<Key>> for KeySet {
	type Error = KeySetError;
	fn try_from(items: Vec<Key>) -> Result<Self, Self::Error> {
		let mut this = KeySet::new();
		items.into_iter().try_for_each(|item| this.insert(item))?;
		Ok(this)
	}
}
impl PartialEq<[Key]> for KeySet {
	fn eq(&self, other: &[Key]) -> bool {
		self.inner == other
	}
}

impl PartialEq<[Key]> for &KeySet {
	fn eq(&self, other: &[Key]) -> bool {
		self.inner == other
	}
}
impl IntoIterator for KeySet {
	type IntoIter = <Vec<Key> as IntoIterator>::IntoIter;
	type Item = Key;
	fn into_iter(self) -> Self::IntoIter {
		self.inner.into_iter()
	}
}

/// An error in creating a set of key combos.
#[derive(Debug, PartialEq, Eq)]
pub enum ComboError {
	/// An existing combo has the same prefix.
	SamePrefix {
		/// Existing combo that has the same prefix.
		original: KeySet,
		/// The combo which was attempted to be added, but failed.
		new: KeySet,
	},
	/// An existing combo has the same set of keys assigned to it.
	Identical(KeySet),
}

/// A set of key combos and their associated action.
#[derive(Clone, Eq, PartialEq, Default)]
#[repr(transparent)]
pub struct ComboSet {
	inner: Vec<(KeySet, OdiliaEvent)>,
}
impl std::fmt::Debug for ComboSet {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.inner.fmt(fmt)
	}
}
impl TryFrom<Vec<(KeySet, OdiliaEvent)>> for ComboSet {
	type Error = ComboError;
	fn try_from(v: Vec<(KeySet, OdiliaEvent)>) -> Result<Self, Self::Error> {
		let mut this = Self::new();
		v.into_iter().try_for_each(|item| this.insert(item.0, item.1))?;
		Ok(this)
	}
}
impl<const N: usize> TryFrom<[(KeySet, OdiliaEvent); N]> for ComboSet {
	type Error = ComboError;
	fn try_from(v: [(KeySet, OdiliaEvent); N]) -> Result<Self, Self::Error> {
		let mut this = ComboSet::new();
		v.into_iter().try_for_each(|item| this.insert(item.0, item.1))?;
		Ok(this)
	}
}

impl ComboSet {
	/// [`Iterator`] through each [`KeySet`] contained in the [`ComboSet`].
	pub fn keys(&self) -> impl Iterator<Item = &'_ KeySet> {
		self.inner.iter().map(|x| &x.0)
	}
	/// Insert a new [`KeySet`], [`OdiliaEvent`] combination.
	///
	/// # Errors
	///
	/// Fails under any of the following conditions:
	///
	/// 1. There is an existing, identical [`KeySet`] already inserted,
	/// 2. There is an existing [`KeySet`] that starts with same sequence,
	/// 3. The attempted [`KeySet`] starts with the same sequence as an existing [`KeySet`]
	///    (reciprocal of 2.)
	/// ```
	/// use rdev::Key;
	/// use odilia_input_server_keyboard::{KeySet, ComboSet};
	/// use odilia_common::{
	///   atspi::Role,
	///   events::{ChangeMode, Direction, ScreenReaderEvent as OdiliaEvent, StopSpeech, StructuralNavigation},
	///   modes::ScreenReaderMode as Mode,
	/// };
	/// // Shift + A
	/// let mut ks1 = KeySet::new();
	/// ks1.insert(Key::ShiftLeft).unwrap();
	/// ks1.insert(Key::KeyA).unwrap();
	/// // Control + A
	/// let mut ks2 = KeySet::new();
	/// ks2.insert(Key::ControlLeft).unwrap();
	/// ks2.insert(Key::KeyA).unwrap();
	/// // Shift + Control + A
	/// let mut ks3 = KeySet::new();
	/// ks3.insert(Key::ShiftLeft).unwrap();
	/// ks3.insert(Key::ControlLeft).unwrap();
	/// ks3.insert(Key::KeyA).unwrap();
	/// // Control + Shift + A
	/// let mut ks4 = KeySet::new();
	/// ks4.insert(Key::ControlLeft).unwrap();
	/// ks4.insert(Key::ShiftLeft).unwrap();
	/// ks4.insert(Key::KeyA).unwrap();
	///
	/// let mut cs1 = ComboSet::new();
	/// assert!(cs1.insert(ks1, StopSpeech.into()).is_ok());
	/// assert!(cs1.insert(ks2, StructuralNavigation(Direction::Forward, Role::Link).into()).is_ok());
	/// assert!(cs1.insert(ks3, ChangeMode(Mode::Focus).into()).is_ok());
	/// assert!(cs1.insert(ks4, ChangeMode(Mode::Browse).into()).is_ok());
	/// ```
	///
	/// This ensures that keys can not overlap and cause unexpected behaviour for the user.
	/// This does add one restriction: you may not have one keybinding run two actions.
	/// While we recognize this limitation, we would like to keep it this way (at least for now) for
	/// stability purposes.
	pub fn insert(&mut self, keys: KeySet, ev: OdiliaEvent) -> Result<(), ComboError> {
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
	/// Create a new, empty [`ComboSet`].
	#[must_use]
	pub fn new() -> Self {
		Self { inner: Vec::new() }
	}
	/// Create a [`ComboSet`] from an iterator.
	/// # Errors
	///
	/// See [`Self::insert`].
	pub fn try_from_iter<I>(mut iter: I) -> Result<Self, ComboError>
	where
		I: Iterator<Item = (KeySet, OdiliaEvent)>,
	{
		let mut this = Self::new();
		iter.try_for_each(|item| this.insert(item.0, item.1))?;
		Ok(this)
	}
}

#[allow(dead_code)]
impl ComboSet {
	fn iter(&self) -> std::slice::Iter<'_, (KeySet, OdiliaEvent)> {
		<&Self as IntoIterator>::into_iter(self)
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

/// An error in adding a new set of keybindings to the [`ComboSets`] list.
#[derive(Debug, PartialEq, Eq)]
pub enum SetError {
	/// An identical combo has already been set.
	/// This happens if either the two combos have the same mode, or the mode of the original combo is `None`
	/// (global).
	IdenticalCombo {
		/// The mode, if specified.
		mode: Option<Mode>,
		/// The keyset
		set: KeySet,
	},
	/// A combo with the same prefix has already been set.
	/// This happens if either the two combos have the same mode, or the mode of the original combo is `None`
	/// (global).
	SamePrefixCombo {
		/// The mode and keyset which contain the same prefix.
		original: (Option<Mode>, KeySet),
		/// The attempted combo to add.
		attempted: (Option<Mode>, KeySet),
	},
	/// Attempted to add a combo with an empty set of keys.
	UnpressableKey,
	/// Attempted to add a keybinding with a mode that is not accessible via pressing other keys.
	/// This can usually be fixed by changing the order in which the keys are added.
	/// Make sure to introduce the keybinding to change to a given mode before the keybindings that
	/// are active in that mode.
	UnreachableMode(Mode),
}

/// A list of modes and their associated key combos.
#[derive(Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ComboSets {
	inner: Vec<(Option<Mode>, ComboSet)>,
}
impl std::fmt::Debug for ComboSets {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.inner.fmt(fmt)
	}
}
impl ComboSets {
	/// Add a new set of combos.
	///
	/// # Errors
	///
	/// Fails under any of the following conditions:
	///
	/// - The new `mode` param is not reachable by current keyboard shortcuts.
	/// - There is an identical (or identically prefixed) combo in an existing [`ComboSet`] which is
	///   globally available (mode: `None`) or in the same mode as the attempted insertion.
	///
	/// ```
	/// use rdev::Key;
	/// use odilia_input_server_keyboard::{KeySet, ComboSet, SetError, ComboSets};
	/// use odilia_common::{
	///   atspi::Role,
	///   events::{ChangeMode, Direction, ScreenReaderEvent as OdiliaEvent, StopSpeech, StructuralNavigation},
	///   modes::ScreenReaderMode as Mode,
	/// };
	/// // Shift + A
	/// let mut ks1 = KeySet::new();
	/// ks1.insert(Key::ShiftLeft).unwrap();
	/// ks1.insert(Key::KeyA).unwrap();
	/// // Control + A
	/// let mut ks2 = KeySet::new();
	/// ks2.insert(Key::ControlLeft).unwrap();
	/// ks2.insert(Key::KeyA).unwrap();
	/// // Shift + Control + A
	/// let mut ks3 = KeySet::new();
	/// ks3.insert(Key::ShiftLeft).unwrap();
	/// ks3.insert(Key::ControlLeft).unwrap();
	/// ks3.insert(Key::KeyA).unwrap();
	/// // Control + Shift + A
	/// let mut ks4 = KeySet::new();
	/// ks4.insert(Key::ControlLeft).unwrap();
	/// ks4.insert(Key::ShiftLeft).unwrap();
	/// ks4.insert(Key::KeyA).unwrap();
	///
	/// let mut cs1 = ComboSet::new();
	/// cs1.insert(ks1, StopSpeech.into());
	/// let mut cs2 = ComboSet::new();
	/// cs2.insert(ks2, StructuralNavigation(Direction::Forward, Role::Link).into());
	/// let mut cs3 = ComboSet::new();
	/// cs3.insert(ks3, ChangeMode(Mode::Focus).into());
	/// cs3.insert(ks4, ChangeMode(Mode::Browse).into());
	///
	/// let mut css = ComboSets::new();
	/// assert!(css.insert(None, cs1).is_ok());
	/// assert_eq!(css.insert(Some(Mode::Focus), cs2.clone()), Err(SetError::UnreachableMode(Mode::Focus)));
	/// assert!(css.insert(None, cs3).is_ok());
	/// assert!(css.insert(Some(Mode::Focus), cs2).is_ok());
	/// ```
	pub fn insert(&mut self, mode: Option<Mode>, cs: ComboSet) -> Result<(), SetError> {
		if let Some(some_mode) = mode {
			if !self.inner
				.iter()
				.flat_map(|x| x.1.inner.iter())
				.filter_map(|ev| match ev.1 {
					OdiliaEvent::ChangeMode(ChangeMode(mode)) => Some(mode),
					_ => None,
				})
				.any(|m| m == some_mode)
			{
				return Err(SetError::UnreachableMode(some_mode));
			}
		}
		if cs.inner
			.iter()
			.map(|set| set.0.inner.iter())
			.any(|combo| combo.count() == 0)
		{
			return Err(SetError::UnpressableKey);
		}
		for (combo_mode, combo_sets) in self.iter() {
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
							.starts_with(&combo2.0.inner) || combo2
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
			}
		}
		self.inner.push((mode, cs));
		Ok(())
	}
	/// Create a new, empty [`ComboSets`].
	#[must_use]
	pub fn new() -> Self {
		Self { inner: Vec::new() }
	}
	/// Attempt to construct from an iterator.
	///
	/// # Errors
	///
	/// See error section for [`Self::insert`].
	pub fn try_from_iter<I>(mut iter: I) -> Result<Self, SetError>
	where
		I: Iterator<Item = (Option<Mode>, ComboSet)>,
	{
		let mut this = Self::new();
		iter.try_for_each(|item| this.insert(item.0, item.1))?;
		Ok(this)
	}
}
impl<const N: usize> TryFrom<[(Option<Mode>, ComboSet); N]> for ComboSets {
	type Error = SetError;
	fn try_from(items: [(Option<Mode>, ComboSet); N]) -> Result<Self, Self::Error> {
		let mut this = ComboSets::new();
		items.into_iter().try_for_each(|item| this.insert(item.0, item.1))?;
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

impl Default for ComboSets {
	fn default() -> Self {
		ComboSets::try_from([
			(
				None,
				ComboSet::try_from([
					(
						[Key::KeyF].try_into().unwrap(),
						ChangeMode(Mode::Focus).into(),
					),
					([Key::KeyG].try_into().unwrap(), StopSpeech.into()),
					(
						[Key::KeyB].try_into().unwrap(),
						ChangeMode(Mode::Browse).into(),
					),
					(
						[Key::ShiftLeft, Key::KeyQ].try_into().unwrap(),
						Quit.into(),
					),
				])
				.unwrap(),
			),
			(
				Some(Mode::Browse),
				ComboSet::try_from([
					(
						[Key::KeyT].try_into().unwrap(),
						StructuralNavigation(
							Direction::Forward,
							Role::Table,
						)
						.into(),
					),
					(
						[Key::ShiftLeft, Key::KeyT].try_into().unwrap(),
						StructuralNavigation(
							Direction::Backward,
							Role::Table,
						)
						.into(),
					),
					(
						[Key::KeyH].try_into().unwrap(),
						StructuralNavigation(
							Direction::Forward,
							Role::Header,
						)
						.into(),
					),
					(
						[Key::ShiftLeft, Key::KeyH].try_into().unwrap(),
						StructuralNavigation(
							Direction::Backward,
							Role::Header,
						)
						.into(),
					),
					(
						[Key::KeyI].try_into().unwrap(),
						StructuralNavigation(
							Direction::Forward,
							Role::Image,
						)
						.into(),
					),
					(
						[Key::ShiftLeft, Key::KeyI].try_into().unwrap(),
						StructuralNavigation(
							Direction::Backward,
							Role::Image,
						)
						.into(),
					),
					(
						[Key::KeyK].try_into().unwrap(),
						StructuralNavigation(
							Direction::Forward,
							Role::Link,
						)
						.into(),
					),
					(
						[Key::ShiftLeft, Key::KeyK].try_into().unwrap(),
						StructuralNavigation(
							Direction::Backward,
							Role::Link,
						)
						.into(),
					),
				])
				.unwrap(),
			),
		])
		.unwrap()
	}
}

impl ComboSets {
	/// Iterate over the items in [`ComboSets`].
	pub fn iter(&self) -> std::slice::Iter<'_, (Option<Mode>, ComboSet)> {
		<&Self as IntoIterator>::into_iter(self)
	}
}

impl<'a> IntoIterator for &'a ComboSets {
	type IntoIter = std::slice::Iter<'a, (Option<Mode>, ComboSet)>;
	type Item = &'a (Option<Mode>, ComboSet);
	fn into_iter(self) -> Self::IntoIter {
		self.inner.iter()
	}
}

/// The primary holder of state for all keybindings in the daemon.
#[derive(Debug)]
pub struct State {
	/// If the activation key ([`crate::ACTIVATION_KEY`]) is pressed.
	pub activation_key_pressed: bool,
	/// Which mode the screen reader is in.
	pub mode: Mode,
	/// All pressed keys _after_ activation is pressed.
	pub pressed: Vec<Key>,
	/// List of key combos.
	pub combos: ComboSets,
	/// A synchronous channel to send events to.
	/// The receiver will send them over a socket to the main Odilia process.
	pub tx: SyncSender<OdiliaEvent>,
}

/// The callback function to call in a tight loop.
/// Returns [`None`] to indicate a desire to swallow an event,
/// Returns [`Some(Event)`] to indicate a passthrough of the event.
///
/// # Panics
///
/// If the [`State`]'s [`SyncSender`] for the [`OdiliaEvent`] is unable to be sent to.
pub fn callback(event: Event, state: &mut State) -> Option<Event> {
	tracing::debug!("Callback called for {event:?}");
	match (event.event_type, state.activation_key_pressed) {
		// if capslock is pressed while activation is disabled
		(EventType::KeyPress(ACTIVATION_KEY), false) => {
			// enable it
			state.activation_key_pressed = true;
			tracing::trace!("Activation enabled!");
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
			tracing::trace!("Activation disabled!");
			// and swallow event
			None
		}
		// if a key press is made while activation is enabled
		(EventType::KeyPress(other), true) => {
			// if the key is already pressed (i.e., it's been held down)
			let None = state.pressed.iter().position(|key| *key == other) else {
				// swallow the event immediately, do not pass through
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
					// if a combo matches the held keys (must be in right order)
					if combo.0 == *state.pressed {
						// print out the command
						tracing::debug!("Combo found for {:?}", combo.1);
						// if it's a change mode event, update the mode
						if let OdiliaEvent::ChangeMode(ChangeMode(
							new_mode,
						)) = combo.1
						{
							state.mode = new_mode;
						}
						state.tx.send(combo.1.clone()).expect(
                "To be able to send the combo over the channel",
              );
						// exit early; found combo!
						return None;
					}
				}
			}
			// swallow the event
			None
		}
		// if a key release is made while activation mode is on
		(EventType::KeyRelease(other), _) => {
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
