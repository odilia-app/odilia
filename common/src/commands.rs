//! # Commands
//! 
//! Commands are specifc, simple items that modify a portion of Odilia's state.
//! The implementation of these commands is in Odilia.

use crate::cache::CacheKey;
use serde::{Serialize, Deserialize};

/// Internal commands to modify the state of the screen reader and/or perform external actions.
/// These differ froma [`crate::events::OdiliaEvent`] in that they are directly related to Odilia's implementation
/// of various actions.
///
/// For example: A [`crate::events::OdiliaEvent::StructuralNavigation`] event, which requires lookups all over the cache, Odilia will convert that event into a direct command: [`MoveFocus`], which actively moves the user's focus to a new location.
/// However, an [`atspi_common::events::object::StateChanged`] event, with its `enabled` field set to 1, and its `state` field set to [`atspi_common::state::State::Focused`], this would produce a [`ChangeFocus`] command here, which updates Odilia's internal pointer to the focused item, but does not actively move the user's focus.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub enum OdiliaCommand {
	/// All commands related to updating the cache.
	Cache(CacheCommand),
	/// Move the *USER*'s focus to a new object.
	MoveFocus(MoveFocusCommand),
	/// Setting *ODILIA*'s internal focused object.
	UpdateFocus(UpdateFocusCommand),
	/// Move the *USER*'s caret to a new position.
	MoveCaretPosition(MoveCaretPositionCommand),
	/// Setting *ODILIA*'s internal caret position.
	UpdateCaretPosition(UpdateCaretPositionCommand),
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
/// Update the user's caret position.
/// NOTE: This does *NOT* set Odilia's cursor position.
pub struct MoveCaretPositionCommand {
	/// The new caret position.
	new_position: i32
}

/// Update Odilia's internal caret position.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct UpdateCaretPositionCommand {
	/// The new caret position.
	new_position: i32
}

/// Any command that directly changes items in the cache.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub enum CacheCommand {
	/// Set the text of a given cache item.
	SetText(SetTextCommand),
}

/// Set new text contents for a cache item.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct SetTextCommand {
	/// The new text to set.
	pub new_text: String,
	/// Which item will the new text be applied to.
	/// This will need to be turned into a mutable [`CacheValue`] by the host.
	pub apply_to: CacheKey,
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

