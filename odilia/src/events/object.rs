use crate::state::ScreenReaderState;
use atspi::{
	identify::{
		ObjectEvents,
	},
};

pub async fn dispatch(state: &ScreenReaderState, event: ObjectEvents) -> eyre::Result<()> {
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
	use atspi::{events::GenericEvent, identify::{object::ChildrenChangedEvent, Signified}};

	pub async fn dispatch(state: &ScreenReaderState, event: ChildrenChangedEvent) -> eyre::Result<()> {
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
	pub async fn add(state: &ScreenReaderState, event: ChildrenChangedEvent) -> eyre::Result<()> {
		let accessible = state.new_accessible(&event).await?;
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
		let cache_item = CacheItem {
			object: accessible.try_into().unwrap(),
			app: app.try_into().unwrap(),
			parent: parent.try_into().unwrap(),
			index,
			children,
			ifaces: ifaces.into(),
			role: role.into(),
			states: states.into(),
			text,
		};

		// finally, write data to the internal cache
		state.cache.add(cache_item).await;
		tracing::debug!("Add a single item to cache.");
		Ok(())
	}
	pub async fn remove(state: &ScreenReaderState, event: ChildrenChangedEvent) -> eyre::Result<()> {
		let path = event.path().expect("All accessibles must have a path").try_into()?;
		state.cache.remove(&path).await;
		tracing::debug!("Remove a single item from cache.");
		Ok(())
	}
}

mod text_caret_moved {
	use crate::state::ScreenReaderState;
	use atspi::{accessible_ext::AccessibleId, convertable::Convertable, events::GenericEvent, identify::{object::TextCaretMovedEvent, Signified}};
	use ssip_client::Priority;

	// TODO: left/right vs. up/down, and use generated speech
	pub async fn text_cursor_moved(
		state: &ScreenReaderState,
		event: TextCaretMovedEvent,
	) -> eyre::Result<()> {
		let current_caret_pos = event.position();
		let previous_caret_pos = state.previous_caret_position.get();
		state.previous_caret_position.set(current_caret_pos);
		let (_start, _end) = match current_caret_pos > previous_caret_pos {
			true => (previous_caret_pos, current_caret_pos),
			false => (current_caret_pos, previous_caret_pos),
		};
		let path = if let Some(path) = event.path() {
			path
		} else {
			return Ok(());
		};
		let accessible = state.new_accessible(&event).await?;
		let _last_accessible = match state.history_item(0).await? {
			Some(acc) => acc,
			None => return Ok(()),
		};
		let last_last_accessible = match state.history_item(1).await? {
			Some(acc) => acc,
			None => return Ok(()),
		};
		let id: AccessibleId = path.try_into()?;
		state.update_accessible(id).await;

		// in the case that this is not a tab navigation
		// TODO: algorithm that only triggers this when a tab navigation is known to have not occured. How the fuck am I supposed to know how that works?
		// Ok, start out with the basics: if a focus event has recently occuredm, there is a good chance that this function is about to get triggered as well. So, for one, a tab navigation GUARENTEES that the last_accessible will be equal to the curent accessible.
		if accessible == last_last_accessible {
			let txt = accessible.to_text().await?;
			let len = txt.character_count().await?;
			// TODO: improve text readout
			state.say(Priority::Text, (txt.get_text(0, len).await?).to_string())
				.await;
		}
		Ok(())
	}

	pub async fn dispatch(state: &ScreenReaderState, event: TextCaretMovedEvent) -> eyre::Result<()> {
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
	use atspi::{accessible_ext::AccessibleId, identify::{object::StateChangedEvent, Signified}};

	pub async fn dispatch(state: &ScreenReaderState, event: StateChangedEvent) -> eyre::Result<()> {
		// Dispatch based on kind
		match event.kind() {
			"focused" => focused(state, event).await?,
			kind => tracing::debug!(kind, "Ignoring event with unknown kind"),
		}
		Ok(())
	}

	pub async fn focused(state: &ScreenReaderState, event: StateChangedEvent) -> eyre::Result<()> {
		let accessible =
			state.new_accessible(&event).await?;
		if let Some(curr) = state.history_item(0).await? {
			if curr == accessible {
				return Ok(());
			}
		}
		let id: AccessibleId = accessible.path().try_into()?;
		state.update_accessible(id).await;

		let (name, description, role, relation) = tokio::try_join!(
			accessible.name(),
			accessible.description(),
			accessible.get_localized_role_name(),
			accessible.get_relation_set(),
		)?;
		tracing::debug!("Focus event received on: {:?} with role {}", id, role);
		tracing::debug!("Relations: {:?}", relation);

		state.say(ssip_client::Priority::Text, format!("{name}, {role}. {description}"))
			.await;

		Ok(())
	}
}
