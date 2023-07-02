//! State management for Odilia.
//! This has a bunch of smaller structures for handling the minimum state necessary to produce a command.
//! Please see the information on the Odilia architecture in the `README.md`.

use crate::cache::{CacheKey, ExternalCacheItem};
use crate::traits::StateView;
use atspi_common::events::{
	object::{StateChangedEvent, TextCaretMovedEvent, TextChangedEvent, ChildrenChangedEvent},
	AddAccessibleEvent, RemoveAccessibleEvent,
};

use serde::{Deserialize, Serialize};

macro_rules! impl_state_view {
	($type:ty, $state_view:ty) => {
		impl StateView for $type {
			type View = $state_view;
		}
	};
}

/// View for a caret position change event.
#[derive(Serialize, Deserialize, Clone)]
pub struct CaretPositionView {
	/// The previous position of the curosr.
	previous_position: i32,
	/// The previously focused item.
	previous_focus: CacheKey,
}
impl_state_view!(TextCaretMovedEvent, CaretPositionView);

impl_state_view!(TextChangedEvent, ExternalCacheItem);
impl_state_view!(StateChangedEvent, ExternalCacheItem);
impl_state_view!(ChildrenChangedEvent, ());
impl_state_view!(AddAccessibleEvent, ());
impl_state_view!(RemoveAccessibleEvent, ());
