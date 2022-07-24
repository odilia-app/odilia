use crate::errors::KeyFromStrError;
use crate::modes::ScreenReaderMode;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageDown,
    PageUp,
    Backspace,
    Delete,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Return,
    Space,
    Tab,
    PrintScreen,
    ScrollLock,
    Pause,
    NumLock,
    KpReturn,
    KpMinus,
    KpPlus,
    KpMultiply,
    KpDivide,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDelete,
    Function,
    Insert,
    Other(char),
}

impl FromStr for Key {
    type Err = KeyFromStrError;

    fn from_str(key: &str) -> Result<Self, Self::Err> {
        use KeyFromStrError as E;

        if key.len() <= 1 {
            let c = key.chars().next().ok_or(E::EmptyKey)?;
            if !c.is_control() && !c.is_whitespace() {
                return Ok(Self::Other(c));
            }
        }
        Ok(match key {
            // Special cases
            k if k.eq_ignore_ascii_case("Up") => Self::Up,
            k if k.eq_ignore_ascii_case("Down") => Self::Down,
            k if k.eq_ignore_ascii_case("Left") => Self::Left,
            k if k.eq_ignore_ascii_case("Right") => Self::Right,
            k if k.eq_ignore_ascii_case("Home") => Self::Home,
            k if k.eq_ignore_ascii_case("End") => Self::End,
            k if k.eq_ignore_ascii_case("PageDown") => Self::PageDown,
            k if k.eq_ignore_ascii_case("PageUp") => Self::PageUp,
            k if k.eq_ignore_ascii_case("Delete") => Self::Delete,
            k if k.eq_ignore_ascii_case("Escape") => Self::Escape,
            k if k.eq_ignore_ascii_case("F1") => Self::F1,
            k if k.eq_ignore_ascii_case("F2") => Self::F2,
            k if k.eq_ignore_ascii_case("F3") => Self::F3,
            k if k.eq_ignore_ascii_case("F4") => Self::F4,
            k if k.eq_ignore_ascii_case("F5") => Self::F5,
            k if k.eq_ignore_ascii_case("F6") => Self::F6,
            k if k.eq_ignore_ascii_case("F7") => Self::F7,
            k if k.eq_ignore_ascii_case("F8") => Self::F8,
            k if k.eq_ignore_ascii_case("F9") => Self::F9,
            k if k.eq_ignore_ascii_case("F10") => Self::F10,
            k if k.eq_ignore_ascii_case("F11") => Self::F11,
            k if k.eq_ignore_ascii_case("F12") => Self::F12,
            k if k.eq_ignore_ascii_case("Return") => Self::Return,
            k if k.eq_ignore_ascii_case("Space") => Self::Space,
            k if k.eq_ignore_ascii_case("Tab") => Self::Tab,
            k if k.eq_ignore_ascii_case("PrintScreen") => Self::PrintScreen,
            k if k.eq_ignore_ascii_case("ScrollLock") => Self::ScrollLock,
            k if k.eq_ignore_ascii_case("Pause") => Self::Pause,
            k if k.eq_ignore_ascii_case("NumLock") => Self::NumLock,
            k if k.eq_ignore_ascii_case("KpReturn") => Self::KpReturn,
            k if k.eq_ignore_ascii_case("KpMinus") => Self::KpMinus,
            k if k.eq_ignore_ascii_case("KpPlus") => Self::KpPlus,
            k if k.eq_ignore_ascii_case("KpMultiply") => Self::KpMultiply,
            k if k.eq_ignore_ascii_case("KpDivide") => Self::KpDivide,
            k if k.eq_ignore_ascii_case("Kp0") => Self::Kp0,
            k if k.eq_ignore_ascii_case("Kp1") => Self::Kp1,
            k if k.eq_ignore_ascii_case("Kp2") => Self::Kp2,
            k if k.eq_ignore_ascii_case("Kp3") => Self::Kp3,
            k if k.eq_ignore_ascii_case("Kp4") => Self::Kp4,
            k if k.eq_ignore_ascii_case("Kp5") => Self::Kp5,
            k if k.eq_ignore_ascii_case("Kp6") => Self::Kp6,
            k if k.eq_ignore_ascii_case("Kp7") => Self::Kp7,
            k if k.eq_ignore_ascii_case("Kp8") => Self::Kp8,
            k if k.eq_ignore_ascii_case("Kp9") => Self::Kp9,
            k if k.eq_ignore_ascii_case("KpDelete") => Self::KpDelete,

            _ => return Err(E::InvalidKey(key.into())),
        })
    }
}

