//! Traits to separate concerns of:
//!
//! 1. Odilia's state management
//! 2. Logic
//! 3. Incomming events
//!
//! You should genrally be following this dataflow:
//! 
//! 1. event + state -> read-only state view
//! 2. state view + logic -> internal Odilia commands
//! 3. Odilia command + state -> modifyable state view
//!		* This step is needed usually for a cache lookup.
//! 4. Odilia command + modifyable state view -> execute the command
//!
//! But you can not await any future within this block.
//! The execute function may internally *spawn* some async futures, but it will not await them directly.
//! The runtime will await them.
//!
//! Plugins will be able (soon) to hook into nearly every step of the process.
//! The only two parts that plugins can not modify are step 3 and step 4.
//! Since these sections contain active references to state that must be managed internally.
//!
//! If you require an operation to the state which is not implemented by any of the commands in [`odilia_common::commands`], then please open an issue.
//! We aim to support any reasonable request here.
//! If you have a more complex action that you would like to be supported, we would suggest breaking it down into individual steps via the existing command types.

use crate::state::ScreenReaderState;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use odilia_common::{errors::OdiliaError, events::ScreenReaderEvent, commands::{OdiliaCommand, CacheCommand}};
pub use odilia_common::traits::StateView;

/// Implemented by any type which executes a speciic, defined action and modifies state.
///
/// Commands are meant to be small, easy to predict operations.
/// If you need to query the cache to complete the command, consider moving that logic into [`MutableStateView::create_view`].
#[async_trait]
pub trait Command: MutableStateView + IntoMutableStateView {
	/// Execute the specific state modification defined for this type.
	/// Some guidance on writing this function:
	///
	/// 1. If you *must* query state before modifying it, use two separate locks: one read lock, then query any data required, manually [`drop`] the lock, then aquire a write lock to modify.
	/// It is heavily prefered that state querying happens outside this function, if it all possible, then embedd that data in the 
	/// 2. Always manually [`drop`] any locks aquired *on the next line* after use. This does not make the runtime code more effecient (the Rust compiler can figure it out), but it will help you catch mistakes like holding multiple write locks at the same time, or accidentally adding another write somewhere else in the function.
	/// 3. Please use trace-level logging. If the modification is completely, or partially successful, why it failed, etc. is all useful information for the logs. But this should be reserved for the most detailed of the log output.
	/// 4. If you need to do multiple operations, it is very likely that they should be two separate commands.
	/// For example, the AT-SPI event [`atspi_common::events::object::ChildrenChanged`] is a single event, but it contains at least two atomic operations.
	/// 
	/// 		1. Remove/insert new item into cache.
	/// 		2. Update the parent's reference to its children.
	///
	/// 	To keep Odilia simple (for testing and reproducability), these operations should be separated, even if they may be executed one-after-the-other or are thought of as a singular change.
	/// 5. If the function is longer than 5-6 lines, you may want to reconsider your design.
	/// The [`execute`] function is meant to perform very small, atomic operations,
	/// and all logic should mostly be within the [`IntoOdiiaCommands`] implementation of any given event.
	async fn execute(&self, mutable_state: <Self as MutableStateView>::View) -> Result<(), OdiliaError>;
}

/// Implemented for any type which would like to be able to 
/// convert into a list of OdiliaEvents.
///
/// These can then be used by Odilia to modify its internal state, update the cache, speak text, change language, etc.
/// Note that this expects that you will *not* consume the event.
/// So you may need to copy strings, if they are used in the data structure.
/// 
/// This fits Odilia's overall architechture as in `README.md`;
/// This code is only able to *create instructions* for modifying the state, but it may not modify the state directly.
/// 
/// NOTE: This can not be done using `impl Into<Vec<ScreenReaderEvent>> for &T` because Odilia may implement this functionality for foreign types (for example, those in [`atspi_common::events`].
/// Events are guarenteed to be executed in the order they are recieved in the vector.
#[async_trait]
pub trait IntoOdiliaCommands: StateView {

	/// Create a list of commands to run against Odilia's current state.
	async fn commands(&self, state_view: &<Self as StateView>::View) -> Result<Vec<OdiliaCommand>, OdiliaError>;
}

/// Indicates that a mutable state view can be created from this structure.
#[async_trait]
pub trait IntoStateView: StateView {
	/// Create read-only state view from a combination the implemented type, and the entire state.
	/// You may query the state as much as you'd like.
	/// Just don't write to it.
	async fn create_view(&self, state: &ScreenReaderState) -> Result<<Self as StateView>::View, OdiliaError>;
}

/// Set a mutable state view for the current type.
pub trait MutableStateView {
	/// The view type; the item which will be modified within the state.
	/// This should be a singular type created from one of the following types:
	///
	/// * Any type defined in [`crate::state::types`].
	/// * [`CacheItem`].
	///
	/// If you need to create a new structure for this type, you may want to reconsider your implementaion.
	/// For example, suppose you need access to two items from the cache, then the [`Command::execute`] function handles updating different pieces of both items' state.
	/// This would violate the general rule of thumb in Odilia, which is: odilia commands should only modify one piece of state at a time.
	/// In the case where you do need to modify two piece of state, and you *think* you need a structure for this type, please implement to [`IntoDoiliaCommands`] trait, and return more than one command.
	/// Then, have those commands perform two separate operations.
	type View: Send + Sync;
}

/// Indicates that a mutable state view can be created from this structure.
#[async_trait]
pub trait IntoMutableStateView: MutableStateView {
	/// Create a mutable state view from a reference to both the implenting type, and a full version of the state.
	/// Note that this function should not directly write to the state, but rather, use this opportunity to build the set of items that need to be modified (usually for a given [`Command`].
	/// When a [`CacheKey`] is part of the implementing type, feel free to lookup the item in the cache, then use a [`CacheValue`] as a field in your return type.
	async fn create_view(&self, state: &ScreenReaderState) -> Result<<Self as MutableStateView>::View, OdiliaError>;
}
