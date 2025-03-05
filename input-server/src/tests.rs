use crate::{ComboError, ComboSet, ComboSets, KeySet, Mode, SetError};
use odilia_common::events::*;
use rdev::Key;

#[test]
fn test_unreachable_mode() {
	let core_combos = ComboSet::from_iter(
		vec![(vec![Key::KeyA].try_into().unwrap(), ChangeMode(Mode::Browse).into())]
			.into_iter(),
	);
	let focus_combos = ComboSet::from_iter(
		vec![(vec![Key::KeyP].try_into().unwrap(), StopSpeech.into())].into_iter(),
	);
	let combos = ComboSets::try_from([(None, core_combos), (Some(Mode::Focus), focus_combos)]);
	assert_eq!(combos, Err(SetError::UnreachableMode(Mode::Focus)), "It should not be possible to construct the ComboSet when there is no way to activate that mode!");
}

#[test]
fn test_same_prefix() {
	let core_combos = ComboSet::from_iter(
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
	);
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
