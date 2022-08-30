use std::{future::Future, sync::Mutex};

use once_cell::sync::{Lazy, OnceCell};
use rdev::{
    Event,
    EventType::{KeyPress, KeyRelease},
    Key as RDevKey,
};
use tokio::sync::mpsc;

use crate::keybinds::keyevent_match_sync;
use odilia_common::{
    events::ScreenReaderEvent,
    input::{Key, KeyBinding, KeyEvent, Modifiers},
};

pub type AsyncFn =
    Box<dyn Fn() -> Box<dyn Future<Output = ()> + Unpin + Send + 'static> + Send + Sync + 'static>;

// These are to be used only from the input monitoring thread
thread_local! {
    /// The channel's [`mpsc::Sender`].
    static TX: OnceCell<mpsc::Sender<ScreenReaderEvent>> = OnceCell::new();
}

static CURRENT_KEYS: Lazy<Mutex<Vec<RDevKey>>> = Lazy::new(|| Mutex::new(Vec::new()));
static LAST_KEYS: Lazy<Mutex<Vec<RDevKey>>> = Lazy::new(|| Mutex::new(Vec::new()));

fn rdev_keys_to_odilia_modifiers(keys: &[RDevKey]) -> Modifiers {
    let mut modifiers = Modifiers::empty();
    for k in keys {
        modifiers |= match *k {
            k if k == RDevKey::CapsLock => Modifiers::ODILIA,
            k if k == RDevKey::Alt => Modifiers::ALT_L,
            k if k == RDevKey::AltGr => Modifiers::ALT_R,
            k if k == RDevKey::ControlLeft => Modifiers::CONTROL_L,
            k if k == RDevKey::ControlRight => Modifiers::CONTROL_R,
            k if k == RDevKey::ShiftLeft => Modifiers::SHIFT_L,
            k if k == RDevKey::ShiftRight => Modifiers::SHIFT_R,
            k if k == RDevKey::MetaLeft => Modifiers::META_L,
            k if k == RDevKey::MetaRight => Modifiers::META_R,
            _ => Modifiers::empty(),
        }
    }
    modifiers
}

/* NOTE: this breaks if a user pressed a combination with two letters, i.e.: Ctrl+Shift+a+n, or CapsLock+a+s.
This function will always return the first pressed key (a and a in our examples).
*/
fn rdev_keys_to_single_odilia_key(keys: &[RDevKey]) -> Option<Key> {
    for k in keys {
        let m = match k {
            RDevKey::Backspace => Some(Key::Backspace),
            RDevKey::Delete => Some(Key::Delete),
            RDevKey::DownArrow => Some(Key::Down),
            RDevKey::UpArrow => Some(Key::Up),
            RDevKey::LeftArrow => Some(Key::Left),
            RDevKey::RightArrow => Some(Key::Right),
            RDevKey::End => Some(Key::End),
            RDevKey::Escape => Some(Key::Escape),
            RDevKey::F1 => Some(Key::F1),
            RDevKey::F2 => Some(Key::F2),
            RDevKey::F3 => Some(Key::F3),
            RDevKey::F4 => Some(Key::F4),
            RDevKey::F5 => Some(Key::F5),
            RDevKey::F6 => Some(Key::F6),
            RDevKey::F7 => Some(Key::F7),
            RDevKey::F8 => Some(Key::F8),
            RDevKey::F9 => Some(Key::F9),
            RDevKey::F10 => Some(Key::F10),
            RDevKey::F11 => Some(Key::F11),
            RDevKey::F12 => Some(Key::F12),
            RDevKey::Home => Some(Key::Home),
            RDevKey::PageDown => Some(Key::PageDown),
            RDevKey::PageUp => Some(Key::PageUp),
            RDevKey::Return => Some(Key::Return),
            RDevKey::Space => Some(Key::Space),
            RDevKey::Tab => Some(Key::Tab),
            RDevKey::PrintScreen => Some(Key::PrintScreen),
            RDevKey::ScrollLock => Some(Key::ScrollLock),
            RDevKey::Pause => Some(Key::Pause),
            RDevKey::NumLock => Some(Key::NumLock),
            RDevKey::BackQuote => Some(Key::Other('`')),
            RDevKey::Num0 => Some(Key::Kp0),
            RDevKey::Num1 => Some(Key::Kp1),
            RDevKey::Num2 => Some(Key::Kp2),
            RDevKey::Num3 => Some(Key::Kp3),
            RDevKey::Num4 => Some(Key::Kp4),
            RDevKey::Num5 => Some(Key::Kp5),
            RDevKey::Num6 => Some(Key::Kp6),
            RDevKey::Num7 => Some(Key::Kp7),
            RDevKey::Num8 => Some(Key::Kp8),
            RDevKey::Num9 => Some(Key::Kp9),
            RDevKey::Minus => Some(Key::Other('-')),
            RDevKey::Equal => Some(Key::Other('=')),
            RDevKey::KeyQ => Some(Key::Other('q')),
            RDevKey::KeyW => Some(Key::Other('w')),
            RDevKey::KeyE => Some(Key::Other('e')),
            RDevKey::KeyR => Some(Key::Other('r')),
            RDevKey::KeyT => Some(Key::Other('t')),
            RDevKey::KeyY => Some(Key::Other('y')),
            RDevKey::KeyU => Some(Key::Other('u')),
            RDevKey::KeyI => Some(Key::Other('i')),
            RDevKey::KeyO => Some(Key::Other('o')),
            RDevKey::KeyP => Some(Key::Other('p')),
            RDevKey::LeftBracket => Some(Key::Other('[')),
            RDevKey::RightBracket => Some(Key::Other(']')),
            RDevKey::KeyA => Some(Key::Other('a')),
            RDevKey::KeyS => Some(Key::Other('s')),
            RDevKey::KeyD => Some(Key::Other('d')),
            RDevKey::KeyF => Some(Key::Other('f')),
            RDevKey::KeyG => Some(Key::Other('g')),
            RDevKey::KeyH => Some(Key::Other('h')),
            RDevKey::KeyJ => Some(Key::Other('j')),
            RDevKey::KeyK => Some(Key::Other('k')),
            RDevKey::KeyL => Some(Key::Other('l')),
            RDevKey::SemiColon => Some(Key::Other(';')),
            RDevKey::Quote => Some(Key::Other('\'')),
            RDevKey::BackSlash => Some(Key::Other('\\')),
            // TODO: check if correct belo)w
            RDevKey::IntlBackslash => Some(Key::Other('\\')),
            RDevKey::KeyZ => Some(Key::Other('z')),
            RDevKey::KeyX => Some(Key::Other('x')),
            RDevKey::KeyC => Some(Key::Other('c')),
            RDevKey::KeyV => Some(Key::Other('v')),
            RDevKey::KeyB => Some(Key::Other('b')),
            RDevKey::KeyN => Some(Key::Other('n')),
            RDevKey::Comma => Some(Key::Other(',')),
            RDevKey::Dot => Some(Key::Other('.')),
            RDevKey::Slash => Some(Key::Other('/')),
            RDevKey::Insert => Some(Key::Insert),
            RDevKey::KpReturn => Some(Key::KpReturn),
            RDevKey::KpMinus => Some(Key::KpMinus),
            RDevKey::KpPlus => Some(Key::KpPlus),
            RDevKey::KpMultiply => Some(Key::KpMultiply),
            RDevKey::KpDivide => Some(Key::KpDivide),
            RDevKey::KpDelete => Some(Key::KpDelete),
            RDevKey::Function => Some(Key::Function),
            _ => None,
        };
        if let Some(m2) = m {
            return Some(m2);
        }
    }
    None
}

