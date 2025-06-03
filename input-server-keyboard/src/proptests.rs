use std::{
	sync::mpsc::{Receiver, TryRecvError},
	time::SystemTime,
};

use atspi_common::Role;
use odilia_common::events::*;
use proptest::prelude::*;
use rdev::{Button, Event, EventType, Key};

use crate::{
	callback, tests::EventFromEventType, ComboSet, ComboSets, KeySet, Mode, OdiliaEvent, State,
	ACTIVATION_KEY,
};

impl ComboSets {
	/// Create a [`ComboSets`] from an iterator.
	/// Ignores all errors.
	#[cfg(feature = "proptest")]
	pub fn from_iter_ignore_errors<I>(iter: I) -> Self
	where
		I: Iterator<Item = (Option<Mode>, ComboSet)>,
	{
		let mut this = Self::new();
		iter.for_each(|item| {
			let _ = this.insert(item.0, item.1);
		});
		this
	}
}

impl ComboSet {
	/// Create a [`ComboSet`] from an iterator.
	/// Ignores all errors.
	#[cfg(feature = "proptest")]
	pub fn from_iter_ignore_errors<I>(iter: I) -> Self
	where
		I: Iterator<Item = (KeySet, OdiliaEvent)>,
	{
		let mut this = Self::new();
		iter.for_each(|item| {
			let _ = this.insert(item.0, item.1);
		});
		this
	}
}

impl KeySet {
	#[cfg(all(test, feature = "proptest"))]
	/// Create a `KeySet` from a list of keys.
	/// Automatically reject and deduplicate the keys during insertion.
	/// While this can not fail, it will simply throw out any key which is the [`ACTIVATION_KEY`] or
	/// a repeated key that is already contained within it.
	///
	/// NOTE: Only used during proptests. This should never become part of the public API.
	fn from_dedup(v: Vec<Key>) -> Self {
		let mut this = Self::new();
		for item in v {
			let _ = this.insert(item);
		}
		this
	}
}

#[allow(clippy::too_many_lines)]
fn key() -> impl Strategy<Value = Key> {
	prop_oneof![
		Just(Key::Alt),
		Just(Key::AltGr),
		Just(Key::Backspace),
		Just(Key::CapsLock),
		Just(Key::ControlLeft),
		Just(Key::ControlRight),
		Just(Key::Delete),
		Just(Key::DownArrow),
		Just(Key::End),
		Just(Key::Escape),
		Just(Key::F1),
		Just(Key::F10),
		Just(Key::F11),
		Just(Key::F12),
		Just(Key::F2),
		Just(Key::F3),
		Just(Key::F4),
		Just(Key::F5),
		Just(Key::F6),
		Just(Key::F7),
		Just(Key::F8),
		Just(Key::F9),
		Just(Key::Home),
		Just(Key::LeftArrow),
		Just(Key::MetaLeft),
		Just(Key::MetaRight),
		Just(Key::PageDown),
		Just(Key::PageUp),
		Just(Key::Return),
		Just(Key::RightArrow),
		Just(Key::ShiftLeft),
		Just(Key::ShiftRight),
		Just(Key::Space),
		Just(Key::Tab),
		Just(Key::UpArrow),
		Just(Key::PrintScreen),
		Just(Key::ScrollLock),
		Just(Key::Pause),
		Just(Key::NumLock),
		Just(Key::BackQuote),
		Just(Key::Num1),
		Just(Key::Num2),
		Just(Key::Num3),
		Just(Key::Num4),
		Just(Key::Num5),
		Just(Key::Num6),
		Just(Key::Num7),
		Just(Key::Num8),
		Just(Key::Num9),
		Just(Key::Num0),
		Just(Key::Minus),
		Just(Key::Equal),
		Just(Key::KeyQ),
		Just(Key::KeyW),
		Just(Key::KeyE),
		Just(Key::KeyR),
		Just(Key::KeyT),
		Just(Key::KeyY),
		Just(Key::KeyU),
		Just(Key::KeyI),
		Just(Key::KeyO),
		Just(Key::KeyP),
		Just(Key::LeftBracket),
		Just(Key::RightBracket),
		Just(Key::KeyA),
		Just(Key::KeyS),
		Just(Key::KeyD),
		Just(Key::KeyF),
		Just(Key::KeyG),
		Just(Key::KeyH),
		Just(Key::KeyJ),
		Just(Key::KeyK),
		Just(Key::KeyL),
		Just(Key::SemiColon),
		Just(Key::Quote),
		Just(Key::BackSlash),
		Just(Key::IntlBackslash),
		Just(Key::KeyZ),
		Just(Key::KeyX),
		Just(Key::KeyC),
		Just(Key::KeyV),
		Just(Key::KeyB),
		Just(Key::KeyN),
		Just(Key::KeyM),
		Just(Key::Comma),
		Just(Key::Dot),
		Just(Key::Slash),
		Just(Key::Insert),
		Just(Key::KpReturn),
		Just(Key::KpMinus),
		Just(Key::KpPlus),
		Just(Key::KpMultiply),
		Just(Key::KpDivide),
		Just(Key::Kp0),
		Just(Key::Kp1),
		Just(Key::Kp2),
		Just(Key::Kp3),
		Just(Key::Kp4),
		Just(Key::Kp5),
		Just(Key::Kp6),
		Just(Key::Kp7),
		Just(Key::Kp8),
		Just(Key::Kp9),
		Just(Key::KpDelete),
		Just(Key::Function),
		// technically should not be "any"
		any::<u32>().prop_map(Key::Unknown),
	]
}

