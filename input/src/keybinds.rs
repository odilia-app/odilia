use odilia_common::{
  events::{
      ScreenReaderEvent,
  },
  input::{
    KeyBinding,
    KeyEvent,
    Modifiers,
  },
  modes::{
    ScreenReaderMode,
  },
};
use tokio::{
  sync::Mutex,
};
use std::{
  future::Future,
  collections::HashMap,
};

lazy_static! {
  static ref KB_MAP: Mutex<HashMap<KeyBinding, ScreenReaderEvent>> = Mutex::new(HashMap::new());
  static ref SR_MODE: Mutex<ScreenReaderMode> = Mutex::new(ScreenReaderMode::new("CommandMoode"));
}

pub type AsyncFn = Box<dyn Fn() -> Box<dyn Future<Output = ()> + Unpin + Send + 'static> + Send + Sync + 'static>;

pub async fn add_keybind(kb: KeyBinding, ev: ScreenReaderEvent) -> bool 
{
  /* WTF? Why can't I check if it didn't workk? I guess tokio mutexes are better somehow? */
  let mut kbhm = KB_MAP.lock().await;
  kbhm.insert(kb, ev);
  true
}

pub async fn remove_keybind(kb: KeyBinding) -> bool {
  let mut kbhm = KB_MAP.lock().await;
  kbhm.remove(&kb);
  true
}

pub async fn keyevent_match(kbm: &KeyEvent) -> Option<KeyBinding>
{
  let kbhm = KB_MAP.lock().await;
  let sr_mode = get_sr_mode().await;
  for (kb, _) in kbhm.iter() {
    let mut matches = true;
    matches &= kb.key == kbm.key;
    matches &= kb.repeat == kbm.repeat;
    matches &= (kb.mods == Modifiers::NONE && kbm.mods == Modifiers::NONE) || kb.mods.intersects(kbm.mods);
    if let Some(mode) = &kb.mode {
      matches &= *mode == sr_mode;
    }
    if matches {
      return Some(kb.clone());
    }
  }
  None
}

/* this will match with the bitflags */
pub fn keyevent_match_sync(kbm: &KeyEvent) -> Option<(KeyBinding, ScreenReaderEvent)>
{
  let kbhm = KB_MAP.blocking_lock();
  let sr_mode = get_sr_mode_sync();
  for (kb, sr_event) in kbhm.iter() {
    let mut matches = true;
    matches &= kb.key == kbm.key;
    matches &= kb.repeat == kbm.repeat;
    matches &= (kb.mods == Modifiers::NONE && kbm.mods == Modifiers::NONE) || kb.mods.intersects(kbm.mods);
    if let Some(mode) = &kb.mode {
      matches &= *mode == sr_mode;
    }
    if matches {
      return Some((kb.clone(), sr_event.clone()));
    }
  }
  None
} 

pub fn get_sr_mode_sync() -> ScreenReaderMode {
  SR_MODE.blocking_lock().clone()
}
pub fn set_sr_mode_sync(srm: ScreenReaderMode) { 
  let mut sr_mode = SR_MODE.blocking_lock();
  *sr_mode = srm;
}
pub async fn get_sr_mode() -> ScreenReaderMode {
  SR_MODE.lock().await.clone()
}
pub async fn set_sr_mode(srm: ScreenReaderMode) {
  let mut sr_mode = SR_MODE.lock().await;
  *sr_mode = srm;
}
