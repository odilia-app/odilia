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
		other_member => tracing::debug!("Ignoring event with unknown member: {:#?}", other_member),
	}
	Ok(())
}

mod children_changed {
	use odilia_cache::CacheItem;
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
		// all these properties will be fetched in paralell
		let (app, parent, index, children, ifaces, role, states, text) = tokio::try_join!(
			accessible.get_application(),
			accessible.parent(),
			accessible.get_index_in_parent(),
			accessible.child_count(),
			accessible.get_interfaces(),
			accessible.get_role(),
			accessible.get_state(),
			accessible.name(),
		)?;
		/*
		let cache_item = CacheItem {
			object: accessible.try_into().unwrap(),
			app: app.try_into().unwrap(),
			parent: parent.try_into().unwrap(),
			index,
			children,
			ifaces,
			role,
			states,
			text,
		};

		// finally, write data to the internal cache
		state.cache.add(cache_item).await;
		tracing::debug!("Add a single item to cache.");
		*/
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
	use atspi::{accessible_ext::AccessibleId, convertable::Convertable, events::GenericEvent, identify::object::TextCaretMovedEvent, signify::Signified};
	use ssip_client::Priority;

	/// this must be checked *before* writing an accessible to the hsitory.
	/// if this is checked after writing, it may give inaccurate results.
	/// that said, this is a *guess* and not a guarentee.
	/// TODO: make this a testable function, anything which queries "state" is not testable
	async fn is_tab_navigation(state: &ScreenReaderState, event: &TextCaretMovedEvent) -> eyre::Result<bool> {
		let current_caret_pos = event.position();
		// if the carat position is not at 0, we know that it is not a tab navigation, this is because tab will automatically set the cursor position at 0.
		if current_caret_pos != 0 {
			return Ok(false);
		}
		// Hopefully this shouldn't happen, but technically the caret may change before any other event happens. Since we already know that the caret position is 0, it may be a caret moved event
		let last_accessible = match state.history_item(0).await? {
			Some(acc) => acc,
			None => return Ok(true),
		};
		// likewise when getting the second-most recently focused accessible; we need the second-most recent accessible because it is possible that a tab navigation happened, which focused something before (or after) the caret moved events gets called, meaning the second-most recent accessible may be the only different accessible.
		// if the accessible is focused before the event happens, the last_accessible variable will be the same as current_accessible.
		// if the accessible is focused after the event happens, then the last_accessible will be different
		let last_last_accessible = match state.history_item(1).await? {
			Some(acc) => acc,
			None => return Ok(true),
		};
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
		if is_tab_navigation(state, event).await? {
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
	use atspi::{accessible_ext::{AccessibleId, AccessibleExt}, identify::{object::StateChangedEvent}, signify::Signified};

	pub async fn dispatch(state: &ScreenReaderState, event: &StateChangedEvent) -> eyre::Result<()> {
		// Dispatch based on kind
		match event.kind() {
			"focused" => focused(state, event).await?,
			kind => tracing::debug!(kind, "Ignoring event with unknown kind"),
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

		let (name, description, role, relation) = tokio::try_join!(
			accessible.name(),
			accessible.description(),
			accessible.get_localized_role_name(),
			accessible.get_relation_set(),
		)?;
		let id = accessible.get_id();
		state.update_accessible(accessible.try_into()?).await;
		tracing::debug!("Focus event received on: {:?} with role {}", id, role);
		tracing::debug!("Relations: {:?}", relation);

		state.say(ssip_client::Priority::Text, format!("{name}, {role}. {description}"))
			.await;

		Ok(())
	}
}