fn button() -> impl Strategy<Value = Button> {
	prop_oneof![
		Just(Button::Left),
		Just(Button::Right),
		Just(Button::Middle),
		any::<u8>().prop_map(Button::Unknown),
	]
}

fn event_type() -> impl Strategy<Value = EventType> {
	prop_oneof![
		key().prop_map(EventType::KeyPress),
		key().prop_map(EventType::KeyRelease),
		button().prop_map(EventType::ButtonPress),
		button().prop_map(EventType::ButtonRelease),
		any::<(f64, f64)>().prop_map(|(x, y)| EventType::MouseMove { x, y }),
		any::<(i64, i64)>()
			.prop_map(|(delta_x, delta_y)| EventType::Wheel { delta_x, delta_y })
	]
}

fn event() -> impl Strategy<Value = Event> {
	event_type().prop_map(|event_type| Event {
		event_type,
		// both these types are ignored in the event processor
		time: SystemTime::now(),
		name: None,
	})
}

fn events_all_release() -> impl Strategy<Value = Vec<Event>> {
	prop::collection::vec(
		key().prop_map(|key| Event {
			event_type: EventType::KeyRelease(key),
			// neither are used
			time: SystemTime::now(),
			name: None,
		}),
		0..100,
	)
	.prop_flat_map(Just)
}

fn events() -> impl Strategy<Value = (Vec<Event>, usize)> {
	prop::collection::vec(event(), 1..50).prop_flat_map(|vec| {
		let len = vec.len();
		(Just(vec), 0..len)
	})
}

fn mode() -> impl Strategy<Value = Mode> {
	prop_oneof![Just(Mode::Focus), Just(Mode::Browse),]
}

fn mode_option() -> impl Strategy<Value = Option<Mode>> {
	prop_oneof![Just(None), mode().prop_map(Some)]
}

