use crate::modes::ScreenReaderMode;
use atspi::accessible::Role;

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum Feature {
    Speech,
    Braille, // TODO
}

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum ScreenReaderEvent {
    Noop, // when we need to do "something" but this is alwyas hardcoded as nothing
    Speak(String, u32),
    StopSpeech,
    Enable(Feature),
    Disable(Feature),
    ChangeMode(ScreenReaderMode),
    StructuralNavigation(Direction, Role),
}
