use async_trait::async_trait;
use crate::state::ScreenReaderState;

use odilia_common::{errors::OdiliaError, events::ScreenReaderEvent};

/// Implemented by any type which executes a speciic, defined action and modifies state.
/// This is implemented as a core *internal* feature of all types contained within the [`odilia_common::events::ScreenReaderEvents`] enum.
#[async_trait]
pub trait StateModification {
	/// Execute the specific state modification defined for this type.
	/// Some guidance on writing this function:
	///
	/// 1. If you *must* query state before modifying it, use two separate locks: one read lock, then query any data required, manually [`drop`] the lock, then aquire a write lock to modify.
	/// It is heavily prefered that state querying happens outside this function, if it all possible, then embedd that data in the structure this trait is being applied to.
	/// 2. Always manually [`drop`] any locks aquired *on the next line* after use. This does not make the runtime code more effecient (the Rust compiler can figure it out), but it will help you catch mistakes like holding multiple write locks at the same time, or accidentally adding another write somewhere else in the function.
	/// 3. You *may* call asynchronous functions within your implementation, but you *CAN NOT* do so while holding a lock. You may not hold any kind of lock (read, write or mutex lock) when awaiting an asyncronous function. This [can not yet be detected by the compiler](https://github.com/rust-lang/rust/issues/83310), and needs to be enforced by the developer.
	/// 4. Avoid calling functions that can panic while holding a lock. This causes something called a "poisoning", which, as I'm sure you can imagine, is not good. This makes code very hard to test and recover from; in many cases, Odilia will crash when a poisoning occurs, since there is not much we can do at that point to ensure that the data we have is correct.
	/// 5. Please use trace-level logging. If the modification is completely, or partially successful, why it failed, etc. is all useful information for the logs. But this should be reserved for the most detailed of the log output.
	async fn execute(&self) -> Result<(), OdiliaError>;
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
	fn commands(&self) -> Vec<ScreenReaderEvent>;
}

/// This trait is meant to minimize the level of access to the state to be relatively granular.
/// In particular, access to the cache, although sometimes necessary, should not be locked behind a full mutable borrow of the entire state structure.
/// This fits Odilia's overall architechture as described in the architechture section of the `REAMDE.md`.
/// Basically, this is code which should only read state.
pub trait IntoStateProduct {
	/// "Product" here referes to a type which holds multiple other types, so in this case, an event and the state variables it will modify.
	/// Any structure which contains pieaces of state should only contain types which can be sent across threads safely and (relatively) trivially.
	/// Types contained within this structure should usually be associated types of [`crate::state`].
	/// But other types are allowed if they can be constructed from the state.
	/// It should also include the event data, like so:
	///
	/// ```rust
	/// # use odilia_common::{events::TextRemovedEvent, cache::CacheRef};
	/// struct TextChangedStateModifier {
	///     event: TextRemovedEvent,
	///     apply_to: CacheRef,
	///     // add more as necessary
	/// }
	/// ```
	///
	/// This type must also implement [`StateModification`], since any type returned from [`create`] should be able to be passed to [`StateModification::execute`].
	type ProductType: Send + Sync + StateModification;

	/// Using both the event and state, construct the necessary type to complete a set of actions on Odilia.
	/// 
	/// 1. You *MAY NOT* aquire a write lock within this function; only copy references necessary to do so within the body of [`StateModification::execute()`]. This can not be enforced by the compiler, only developers.
	/// 2. You *should* call *synchronous* function on various state items to *read* from them.
	/// This may be useful if you want to, for example, query for an item in the cache as that can be directly modified without locking the cache later.
	/// If you want to do this, consider using an [`odilia_common::cache::CacheRef`], since this adds some nice convenience features like being able to reference the cache item by ID or by direct reference.
	fn create(&self, state: &ScreenReaderState) -> Result<Self::ProductType, OdiliaError>;
}
