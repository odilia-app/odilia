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
//! 3. state -> mutable state view (this is generic for every Odilia command, and does not require the actual structure to be passed)
//! 4. Odilia command + mutable state view -> execute the command
//!
//! But you can not await any future within this block.
//! The execute function may internally *spawn* some async futures, but it will not await them directly.
//! The runtime will await them.

use crate::state::ScreenReaderState;

use odilia_common::{errors::OdiliaError, events::ScreenReaderEvent, commands::{OdiliaCommand, CacheCommand}, state::OdiliaState};

/// Implemented by any type which executes a speciic, defined action and modifies state.
pub trait Command: MutableStateView {
	/// Execute the specific state modification defined for this type.
	/// Some guidance on writing this function:
	///
	/// 1. If you *must* query state before modifying it, use two separate locks: one read lock, then query any data required, manually [`drop`] the lock, then aquire a write lock to modify.
	/// It is heavily prefered that state querying happens outside this function, if it all possible, then embedd that data in the structure this trait is being applied to.
	/// 2. Always manually [`drop`] any locks aquired *on the next line* after use. This does not make the runtime code more effecient (the Rust compiler can figure it out), but it will help you catch mistakes like holding multiple write locks at the same time, or accidentally adding another write somewhere else in the function.
	/// 3. Please use trace-level logging. If the modification is completely, or partially successful, why it failed, etc. is all useful information for the logs. But this should be reserved for the most detailed of the log output.
	fn execute(&self, mutable_state: <Self as MutableStateView>::MutableState) -> Result<(), OdiliaError>;
}

impl Command for OdiliaCommand {
	fn execute(&self) -> Result<(), OdiliaError> {
		match self {
			OdiliaCommand::Cache(cache_command) => cache_command.execute(),
			_ => todo!()
		}
	}
}
impl Command for CacheCommand {
	fn execute(&self) -> Result<(), OdiliaError> {
		match self {
			CacheCommand::SetText(set_text_command) => set_text_command.execute(),
		}
	}
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
/// If you need more information from the state before creating these state-modifcation commands, then implement additional fields in the type you are implementing this trait on.
/// This is usually a type returned from [`IntoStateProduct::get_state_pieces`].
/// 
/// NOTE: This can not be done using `impl Into<Vec<ScreenReaderEvent>> for &T` because Odilia may implement this functionality for foreign types (for example, those in [`atspi_common::events`].
/// Events are guarenteed to be executed in the order they are recieved in the vector.
pub trait IntoOdiliaCommands {

	/// Create a list of commands to run against Odilia's current state.
	fn commands(&self) -> Result<Vec<OdiliaCommand>, OdiliaError>;
}

/// Set the type which has direct, mutable access to Odilia's state.
/// This is an associated type, usually implmented on specific [`Command`]s.
/// This type should either be a type from, or a structure composed from types defined in [`crate::state::types`], or [`CacheItem`].
pub trait MutableStateView {
	/// This type must be bale to be sync across threads safely *using clone*.
	type MutableView: Send + Sync + Clone;
}
