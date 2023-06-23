//! # Evnets
//!
//! The set of events which may come in (usually through key bindings) to execute an action with Odilia.

use serde::{Deserialize, Serialize};

use crate::ScreenReaderMode;
use atspi_common::Role;

#[derive(Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
/// A list of features supported natively by Odilia.
pub enum Feature {
	/// Unimplemented, but will eventually stop all speech until re-activated.
	Speech,
	/// Unimplemented.
	Braille, // TODO
}

/// The direction of an action. This is usually for a structural navigation event.
#[derive(Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
#[serde(tag = "direction")]
pub enum Direction {
	/// Forward (in the tree)
	Forward,
	/// Backward (in the tree)
	Backward,
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "args", rename_all = "camelCase")]
/// Events which can be trigged through Odilia's external API.
/// Subject to change without notice until v1.0, but we're [open to suggestions on our Github](https://github.com/odilia-app/odilia/); please reach out with features you'd like to see.
pub enum ScreenReaderEvent {
	/// when we need to do "something" but this is always hardcoded as nothing
	Noop,
	/// Stop all current speech.
	StopSpeech,
	/// Enable a feature from working.
	Enable(Feature),
	/// Disable a feature.
	Disable(Feature),
	/// Change mode of the screen reader. This is currently global, but it should be per application, and an update should only affect the current application.
	ChangeMode(ScreenReaderMode),
  /// Focus on a new item which is in the direction [`Direction`] and a role of [`Role`].
	StructuralNavigation(Direction, Role),
}

