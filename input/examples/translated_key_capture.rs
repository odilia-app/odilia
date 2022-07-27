use breadx::Error;
use breadx::{prelude::*, display::DisplayConnection, protocol::xproto, Result};
use tokio;
use breadx::protocol::xproto::GrabMode;
use breadx::protocol::xproto::ModMask;
use breadx::protocol::Event;
use breadx::rt_support::tokio_support::connect;
use breadx::protocol::xproto::KeyPressEvent;
use breadx::protocol::xkb::KeySymMap;
use breadx_keysyms::KeyboardState;
use breadx_keysyms::keysyms;
use odilia_input::keys::Key as OKey;
use odilia_input::keys::Modifiers as OMods;
use breadx::protocol::xproto::Keysym;

fn xkeystate_to_omodifiers(xks: u16) -> OMods {
    let mut omods = OMods::NONE;
    let mms = u16::from(ModMask::SHIFT);
    let mml = u16::from(ModMask::LOCK);
    let mmc = u16::from(ModMask::CONTROL);
    if xks & mms == mms { omods |= OMods::SHIFT }
    if xks & mml == mml { omods |= OMods::ODILIA }
    if xks & mmc == mmc { omods |= OMods::CONTROL }
    return omods;
}

/* Methods implemented ON TOP of X11, i.e., IBus, Fcitx5, etc. will be ignored. Only methods
 * implemented BELOW X11 will affect the changing of these characters. */
fn xkeysym_to_okey(xks: Keysym) -> OKey {
    return match xks {
        keysyms::KEY_q => OKey::Other('q'),
        keysyms::KEY_w => OKey::Other('w'),
        keysyms::KEY_e => OKey::Other('e'),
        keysyms::KEY_r => OKey::Other('r'),
        keysyms::KEY_t => OKey::Other('t'),
        keysyms::KEY_y => OKey::Other('y'),
        keysyms::KEY_u => OKey::Other('u'),
        keysyms::KEY_i => OKey::Other('i'),
        keysyms::KEY_o => OKey::Other('o'),
        keysyms::KEY_p => OKey::Other('p'),
        keysyms::KEY_a => OKey::Other('a'),
        keysyms::KEY_s => OKey::Other('s'),
        keysyms::KEY_d => OKey::Other('d'),
        keysyms::KEY_f => OKey::Other('f'),
        keysyms::KEY_g => OKey::Other('g'),
        keysyms::KEY_h => OKey::Other('h'),
        keysyms::KEY_j => OKey::Other('j'),
        keysyms::KEY_k => OKey::Other('k'),
        keysyms::KEY_l => OKey::Other('l'),
        keysyms::KEY_z => OKey::Other('z'),
        keysyms::KEY_x => OKey::Other('x'),
        keysyms::KEY_c => OKey::Other('c'),
        keysyms::KEY_v => OKey::Other('v'),
        keysyms::KEY_b => OKey::Other('b'),
        keysyms::KEY_n => OKey::Other('n'),
        keysyms::KEY_m => OKey::Other('m'),
        _ => OKey::Other(' ')
    }
}
fn omodifers_to_xmodifiers(omods: OMods) -> ModMask {
    /* asumption of u16 may be an issue */
    let mut xmods = 0 as u16;
    if omods & OMods::ODILIA == OMods::ODILIA { xmods &= u16::from(ModMask::LOCK) }
    if omods & OMods::SHIFT == OMods::SHIFT { xmods &= u16::from(ModMask::SHIFT) }
    if omods & OMods::CONTROL == OMods::CONTROL { xmods &= u16::from(ModMask::CONTROL) }
    /* is this always true? I think this is probably set by the keyboard layout? */
    if omods & OMods::ALT == OMods::ALT { xmods &= u16::from(ModMask::M1) }
    return ModMask::from(xmods);
}
/* Due to lack of XKeySymToKeyCode or related functions, this is necessary for now.
 * In theory, Xlib does have a function for this, but we'd need to break out FFI wrappers to get to
 * it. TODO: make this cleaner.
 */
fn okey_to_xkeycode(key: OKey) -> u8 {
    return match key {
        OKey::Other('q') => 24,
        OKey::Other('w') => 25,
        OKey::Other('e') => 26,
        OKey::Other('r') => 27,
        OKey::Other('t') => 28,
        OKey::Other('y') => 29,
        OKey::Other('u') => 30,
        OKey::Other('i') => 31,
        OKey::Other('o') => 32,
        OKey::Other('p') => 33,
        OKey::Other('[') => 34,
        OKey::Other(']') => 35,
        OKey::Other('a') => 38,
        OKey::Other('s') => 39,
        OKey::Other('d') => 40,
        OKey::Other('f') => 41,
        OKey::Other('g') => 42,
        OKey::Other('h') => 43,
        OKey::Other('j') => 44,
        OKey::Other('k') => 45,
        OKey::Other('l') => 46,
        OKey::Other(';') => 47,
        OKey::Other('\'') => 48,
        OKey::Other('z') => 52,
        OKey::Other('x') => 53,
        OKey::Other('c') => 54,
        OKey::Other('v') => 55,
        OKey::Other('b') => 56,
        OKey::Other('n') => 57,
        OKey::Other('m') => 58,
        OKey::Other(',') => 59,
        OKey::Other('.') => 60,
        OKey::Other('/') => 61,
        _ => 0
    }
}

#[tokio::main]
async fn main() -> Result<()> {
  // establish a connection to the X11 server
  let mut connection = connect(None).await?;
  // get root window (all windows decend from)
  let root = connection.default_screen().root;
  let _pk = connection.grab_key(
    false, // do NOT report the events normally further down the 
    root, // the grab window (in the case of root, that's for all windows)
    ModMask::LOCK, // only on 
    okey_to_xkeycode(OKey::Other('x')), // stop 
    //okey_to_xkeycode(OKey::Other('x')), // the p key? Into<Grab>
    GrabMode::ASYNC, // pointer mode
    GrabMode::ASYNC // keyboard mode
  ).await;
  let mut converter = KeyboardState::new_async(&mut connection).await?;
  loop {
    let event = connection.wait_for_event().await?;
    match event {
      Event::KeyPress(kpe) => {
        println!("KPE recv.");
        println!("{:?}", kpe);
        let d = converter.symbol_async(
            &mut connection,
            kpe.detail,
            0
        ).await?;
        let o = xkeysym_to_okey(d);
        let om = xkeystate_to_omodifiers(kpe.state);
        println!("{:?}", o);
        println!("{:?}", om);
      }
      Event::Expose(_exe) => {
        println!("EXPOSE!");       
      }
      _ => {
        println!("Event recv.");
      }
    }
  }
  Ok(())
}
