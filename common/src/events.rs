use serde::{
  Serialize,
  Deserialize,
};

use atspi::{
  accessible::Role,
  text::TextGranularity,
};
use crate::modes::ScreenReaderMode;

#[derive(Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub enum Feature {
    Speech,
    Braille, // TODO
}

#[derive(Eq,PartialEq,Clone,Hash,Serialize,Deserialize)]
#[serde(tag="direction")]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Eq,PartialEq,Clone,Hash,Serialize,Deserialize)]
#[serde(tag="event", content="args", rename_all="camelCase")]
pub enum ScreenReaderEvent {
    Noop, // when we need to do "something" but this is always hardcoded as nothing
    Speak(String, u32),
    StopSpeech,
    Enable(Feature),
    Disable(Feature),
    ChangeMode(ScreenReaderMode),
    ChangeGranularity(TextGranularity),
    StructuralNavigation(Direction, Role),
}
