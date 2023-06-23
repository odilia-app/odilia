use serde::{Serialize, Deserialize};

/// Set a state view for a given type.
/// The actual conversion from state to this type is (necessarily) implemented by the Odilia host ([`odilia`]), since the state type can not be directly sent across FFI boundaries.
/// The point is that the map between the event type and the state view is defined.
pub trait StateView {
	/// The type which is defined as a state's view for a given type.
	/// This must be able to be sent across any generic IPC system.
	/// Therefore, it must implement [`Serialize`], and [`Deserialie`].
	/// It must also implement [`Clone`], since the type are going to be sent to multiple plugins, or other IPC mechanisms, not just one.
	type View: Serialize + for <'a> Deserialize<'a> + Clone;
}

