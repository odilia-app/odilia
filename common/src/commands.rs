use crate::cache::CacheRef;
use serde::{Serialize, Deserialize};

/// Internal commands to modify the state of the screen reader and/or perform external actions.
/// These differ froma [`crate::events::OdiliaEvent`] in that they are directly related to Odilia's implementation
/// of various actions.
///
/// For example: A [`crate::events::OdiliaEvent::StructuralNavigation`] event, which requires lookups all over the cache, Odilia will convert that event into a direct command: [`MoveFocus`], which actively moves the user's focus to a new location.
/// However, an [`atspi_common::events::object::StateChanged`] event, with its `enabled` field set to 1, and its `state` field set to [`atspi_common::state::State::Focused`], this would produce a [`ChangeFocus`] command here, which updates Odilia's internal pointer to the focused item, but does not actively move the user's focus.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub enum OdiliaCommand {
	Cache(CacheCommand),
	MoveFocus(MoveFocusCommand),
	UpdateFocus(UpdateFocusCommand),
	MoveCaretPosition(MoveCaretPositionCommand),
	UpdateCaretPosition(UpdateCaretPositionCommand),
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct MoveCaretPositionCommand {
	new_position: i32
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct UpdateCaretPositionCommand {
	new_position: i32
}

/// Any command that directly changes items in the cache.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub enum CacheCommand {
	SetText(SetTextCommand),
}

/// Set new text contents for a cache item.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct SetTextCommand {
	pub new_text: String,
	pub apply_to: CacheRef,
}
impl From<SetTextCommand> for CacheCommand {
	fn from(stc: SetTextCommand) -> CacheCommand {
		CacheCommand::SetText(stc)
	}
}
impl From<CacheCommand> for OdiliaCommand {
	fn from(ccs: CacheCommand) -> OdiliaCommand {
		OdiliaCommand::Cache(ccs)
	}
}
impl From<SetTextCommand> for OdiliaCommand {
	fn from(stc: SetTextCommand) -> OdiliaCommand {
		OdiliaCommand::Cache(stc.into())
	}
}

/// Update Odilia's pointer as to where the current focus is.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct UpdateFocusCommand {
}

/// Move the user's focus to a new location.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct MoveFocusCommand {
}

