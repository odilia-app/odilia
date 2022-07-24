use crate::{elements::ElementType, modes::ScreenReaderMode};

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum ScreenReaderEventType {
    ChangeMode(ScreenReaderMode),
    Next(ElementType),
    Previous(ElementType),
}
