//! State management for Odilia.
//! This has a bunch of smaller structures for handling the minimum state necessary to produce a command.
//! Please see the information on the Odilia architecture in the `README.md`.

use crate::cache::{CacheItem, CacheKey};
use crate::traits::StateView;
use atspi_common::events::{
	object::{
		TextCaretMovedEvent,
		TextChangedEvent,
	},
};

use serde::{Serialize, Deserialize};

macro_rules! impl_state_view {
	($type:ty, $state_view:ty) => {
		impl StateView for $type {
			type View = $state_view;
		}
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CaretPositionView {
	previous_position: i32,
	previous_focus: CacheKey,
}
impl_state_view!(TextCaretMovedEvent, CaretPositionView);

impl_state_view!(TextChangedEvent, CacheItem);