fn rdev_event_to_odilia_event(events: &[RDevKey]) -> KeyEvent {
    KeyEvent {
        key: rdev_keys_to_single_odilia_key(events),
        mods: rdev_keys_to_odilia_modifiers(events),
        /* TODO: set repeat properly */
        repeat: 1,
    }
}

fn is_new_key_event(
    event: &Event,
    current_keys: &mut Vec<RDevKey>,
    last_keys: &mut Vec<RDevKey>,
) -> bool {
    match event.event_type {
        KeyPress(x) => {
            *last_keys = current_keys.clone();
            current_keys.push(x);
            current_keys.dedup();
            // if there is a new key pressed/released and it is not a repeat event
            last_keys != current_keys
        }
        KeyRelease(x) => {
            *last_keys = current_keys.clone();
            // remove just released key from curent keys
            current_keys.retain(|&k| k != x);
            false
        }
        _ => false,
    }
}

/// The maximum number of `[rdev::Event`]s that can be in the input queue at one time.
/// The queue could be unbounded, but this allows for backpressure, which allows us to catch up if
/// we get spammed with events.
///
/// On x86_64-unknown-linux-gnu, [`rdev::Event`] is 64 bytes, so this is 16 KiB of queue.
const MAX_EVENTS: usize = 256;

/// Initialise the input monitoring system, returning an [`mpsc::Receiver`] which can be used to
/// recieve input events.
///
/// `decide_action` will be used to determine whether the [`Event`][rdev::Event] is consumed, and
/// also whether we are notified about it via the channel.
/// # Panics
/// * If called more than once in the same program.
pub fn create_keybind_channel() -> mpsc::Receiver<ScreenReaderEvent> {
    // Create the channel for communication between the input monitoring thread and async tasks
    let (tx, rx) = mpsc::channel(MAX_EVENTS);

    // Spawn a synchronous input monitoring thread
    std::thread::spawn(move || {
        // should work as long as called from a tokio runtime
        // Set the thread-local variables
        TX.with(|global| global.set(tx).unwrap());
        // Start the event loop
        rdev::grab(move |ev| {
            let mut current_keys = CURRENT_KEYS.lock().unwrap();
            let mut last_keys = LAST_KEYS.lock().unwrap();
            // if the event is not new (i.e. a held key), just passthrough the event
            if !is_new_key_event(&ev, &mut current_keys, &mut last_keys) {
                return Some(ev);
            }
            TX.with(|tx| {
                // Decide what to do with this `Event`
                let o_event = rdev_event_to_odilia_event(&current_keys);
                let keybind_event_match: Option<(KeyBinding, ScreenReaderEvent)> =
                    keyevent_match_sync(&o_event);
                /* if a matching keybinding is not found, pass through the event */
                if keybind_event_match.is_none() {
                    return Some(ev);
                }
                let keybind_event_match = keybind_event_match.unwrap(); // should never panic due to above if
                let tx = tx.get().unwrap();
                if keybind_event_match.0.notify {
                    // Notify us by sending the `Event` down the channel
                    if let Err(e) = tx.blocking_send(keybind_event_match.1) {
                        eprintln!("Warning: Failed to process key event: {}", e);
                    }
                }
                // Decide whether to consume the action or pass it through
                if keybind_event_match.0.consume {
                    None
                } else {
                    Some(ev)
                }
            })
        })
    });

    rx // Return the receiving end of the channel
}
