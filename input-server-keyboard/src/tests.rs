use crate::{
	callback, ComboError, ComboSet, ComboSets, EventFromEventType, KeySet, Mode, SetError,
	State,
};
use odilia_common::events::*;
use rdev::{Event, EventType, Key};

#[test]
fn test_unreachable_mode() {
	let core_combos = ComboSet::try_from_iter(
		vec![(vec![Key::KeyA].try_into().unwrap(), ChangeMode(Mode::Browse).into())]
			.into_iter(),
	)
	.unwrap();
	let focus_combos = ComboSet::try_from_iter(
		vec![(vec![Key::KeyP].try_into().unwrap(), StopSpeech.into())].into_iter(),
	)
	.unwrap();
	let combos = ComboSets::try_from([(None, core_combos), (Some(Mode::Focus), focus_combos)]);
	assert_eq!(combos, Err(SetError::UnreachableMode(Mode::Focus)), "It should not be possible to construct the ComboSet when there is no way to activate that mode!");
}

#[test]
fn test_same_prefix() {
	let core_combos = ComboSet::try_from_iter(
		vec![
			(
				vec![Key::ShiftLeft, Key::KeyA].try_into().unwrap(),
				ChangeMode(Mode::Browse).into(),
			),
			(
				vec![Key::ShiftLeft, Key::KeyP].try_into().unwrap(),
				ChangeMode(Mode::Browse).into(),
			),
		]
		.into_iter(),
	)
	.unwrap();
	ComboSets::try_from([(None, core_combos)])
		.expect("Able to construct two bindings with the same prefix!");
}

#[test]
fn test_two_bindings_same_keys() {
	let shift_plus_a: KeySet = vec![Key::ShiftLeft, Key::KeyA].try_into().unwrap();
	let core_combos = ComboSet::try_from(vec![
		(shift_plus_a.clone(), ChangeMode(Mode::Browse).into()),
		(shift_plus_a.clone(), ChangeMode(Mode::Focus).into()),
	]);
	assert_eq!(core_combos, Err(ComboError::Identical(shift_plus_a)), "You should not be able to construct two key bindings in the same mode with identical keystrokes!");
}

#[test]
fn test_repeating_key_problem() {
	// Consuming a key release that was pressed before the activation key causes applications to
	// believe the g key is held down forever (until the user presses and releases g again)
	//
	// If this happens, it will cause numerous input-related bugs that are a pain in the ass to
	// solve.
	//
	// This is a minimal test case;
	// TODO: proptest for this!
	let press_g = Event::from_event_type(EventType::KeyPress(Key::KeyG));
	let release_g = Event::from_event_type(EventType::KeyRelease(Key::KeyG));
	let press_caps = Event::from_event_type(EventType::KeyPress(Key::CapsLock));
	let release_caps = Event::from_event_type(EventType::KeyRelease(Key::CapsLock));
	let events = [
		press_caps.clone(),
		press_g.clone(),
		release_caps.clone(),
		release_g.clone(),
		press_g.clone(),
		press_caps.clone(),
		release_g.clone(),
		release_caps.clone(),
	];
	let correct_return_values = [
		None,
		None,
		None,
		None,
		Some(press_g.clone()),
		None,
		Some(release_g.clone()),
		None,
	];
	let g: KeySet = vec![Key::KeyG].try_into().unwrap();
	let core_combos =
		ComboSet::try_from(vec![(g, StopSpeech.into())]).expect("Valid comboset!");
	let cs = ComboSets::try_from([(None, core_combos)]).expect("Valid combosets!");
	let (mut state, _rx) = State::new_unbounded();
	state.combos = cs;
	for (i, (ev, correct)) in
		events.into_iter().zip(correct_return_values.into_iter()).enumerate()
	{
		let val = callback(ev, &mut state);
		assert_eq!(val, correct, "Failed on action [{i}]!");
	}
}
