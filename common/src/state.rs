//! State management for Odilia.
//! This has a bunch of smaller structures for handling the minimum state necessary to produce a command.
//! Please see the information on the Odilia architecture in the `README.md`.

use crate::cache::CacheRef;
use crate::errors::OdiliaError;

use serde::{Serialize, Deserialize};

macro_rules! impl_from_state {
	($type:ty, $inner_type:ty, $inner_path:path, $outer_type:ty, $outer_path:path, $root_path:path) => {
		impl From<$type> for $inner_type {
			fn from(ty: $type) -> $inner_type {
				$inner_path(ty)
			}
		}
		impl From<$type> for $outer_type {
			fn from(ty: $type) -> $outer_type {
				$outer_path($inner_path(ty))
			}
		}
		impl From<$type> for OdiliaState {
			fn from(ty: $type) -> OdiliaState {
				$root_path($outer_path($inner_path(ty)))
			}
		}
		impl TryFrom<OdiliaState> for $type {
			type Error = OdiliaError;
			fn try_from(state: OdiliaState) -> Result<$type, Self::Error> {
				if let $root_path($outer_path($inner_path(inner_event))) = state {
					Ok(inner_event)
				} else {
					Err(OdiliaError::InvalidStateVariant)
				}
			}
		}
	}
}
macro_rules! impl_conv {
	($inner_type:ty, $outer_type:ty, $conv_path:path) => {
		impl TryFrom<$outer_type> for $inner_type {
			type Error = OdiliaError;
			fn try_from(state: $outer_type) -> Result<$inner_type, Self::Error> {
				if let $conv_path(inner_event) = state {
					Ok(inner_event)
				} else {
					Err(OdiliaError::InvalidStateVariant)
				}
			}
		}
		impl From<$inner_type> for $outer_type {
			fn from(inner_event: $inner_type) -> $outer_type {
				$conv_path(inner_event)
			}
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
/// An enum containing all state structures.
pub enum OdiliaState {
	Cache(CacheState),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
/// All possible state structures related to cache modification.
pub enum CacheState {
	Text(TextState),
}
impl_conv!(CacheState, OdiliaState, OdiliaState::Cache);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
/// All possible state structures related to modifying text.
pub enum TextState {
	Insert(TextInsertState),
	Delete(TextDeleteState),
}
impl_conv!(TextState, CacheState, CacheState::Text);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Default)]
/// To insert text into a CacheRef's text field.
pub struct TextInsertState {
	pub start_index: usize,
	pub text: String,
	pub apply_to: CacheRef,
}
impl_from_state!(TextInsertState, TextState, TextState::Insert, CacheState, CacheState::Text, OdiliaState::Cache);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Default)]
/// To delete text from a [`CacheRef`], only the indexes of the deletion are required.
pub struct TextDeleteState {
	pub start_index: usize,
	pub end_index: usize,
	pub apply_to: CacheRef,
}
impl_from_state!(TextDeleteState, TextState, TextState::Delete, CacheState, CacheState::Text, OdiliaState::Cache);