/* Notice it has almost the same fields as KeyBinding. */
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub key: Option<Key>,
    pub mods: Modifiers,
    pub repeat: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: Option<Key>,
    pub mods: Modifiers,
    pub repeat: u8,
    /* if none, match all modes */
    pub mode: Option<ScreenReaderMode>,
    /* whether or not to consume the event, or let it pass through */
    pub consume: bool,
    /* whether to notify the SR that the key has been pressed; currently at least one function in -prototype/main.rs will ALWAYS see every key, but this could change. */
    pub notify: bool,
}

/* get mode and return it with a stripped version of the string */
fn get_mode_strip(s: &str) -> (Option<ScreenReaderMode>, String) {
  let new_str: String;
  let mode_index = s.find("|");
  let mode: Option<ScreenReaderMode> = match mode_index {
    Some(mode_index) => {
      new_str = s.get(mode_index+1..).unwrap().to_string(); // pretty sure is safe
      Some(ScreenReaderMode {
        name: s.get(..mode_index).unwrap().to_string() // mostly safe I think?
      })
    },
    _ => {
      new_str = s.to_string().clone();
      None
    },
  };

  (mode, new_str)
}
fn get_consume_strip(s: &str) -> (bool, String) {
  let new_str: String;
  let consume_index = s.find("|");
  let consume: bool = match consume_index {
    Some(consume_index) => {
      new_str = s.get(consume_index+1..).unwrap().to_string(); // pretty sure is safe
      let b_str = s.get(..consume_index).unwrap().to_string(); // mostly safe I think?
      b_str == "C" 
    },
    _ => {
      new_str = s.to_string().clone();
      false
    },
  };

  (consume, new_str)
}

impl FromStr for KeyBinding {
    type Err = KeyFromStrError;

    fn from_str(s1: &str) -> Result<Self, Self::Err> {
        use KeyFromStrError as E;
        let (consume, s2) = get_consume_strip(&s1);
        let (mode, s) = get_mode_strip(&s2);

        let mut parts = s.rsplit('+').map(str::trim);
        let key_and_repeat = parts.next().ok_or(E::EmptyString)?;
        let mut subparts = key_and_repeat.split(':');

        let key: Key = subparts.next().ok_or(E::NoKey)?.parse()?;

        let repeat: u8 = {
            let repeat_str = subparts.next().unwrap_or("1");
            repeat_str
                .parse()
                .map_err(|_| E::InvalidRepeat(repeat_str.into()))?
        };

        let mut mods = Modifiers::empty();
        for m in parts {
            // Yeh ... it's not pretty ... but would an if chain really look any better?
            mods |= match m {
                m if m.eq_ignore_ascii_case("Odilia") => Modifiers::ODILIA,
                m if m.eq_ignore_ascii_case("Applications") => Modifiers::APPLICATIONS,

                m if m.eq_ignore_ascii_case("LeftControl") => Modifiers::CONTROL_L,
                m if m.eq_ignore_ascii_case("RightControl") => Modifiers::CONTROL_R,
                m if m.eq_ignore_ascii_case("Control") => Modifiers::CONTROL,

                m if m.eq_ignore_ascii_case("LeftAlt") => Modifiers::ALT_L,
                m if m.eq_ignore_ascii_case("RightAlt") => Modifiers::ALT_R,
                m if m.eq_ignore_ascii_case("Alt") => Modifiers::ALT,

                m if m.eq_ignore_ascii_case("LeftShift") => Modifiers::SHIFT_L,
                m if m.eq_ignore_ascii_case("RightShift") => Modifiers::SHIFT_R,
                m if m.eq_ignore_ascii_case("Shift") => Modifiers::SHIFT,

                m if m.eq_ignore_ascii_case("LeftMeta") => Modifiers::META_L,
                m if m.eq_ignore_ascii_case("RightMeta") => Modifiers::META_R,
                m if m.eq_ignore_ascii_case("Meta") => Modifiers::META,

                _ => return Err(E::InvalidModifier(m.into())),
            };
        }

        Ok(Self {
            key: Some(key),
            mods,
            repeat,
            mode,
            consume,
            notify: true,
        })
    }
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct Modifiers: u16 {
        const NONE = 0;

        /// Usually capslock, insert, or kp-insert
        const ODILIA = 1 << 0;

        const CONTROL_L = 1 << 1;
        const CONTROL_R = 1 << 2;
        const CONTROL = 1 << 1 | 1 << 2;

        const ALT_L = 1 << 3;
        const ALT_R = 1 << 4;
        const ALT = 1 << 3 | 1 << 4;

        const SHIFT_L = 1 << 5;
        const SHIFT_R = 1 << 6;
        const SHIFT = 1 << 5 | 1 << 6;

        const META_L = 1 << 7;
        const META_R = 1 << 8;
        const META = 1 << 7 | 1 << 8;

        const APPLICATIONS = 1 << 9;
    }
}

