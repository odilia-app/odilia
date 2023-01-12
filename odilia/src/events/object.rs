use crate::state::ScreenReaderState;
use atspi::{
	identify::{
		object::ObjectEvents,
	},
};

pub async fn dispatch(state: &ScreenReaderState, event: &ObjectEvents) -> eyre::Result<()> {
	// Dispatch based on member
	match event {
		ObjectEvents::StateChanged(state_changed_event) => state_changed::dispatch(state, state_changed_event).await?,
		ObjectEvents::TextCaretMoved(text_caret_moved_event) => text_caret_moved::dispatch(state, text_caret_moved_event).await?,
		ObjectEvents::ChildrenChanged(children_changed_event) => children_changed::dispatch(state, children_changed_event).await?,
		ObjectEvents::TextChanged(text_changed_event) => text_changed::dispatch(state, text_changed_event).await?,
		other_member => tracing::debug!("Ignoring event with unknown member: {:#?}", other_member),
	}
	Ok(())
}

mod text_changed {
	use crate::state::ScreenReaderState;
	use atspi::{
		events::GenericEvent,
		identify::object::TextChangedEvent,
		signify::Signified
	};
	pub async fn dispatch(state: &ScreenReaderState, event: &TextChangedEvent) -> zbus::Result<()> {
		match event.kind() {
			"remove" => remove(state, event).await?,
			"remove/system" => remove(state, event).await?,
			"add" => add(state, event).await?,
			"add/system" => add(state, event).await?,
		}
		Ok(())
	}
	pub async fn add(state: &ScreenReaderState, event: &TextChangedEvent) -> zbus::Result<()> {
		Ok(())
	}
	pub async fn remove(state: &ScreenReaderState, event: &TextChangedEvent) -> zbus::Result<()> {
		Ok(())
	}
}

mod children_changed {
	use crate::state::ScreenReaderState;
	use atspi::{events::GenericEvent, identify::{object::ChildrenChangedEvent}, signify::Signified};

	pub async fn dispatch(state: &ScreenReaderState, event: &ChildrenChangedEvent) -> eyre::Result<()> {
		// Dispatch based on kind
		match event.kind() {
			"remove/system" => remove(state, event).await?,
			"remove" => remove(state, event).await?,
			"add/system" => add(state, event).await?,
			"add" => add(state, event).await?,
			kind => tracing::debug!(kind, "Ignoring event with unknown kind"),
		}
		Ok(())
	}
	pub async fn add(state: &ScreenReaderState, event: &ChildrenChangedEvent) -> eyre::Result<()> {
		let accessible = state.new_accessible(event).await?;
		let _ = state.cache.get_or_create(&accessible).await;
		tracing::debug!("Add a single item to cache.");
		Ok(())
	}
	pub async fn remove(state: &ScreenReaderState, event: &ChildrenChangedEvent) -> eyre::Result<()> {
		let path = event.path().expect("All accessibles must have a path").try_into()?;
		state.cache.remove(&path).await;
		tracing::debug!("Remove a single item from cache.");
		Ok(())
	}
}

mod text_caret_moved {
	use crate::state::ScreenReaderState;
	use atspi::{convertable::Convertable, identify::object::TextCaretMovedEvent, signify::Signified};
	use ssip_client::Priority;

	/// this must be checked *before* writing an accessible to the hsitory.
	/// if this is checked after writing, it may give inaccurate results.
	/// that said, this is a *guess* and not a guarentee.
	/// TODO: make this a testable function, anything which queries "state" is not testable
	async fn is_tab_navigation(state: &ScreenReaderState, event: &TextCaretMovedEvent, string_len: i32) -> eyre::Result<bool> {
		let current_caret_pos = event.position();
		// if the carat position is not at 0, we know that it is not a tab navigation, this is because tab will automatically set the cursor position at 0.
		if current_caret_pos != 0 && current_caret_pos != string_len {
			return Ok(false);
		}
		// Hopefully this shouldn't happen, but technically the caret may change before any other event happens. Since we already know that the caret position is 0, it may be a caret moved event
		let last_accessible = match state.history_item(0).await? {
			Some(acc) => acc,
			None => return Ok(false),
		};
		// likewise when getting the second-most recently focused accessible; we need the second-most recent accessible because it is possible that a tab navigation happened, which focused something before (or after) the caret moved events gets called, meaning the second-most recent accessible may be the only different accessible.
		// if the accessible is focused before the event happens, the last_accessible variable will be the same as current_accessible.
		// if the accessible is focused after the event happens, then the last_accessible will be different
		let previous_caret_pos = state.previous_caret_position.get();
		let current_accessible = state.new_accessible(event).await?;
		// if we know that the previous caret position was not 0, and the current and previous accessibles are the same, we know that this is NOT a tab navigation.
		if previous_caret_pos != 0 &&
			current_accessible == last_accessible {
			return Ok(false);
		}
		// otherwise, it probably was a tab navigation
		Ok(true)
	}

