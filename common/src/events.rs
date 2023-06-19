use serde::{Deserialize, Serialize};

use crate::{
	errors::OdiliaError,
  modes::ScreenReaderMode,
  types::Accessible,
};
use atspi_common::{Role, State};

#[derive(Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
/// A list of features supported natively by Odilia.
pub enum Feature {
	/// Unimplemented, but will eventually stop all speech until re-activated.
	Speech,
	/// Unimplemented.
	Braille, // TODO
}

#[derive(Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
#[serde(tag = "direction")]
pub enum Direction {
	Forward,
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
  /// Cache modification events.
  Cache(CacheEvent),
  /// This is used to update the internal history of events.
  /// This is sometimes referenced to find combinations of events.
  UpdateHistory(atspi_common::events::Event),
  /// Speaking text.
  Speak(String),
  /// Move the caret to a specific accessible object at a specific index.
  /// Note that this does not affect the cache, but merely the state of Odilia.
  /// Often, you need to check where the cursor *is* before setting it to the new place.
  /// For example, when a curosr is moved within the same accessible object, the screen reader may want to speak what was between the old and new curosr position.
  MoveCaret(Accessible, i32),
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum CacheEvent {
  /// Load all items underneath a root into the cache.
  /// Internally, this is an extremely expensive event to process. We do not recommend triggering this often, since it can cause quite the performance hit.
  LoadAll(Accessible),
  /// Remove a state from an accessible.
  RemoveState(Accessible, State),
  /// Add a state to an accessible.
  AddState(Accessible, State),
  /// Change text in an accessible.
  TextChanged(TextChangedEvent),
  /// Remove an accessible object from the cache. This event should usually only be triggered internally.
  /// When attempting to query information about an accessible object which is not contained within the cache will result in a very chatty conversation with `atspi` over DBus.
  RemoveItem(Accessible),
  /// This queries zbus for all the information required to add a new cache item.
  /// It can take quite some time, so if you're a developer of Odilia itself, please make sure this always runs in a separate asyncronous task.
  // this should eventually contains an odilia_cache::CacheItem; this adds a bunch of information that would not need to be queried.
  AddItem(Accessible),
}

#[derive(Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub enum Operation {
	#[serde(alias = "insert")]
	#[serde(alias = "insert/system")]
	Insert,
	#[serde(alias = "delete")]
	#[serde(alias = "delete/system")]
	Delete,
}
impl TryFrom<&str> for Operation {
	type Error = OdiliaError;

	fn try_from(s: &str) -> Result<Operation, Self::Error> {
		match s {
			"insert" | "insert/system" => Ok(Operation::Insert),
			"delete" | "delete/system" => Ok(Operation::Delete),
			_ => Err(OdiliaError::ParseError(format!("Converting to an Operation type was unsuccessful because the string \"{s}\" did not match an appropriate pattern")))
		}
	}
}

#[derive(Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct TextChangedEvent {
	pub operation: Operation,
  pub item: Accessible,
  pub start_index: i32,
  pub length: i32,
  pub text: String,
}