impl Modifiers {
    // Using `self` instead of `&self` here is fine, `Self` is `Copy`.
    //
    #[inline]
    pub fn control(self) -> bool {
        self.intersects(Self::CONTROL)
    }

    #[inline]
    pub fn alt(self) -> bool {
        self.intersects(Self::ALT)
    }

    #[inline]
    pub fn shift(self) -> bool {
        self.intersects(Self::SHIFT)
    }

    #[inline]
    pub fn meta(self) -> bool {
        self.intersects(Self::META)
    }
    // Are these two useful? Almost certainly not! Am I going to leave them in? You bet ya!
    // You never know, addon developers are clever!

    #[inline]
    pub fn left(self) -> bool {
        self.intersects(Self::CONTROL_L | Self::ALT_L | Self::SHIFT_L | Self::META_L)
    }

    #[inline]
    pub fn right(self) -> bool {
        self.intersects(Self::CONTROL_R | Self::ALT_R | Self::SHIFT_R | Self::META_R)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_key_binding() {
        // simple
        let kb: KeyBinding = "Odilia+h".parse().unwrap();
        println!("{:?}", kb);
        assert_eq!(kb.key, Some(Key::Other('h')));
        assert_eq!(kb.mods, Modifiers::ODILIA);
        assert_eq!(kb.repeat, 1);
        // With whitespace
        let kb: KeyBinding = "Odilia + h".parse().unwrap();
        assert_eq!(kb.key, Some(Key::Other('h')));
        assert_eq!(kb.mods, Modifiers::ODILIA);
        assert_eq!(kb.repeat, 1);
        // Complex
        let kb: KeyBinding = "Control+Shift+Alt+Meta+Applications+Odilia+Return:3"
            .parse()
            .unwrap();
        assert_eq!(kb.key, Some(Key::Return));
        assert_eq!(kb.mods, Modifiers::all());
        assert_eq!(kb.repeat, 3);
        // Left only
        let kb: KeyBinding = "LeftControl+LeftShift+LeftAlt+LeftMeta+.:2"
            .parse()
            .unwrap();
        assert_eq!(kb.key, Some(Key::Other('.')));
        assert_eq!(
            kb.mods,
            Modifiers::CONTROL_L | Modifiers::ALT_L | Modifiers::SHIFT_L | Modifiers::META_L
        );
        assert_eq!(kb.repeat, 2);
        let kb: KeyBinding = "RightControl+RightShift+RightAlt+RightMeta+.:2"
            .parse()
            .unwrap();
        assert_eq!(kb.key, Some(Key::Other('.')));
        assert_eq!(
            kb.mods,
            Modifiers::CONTROL_R | Modifiers::ALT_R | Modifiers::SHIFT_R | Modifiers::META_R
        );
        assert_eq!(kb.repeat, 2);
        assert_eq!(kb.consume, false);
        // test consume
        let kb: KeyBinding = "C|Odilia+h"
            .parse()
            .unwrap();
        assert_eq!(kb.consume, true);
        assert_eq!(kb.notify, true);
    }
}