#[allow(clippy::too_many_lines)]
fn role() -> impl Strategy<Value = Role> {
	prop_oneof![
		Just(Role::Invalid),
		Just(Role::AcceleratorLabel),
		Just(Role::Alert),
		Just(Role::Animation),
		Just(Role::Arrow),
		Just(Role::Calendar),
		Just(Role::Canvas),
		Just(Role::CheckBox),
		Just(Role::CheckMenuItem),
		Just(Role::ColorChooser),
		Just(Role::ColumnHeader),
		Just(Role::ComboBox),
		Just(Role::DateEditor),
		Just(Role::DesktopIcon),
		Just(Role::DesktopFrame),
		Just(Role::Dial),
		Just(Role::Dialog),
		Just(Role::DirectoryPane),
		Just(Role::DrawingArea),
		Just(Role::FileChooser),
		Just(Role::Filler),
		Just(Role::FocusTraversable),
		Just(Role::FontChooser),
		Just(Role::Frame),
		Just(Role::GlassPane),
		Just(Role::HTMLContainer),
		Just(Role::Icon),
		Just(Role::Image),
		Just(Role::InternalFrame),
		Just(Role::Label),
		Just(Role::LayeredPane),
		Just(Role::List),
		Just(Role::ListItem),
		Just(Role::Menu),
		Just(Role::MenuBar),
		Just(Role::MenuItem),
		Just(Role::OptionPane),
		Just(Role::PageTab),
		Just(Role::PageTabList),
		Just(Role::Panel),
		Just(Role::PasswordText),
		Just(Role::PopupMenu),
		Just(Role::ProgressBar),
		Just(Role::Button),
		Just(Role::RadioButton),
		Just(Role::RadioMenuItem),
		Just(Role::RootPane),
		Just(Role::RowHeader),
		Just(Role::ScrollBar),
		Just(Role::ScrollPane),
		Just(Role::Separator),
		Just(Role::Slider),
		Just(Role::SpinButton),
		Just(Role::SplitPane),
		Just(Role::StatusBar),
		Just(Role::Table),
		Just(Role::TableCell),
		Just(Role::TableColumnHeader),
		Just(Role::TableRowHeader),
		Just(Role::TearoffMenuItem),
		Just(Role::Terminal),
		Just(Role::Text),
		Just(Role::ToggleButton),
		Just(Role::ToolBar),
		Just(Role::ToolTip),
		Just(Role::Tree),
		Just(Role::TreeTable),
		Just(Role::Unknown),
		Just(Role::Viewport),
		Just(Role::Window),
		Just(Role::Extended),
		Just(Role::Header),
		Just(Role::Footer),
		Just(Role::Paragraph),
		Just(Role::Ruler),
		Just(Role::Application),
		Just(Role::Autocomplete),
		Just(Role::Editbar),
		Just(Role::Embedded),
		Just(Role::Entry),
		Just(Role::CHART),
		Just(Role::Caption),
		Just(Role::DocumentFrame),
		Just(Role::Heading),
		Just(Role::Page),
		Just(Role::Section),
		Just(Role::RedundantObject),
		Just(Role::Form),
		Just(Role::Link),
		Just(Role::InputMethodWindow),
		Just(Role::TableRow),
		Just(Role::TreeItem),
		Just(Role::DocumentSpreadsheet),
		Just(Role::DocumentPresentation),
		Just(Role::DocumentText),
		Just(Role::DocumentWeb),
		Just(Role::DocumentEmail),
		Just(Role::Comment),
		Just(Role::ListBox),
		Just(Role::Grouping),
		Just(Role::ImageMap),
		Just(Role::Notification),
		Just(Role::InfoBar),
		Just(Role::LevelBar),
		Just(Role::TitleBar),
		Just(Role::BlockQuote),
		Just(Role::Audio),
		Just(Role::Video),
		Just(Role::Definition),
		Just(Role::Article),
		Just(Role::Landmark),
		Just(Role::Log),
		Just(Role::Marquee),
		Just(Role::Math),
		Just(Role::Rating),
		Just(Role::Timer),
		Just(Role::Static),
		Just(Role::MathFraction),
		Just(Role::MathRoot),
		Just(Role::Subscript),
		Just(Role::Superscript),
		Just(Role::DescriptionList),
		Just(Role::DescriptionTerm),
		Just(Role::DescriptionValue),
		Just(Role::Footnote),
		Just(Role::ContentDeletion),
		Just(Role::ContentInsertion),
		Just(Role::Mark),
		Just(Role::Suggestion),
		Just(Role::PushButtonMenu),
	]
}

fn direction() -> impl Strategy<Value = Direction> {
	prop_oneof![Just(Direction::Forward), Just(Direction::Backward),]
}

fn feature() -> impl Strategy<Value = Feature> {
	prop_oneof![Just(Feature::Speech), Just(Feature::Braille),]
}

fn odilia_event() -> impl Strategy<Value = OdiliaEvent> {
	prop_oneof![
		Just(OdiliaEvent::StopSpeech(StopSpeech)),
		feature().prop_map(|feat| OdiliaEvent::Enable(Enable(feat))),
		feature().prop_map(|feat| OdiliaEvent::Disable(Disable(feat))),
		mode().prop_map(|mode| OdiliaEvent::ChangeMode(ChangeMode(mode))),
		(direction(), role()).prop_map(|(dir, rle)| OdiliaEvent::StructuralNavigation(
			StructuralNavigation(dir, rle)
		)),
	]
}