	// TODO: left/right vs. up/down, and use generated speech
	pub async fn text_cursor_moved(
		state: &ScreenReaderState,
		event: &TextCaretMovedEvent,
	) -> eyre::Result<()> {
		let text_len = state.new_accessible(event).await?.to_text().await?.character_count().await?;
		if is_tab_navigation(state, event, text_len).await? {
			return Ok(());
		}

		let text = state.new_accessible(event).await?.to_text().await?.get_string_at_offset(event.position(), *state.granularity.lock().await).await?.0;
		state.say(Priority::Text, text).await;
		Ok(())
	}

	pub async fn dispatch(state: &ScreenReaderState, event: &TextCaretMovedEvent) -> eyre::Result<()> {
		// Dispatch based on kind
		match event.kind() {
			"" => text_cursor_moved(state, event).await?,
			kind => tracing::debug!(kind, "Ignoring event with unknown kind"),
		}
		Ok(())
	}
} // end of text_caret_moved

mod state_changed {
	use crate::state::ScreenReaderState;
	use atspi::{accessible_ext::{AccessibleExt, AccessibleId}, identify::{object::StateChangedEvent}, signify::Signified, State};
	use odilia_cache::{AccessiblePrimitive};

	/// Update the state of an item in the cache using a StateChanged event and the ScreenReaderState as context.
	/// This writes to the value in-place, and does not clone any values.
	pub async fn update_state(state: &ScreenReaderState, a11y_id: &AccessibleId, state_changed: State, active: bool) -> eyre::Result<bool> {
		if active {
			Ok(state.cache.modify_item(a11y_id, |cache_item| cache_item.states.remove(state_changed)).await)
		} else {
			Ok(state.cache.modify_item(a11y_id, |cache_item| cache_item.states.insert(state_changed)).await)
		}
	}

	pub async fn dispatch(state: &ScreenReaderState, event: &StateChangedEvent) -> eyre::Result<()> {
		let accessible = state.new_accessible(event).await?;
		let _ci = state.cache.get_or_create(&accessible).await?;
		let a11y_state: State = match serde_plain::from_str(event.kind()) {
			Ok(s) => s,
			Err(e) => {
				tracing::error!("Not able to deserialize state: {}", event.kind());
				return Err(e.into())
			},
		};
		let state_value = event.enabled() == 1;
		let a11y_prim = AccessiblePrimitive::from_event(event)?;
		// update cache with state of item
		match update_state(state, &a11y_prim.id, a11y_state, state_value).await {
			Ok(false) => tracing::error!("Updating of the state was not succesful! The item with id {:?} was not found in the cache.", a11y_prim.id),
			Ok(true) => tracing::trace!("Updated the state of accessible with ID {:?}, and state {:?} to {state_value}.", a11y_prim.id, a11y_state),
			Err(e) => return Err(e),
		};
		// Dispatch based on kind
		let state_type = serde_plain::from_str(event.kind())?;
		// enabled can only be 1 or 0, but is not a boolean over dbus
		match (state_type, event.enabled() == 1) {
			(State::Focused, true) => focused(state, event).await?,
			(state, enabled) => tracing::debug!("Ignoring state_changed event with unknown kind: {:?}/{}", state, enabled),
		}
		Ok(())
	}

	pub async fn focused(state: &ScreenReaderState, event: &StateChangedEvent) -> eyre::Result<()> {
		let accessible =
			state.new_accessible(event).await?;
		if let Some(curr) = state.history_item(0).await? {
			if curr == accessible {
				return Ok(());
			}
		}

		let (name, description, role, relation, attrs, states) = tokio::try_join!(
			accessible.name(),
			accessible.description(),
			accessible.get_localized_role_name(),
			accessible.get_relation_set(),
			accessible.get_attributes(),
			accessible.get_state(),
		)?;
		let id = accessible.get_id();
		state.update_accessible(accessible.try_into()?).await;
		tracing::debug!("Focus event received on: {:?} with role {}", id, role);
		tracing::debug!("Relations: {:?}", relation);
		tracing::debug!("Attributes: {:?}", attrs);
		tracing::debug!("State: {:?}", states);

		// TODO: there should be a speech generation function
		if states.contains(State::Visited) {
			state.say(ssip_client::Priority::Text, format!("{name}, visited link"))
				.await;
			return Ok(());
		}
		if states.contains(State::Required) {
			state.say(ssip_client::Priority::Text, format!("{name}, required"))
				.await;
			return Ok(());
		}

		state.say(ssip_client::Priority::Text, format!("{name}, {role}"))
			.await;

		Ok(())
	}
}
