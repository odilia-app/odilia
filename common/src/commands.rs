//! # Commands
//!
//! Commands are specifc, simple items that modify a portion of Odilia's state.
//! The implementation of these commands is in Odilia.

use crate::{cache::CacheKey, errors::OdiliaError};
use atspi_common::StateSet;
use serde::{Deserialize, Serialize};

macro_rules! impl_conversions {
	($inner_type:ty, $inner_path:path, $outer_type:ty, $outer_path:path) => {
		impl From<$inner_type> for $outer_type {
			fn from(inner: $inner_type) -> $outer_type {
				$inner_path(inner)
			}
		}
		impl From<$inner_type> for OdiliaCommand {
			fn from(inner: $inner_type) -> OdiliaCommand {
				$outer_path($inner_path(inner))
			}
		}
		impl TryFrom<OdiliaCommand> for $inner_type {
			type Error = OdiliaError;

			fn try_from(command: OdiliaCommand) -> Result<$inner_type, Self::Error> {
				if let $outer_path($inner_path(specific_command)) = command {
					Ok(specific_command)
				} else {
					Err(OdiliaError::InvalidVariant(format!("Invalid variant of OdiliaCommand. Type wanted: SetTextCommand. Type contained: {command:?}")))
				}
			}
		}
	}
}

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
	new_position: i32,
}

/// Update Odilia's internal caret position.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct UpdateCaretPositionCommand {
	/// The new caret position.
	new_position: i32,
}

/// Any command that directly changes items in the cache.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub enum CacheCommand {
	/// Set the text of a given cache item.
	SetText(SetTextCommand),
	/// Set the state of a given cache item.
	SetState(SetStateCommand),
	/// Adds/removes a child from a given cache item.
	ChangeChild(ChangeChildCommand),
}

/// Adds a new child reference to a cache item.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct ChangeChildCommand {
	/// An ID of the new child to add.
	pub new_child: CacheKey,
	/// The index of the new child in the parent.
	pub index: usize,
	/// Should the child be added (or removed).
	pub add: bool,
	/// Which item will the command will be applied to.
	/// This will need to be turned into a mutable [`CacheValue`] by the host.
	pub apply_to: CacheKey,
}
impl_conversions!(ChangeChildCommand, CacheCommand::ChangeChild, CacheCommand, OdiliaCommand::Cache);

/// Set new text contents for a cache item.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct SetTextCommand {
	/// The new text to set.
	pub new_text: String,
	/// Which item will the new text be applied to.
	/// This will need to be turned into a mutable [`CacheValue`] by the host.
	pub apply_to: CacheKey,
}
impl_conversions!(SetTextCommand, CacheCommand::SetText, CacheCommand, OdiliaCommand::Cache);
/// Set new state for a cache item.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct SetStateCommand {
	/// The new state set to use.
	pub new_states: StateSet,
	/// Which item will the new text be applied to.
	/// This will need to be turned into a mutable [`CacheValue`] by the host.
	pub apply_to: CacheKey,
}
impl_conversions!(SetStateCommand, CacheCommand::SetState, CacheCommand, OdiliaCommand::Cache);

/// Update Odilia's pointer as to where the current focus is.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct UpdateFocusCommand {}

/// Move the user's focus to a new location.
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct MoveFocusCommand {}