fn combo() -> impl Strategy<Value = (KeySet, OdiliaEvent)> {
	(prop::collection::vec(key(), 1..20).prop_map(KeySet::from_dedup), odilia_event())
}

fn combo_set() -> impl Strategy<Value = ComboSet> {
	prop::collection::vec(combo(), 1..20)
		.prop_map(|v| ComboSet::from_iter_ignore_errors(v.into_iter()))
}

fn combo_sets() -> impl Strategy<Value = ComboSets> {
	prop::collection::vec((mode_option(), combo_set()), 0..20)
		.prop_map(|v| ComboSets::from_iter_ignore_errors(v.into_iter()))
}

prop_compose! {
    fn state()
	(cmbs in combo_sets()) -> (State, Receiver<OdiliaEvent>) {
	let (mut state, rx) = State::new_unbounded();
	state.combos = cmbs;
	(state, rx)
    }
}

proptest! {
    #[test]
    fn test_all_keybindings_capture(
	(mut state, rx) in state(),
    ) {
	let combo_sets = state.combos.clone();
	let caps_press = Event::from_event_type(EventType::KeyPress(ACTIVATION_KEY));
	callback(caps_press, &mut state);
	for (mode, combos) in combo_sets {
	    for (combo, odilia_trigger) in combos {
	    // directly set mode required to trigger the given combo
	    if let Some(mode) = mode {
		state.mode = mode;
	    }
		for key in combo.clone() {
		    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty), "An OdiliaCommand was sent before a full keybinding has been pressed!");
		    callback(Event::from_event_type(EventType::KeyPress(key)), &mut state);
		}
		assert_eq!(rx.try_recv(), Ok(odilia_trigger), "All keys were pressed to produce an event; either the wrong event was sent, or there was some error sending it!");
		for key in combo {
		    callback(Event::from_event_type(EventType::KeyRelease(key)), &mut state);
		}
		assert_eq!(state.pressed.len(), 0, "Pressed keys is not 0-length after releasing them!");
	    }
	}
    }
    #[test]
    fn all_release_all_passthrough(
	events in events_all_release(),
	(mut state, _rx) in state(),
    ) {
	for event in events {
	    let ev1 = event.clone();
	    assert_eq!(callback(event, &mut state), Some(ev1), "Despite all events being key release events, a key was not passed through!");
	}
    }

    #[test]
    fn all_unused_keys_are_passed_through_and_capslock_always_consumed(
	(events, _size) in events(),
	(mut state, _rx) in state(),
    ) {
	let mut caps_held = false;
	let all_grabbable_keys: Vec<Key> = state.combos.inner.iter()
	    .flat_map(|combos| combos.1.keys().map(|combo_set| combo_set.inner.clone()))
	    .flatten()
	    .collect();
	for event in events {
	    let ev1 = event.clone();
	    match ev1.event_type {
		EventType::KeyPress(ACTIVATION_KEY) => {
		    caps_held = true;
		    assert_eq!(callback(event, &mut state), None, "CapsLock should always be captured!");
		}
		// ignore case where capslock release is generated before a capslock press
		EventType::KeyRelease(ACTIVATION_KEY) if caps_held => {
		    caps_held = false;
		    assert_eq!(callback(event, &mut state), None, "CapsLock should always be captured!");
		}
		EventType::KeyPress(key) | EventType::KeyRelease(key) => {
		    let ev2 = event.clone();
	// If it was pressed _during_ the holding of capslock, make sure to capture its release, even
	// if it was released _after_ caps has been released.
		    if !all_grabbable_keys.contains(&key) && !caps_held && !state.pressed.contains(&key) {
			assert_eq!(callback(event, &mut state), Some(ev2), "{key:?} is not in the grabale key list, but it still was captured!");
		    } else {
			let _ = callback(event, &mut state);
		    }
		},
		_ => {
		    let _ = callback(event, &mut state);
		},
	    }
	}
    }
    #[test]
    fn new_event_is_not_constructed(
	(events, _size) in events(),
	(mut state, _rx) in state(),
    ) {
	for event in events {
	    let ev1 = event.clone();
	    if let Some(ev2) = callback(event, &mut state) {
		assert_eq!(ev1, ev2);
	    }
	}
    }

		#[test]
		fn doesnt_panic(
	(events, _size) in events(),
	(mut state, _rx) in state(),
    ) {
			for ev in events {
				callback(ev, &mut state);
			}
		}
}
