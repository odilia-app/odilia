use crate::{elements::ElementType, modes::ScreenReaderMode};
use speech_dispatcher::Priority;

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum Feature {
    Speech,
    Braille, // TODO
    Navigation, // TODO
}

#[derive(Eq,PartialEq,Clone,Hash)]
pub enum ScreenReaderEvent {
    Noop,
    Speak(String, u32),
    StopSpeech,
    Enable(Feature),
    Disable(Feature),
    ChangeMode(ScreenReaderMode),
    Next(ElementType),
    Previous(ElementType),
}
