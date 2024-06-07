use crate::state::ScreenReaderState;
use atspi_common::events::object::ObjectEvents;

#[tracing::instrument(level = "debug", skip(state), ret, err)]
pub async fn dispatch(state: &ScreenReaderState, event: &ObjectEvents) -> eyre::Result<()> {
	// Dispatch based on member
	match event {
		ObjectEvents::StateChanged(state_changed_event) => {
			state_changed::dispatch(state, state_changed_event).await?;
		}
		ObjectEvents::TextCaretMoved(text_caret_moved_event) => {
			text_caret_moved::dispatch(state, text_caret_moved_event).await?;
		}
		ObjectEvents::TextChanged(text_changed_event) => {
			text_changed::dispatch(state, text_changed_event).await?;
		}
		ObjectEvents::ChildrenChanged(children_changed_event) => {
			children_changed::dispatch(state, children_changed_event).await?;
		}
		other_member => {
			tracing::debug!("Ignoring event with unknown member: {:#?}", other_member);
		}
	}
	Ok(())
}

mod text_changed {
	use crate::state::ScreenReaderState;
	use atspi_common::events::object::TextChangedEvent;
	use odilia_cache::CacheItem;
	use odilia_common::{
		errors::OdiliaError,
		result::OdiliaResult,
		types::{AriaAtomic, AriaLive},
	};
	use ssip_client_async::Priority;
	use std::collections::HashMap;

	#[inline]
	#[tracing::instrument(level = "trace")]
	pub fn update_string_insert(
		start_pos: usize,
		update_length: usize,
		updated_text: &str,
	) -> impl Fn(&mut CacheItem) + '_ {
		move |cache_item| {
			tracing::trace!(
				"Insert into \"{}\"({:?} @ {}+{} should insert \"{}\"",
				cache_item.text,
				cache_item.object.id,
				start_pos,
				update_length,
				updated_text
			);
			let char_num = cache_item.text.chars().count();
			let prepend = start_pos == 0;
			let append = start_pos >= char_num;
			// if the end of the inserted string will go past the end of the original string
			let insert_and_append = start_pos + update_length >= char_num;
			cache_item.text = if prepend {
				append_to_object(updated_text, &cache_item.text)
			} else if append {
				append_to_object(&cache_item.text, updated_text)
			} else if insert_and_append {
				insert_at_index(&cache_item.text, updated_text, start_pos)
			} else {
				insert_at_range(
					&cache_item.text,
					updated_text,
					start_pos,
					start_pos + update_length,
				)
			}
		}
	}

	#[inline]
	pub fn append_to_object(original: &str, to_append: &str) -> String {
		let mut new_text = original.chars().collect::<Vec<char>>();
		new_text.extend(to_append.chars());
		new_text.into_iter().collect()
	}

	#[inline]
	pub fn insert_at_index(original: &str, to_splice: &str, index: usize) -> String {
		let mut new_text = original.chars().collect::<Vec<char>>();
		new_text.splice(index.., to_splice.chars());
		new_text.into_iter().collect()
	}

	#[inline]
	pub fn insert_at_range(
		original: &str,
		to_splice: &str,
		start: usize,
		end: usize,
	) -> String {
		let mut new_text = original.chars().collect::<Vec<char>>();
		new_text.splice(start..end, to_splice.chars());
		new_text.into_iter().collect()
	}

	#[inline]
	/// Get the live state of a set of attributes.
	/// Although the function only currently tests one attribute, in the future it may be important to inspect many attributes, compare them, or do additional logic.
	#[tracing::instrument(level = "trace", ret, err)]
	pub fn get_live_state(attributes: &HashMap<String, String>) -> OdiliaResult<AriaLive> {
		match attributes.get("live") {
			None => Err(OdiliaError::NoAttributeError("live".to_string())),
			Some(live) => Ok(serde_plain::from_str(live)?),
		}
	}

	#[inline]
	/// if the aria-live attribute is set to "polite", then set the priority of the message to speak once all other messages are done
	/// if the aria-live attribute is set to "assertive", then set the priority of the message to speak immediately, stop all other messages, and do not interrupt that piece of speech
	/// otherwise, do not continue
	pub fn live_to_priority(live_str: &AriaLive) -> Priority {
		match live_str {
			AriaLive::Assertive => Priority::Important,
			AriaLive::Polite => Priority::Notification,
			_ => Priority::Message,
		}
	}

	#[inline]
	#[tracing::instrument(level = "trace", ret, err)]
	pub fn get_atomic_state(attributes: &HashMap<String, String>) -> OdiliaResult<AriaAtomic> {
		match attributes.get("atomic") {
			None => Err(OdiliaError::NoAttributeError("atomic".to_string())),
			Some(atomic) => Ok(serde_plain::from_str(atomic)?),
		}
	}

	pub fn get_string_within_bounds(
		start_pos: usize,
		update_length: usize,
	) -> impl Fn((usize, char)) -> Option<char> {
		move |(index, chr)| {
			let is_after_start = index >= start_pos;
			let is_before_end = index <= start_pos + update_length;
			if is_after_start && is_before_end {
				Some(chr)
			} else {
				None
			}
		}
	}

	pub fn get_string_without_bounds(
		start_pos: usize,
		update_length: usize,
	) -> impl Fn((usize, char)) -> Option<char> {
		move |(index, chr)| {
			let is_before_start = index < start_pos;
			let is_after_end = index > start_pos + update_length;
			if is_before_start || is_after_end {
				Some(chr)
			} else {
				None
			}
		}
	}

	#[tracing::instrument(level = "debug", skip(state), ret, err)]
	pub async fn dispatch(
		state: &ScreenReaderState,
		event: &TextChangedEvent,
	) -> eyre::Result<()> {
		match event.operation.as_str() {
			"insert/system" => insert_or_delete(state, event, true).await?,
			"insert" => insert_or_delete(state, event, true).await?,
			"delete/system" => insert_or_delete(state, event, false).await?,
			"delete" => insert_or_delete(state, event, false).await?,
			_ => tracing::trace!(
				"TextChangedEvent has invalid kind: {}",
				event.operation
			),
		};
		Ok(())
	}

	#[tracing::instrument(level = "debug", skip(state), ret, err)]
	pub async fn speak_insertion(
		state: &ScreenReaderState,
		event: &TextChangedEvent,
		attributes: &HashMap<String, String>,
		cache_text: &str,
	) -> OdiliaResult<()> {
		// note, you should update the text before this happens, since this could potentially end the function
		let live = get_live_state(attributes)?;
		let atomic = get_atomic_state(attributes)?;
		// if the atomic state is true, then read out the entite piece of text
		// if atomic state is false, then only read out the portion which has been added
		// otherwise, do not continue through this function
		let text_to_say =
			if atomic { cache_text.to_string() } else { (&event.text).into() };
		let priority = live_to_priority(&live);
		state.say(priority, text_to_say).await;
		Ok(())
	}

	/// The `insert` boolean, if set to true, will update the text in the cache.
	/// If it is set to false, the selection will be removed.
	/// The [`TextChangedEvent::operation`] value will *NOT* be checked by this function.
	#[tracing::instrument(level = "debug", skip(state), ret, err)]
	pub async fn insert_or_delete(
		state: &ScreenReaderState,
		event: &TextChangedEvent,
		insert: bool,
	) -> eyre::Result<()> {
		let accessible = state.new_accessible(event).await?;
		let cache_item = state.get_or_create_event_object_to_cache(event).await?;
		let updated_text: String = (&event.text).into();
		let current_text = cache_item.text;
		let (start_pos, update_length) =
			(usize::try_from(event.start_pos)?, usize::try_from(event.length)?);
		// if this is an insert, figure out if we shuld announce anything, then speak it;
		// only after should we try to update the cache
		if insert {
			let attributes = accessible.get_attributes().await?;
			let _: OdiliaResult<()> =
				speak_insertion(state, event, &attributes, &current_text).await;
		}

		let text_selection_from_cache: String = current_text
			.char_indices()
			.filter_map(get_string_within_bounds(start_pos, update_length))
			.collect();
		let selection_matches_update = text_selection_from_cache == updated_text;
		let insert_has_not_occured = insert && !selection_matches_update;
		let remove_has_not_occured = !insert && selection_matches_update;
		if insert_has_not_occured {
			state.cache.modify_item(
				&cache_item.object,
				update_string_insert(start_pos, update_length, &updated_text),
			)?;
		} else if remove_has_not_occured {
			state.cache.modify_item(&cache_item.object, move |cache_item| {
				cache_item.text = cache_item
					.text
					.char_indices()
					.filter_map(get_string_without_bounds(
						start_pos,
						update_length,
					))
					.collect();
			})?;
		}
		Ok(())
	}
}

mod children_changed {
	use crate::state::ScreenReaderState;
	use atspi_common::events::object::ChildrenChangedEvent;
	use odilia_cache::{AccessiblePrimitive, CacheItem};
	use odilia_common::result::OdiliaResult;
	use std::sync::Arc;

	#[tracing::instrument(level = "debug", skip(state), ret, err)]
	pub async fn dispatch(
		state: &ScreenReaderState,
		event: &ChildrenChangedEvent,
	) -> eyre::Result<()> {
		// Dispatch based on kind
		match event.operation.as_str() {
			"remove" | "remove/system" => remove(state, event)?,
			"add" | "add/system" => add(state, event).await?,
			kind => tracing::debug!(kind, "Ignoring event with unknown kind"),
		}
		Ok(())
	}
	#[tracing::instrument(level = "debug", skip(state), ret, err)]
	pub async fn add(
		state: &ScreenReaderState,
		event: &ChildrenChangedEvent,
	) -> eyre::Result<()> {
		let accessible = get_child_primitive(event)
			.into_accessible(state.atspi.connection())
			.await?;
		let _: OdiliaResult<CacheItem> =
			state.cache.get_or_create(&accessible, Arc::clone(&state.cache)).await;
		tracing::debug!("Add a single item to cache.");
		Ok(())
	}
	#[inline]
	fn get_child_primitive(event: &ChildrenChangedEvent) -> AccessiblePrimitive {
		event.child.clone().into()
	}
	#[tracing::instrument(level = "debug", skip(state), ret, err)]
	pub fn remove(state: &ScreenReaderState, event: &ChildrenChangedEvent) -> eyre::Result<()> {
		let prim = get_child_primitive(event);
		state.cache.remove(&prim);
		tracing::debug!("Remove a single item from cache.");
		Ok(())
	}
}

mod text_caret_moved {
	use crate::state::ScreenReaderState;
	use atspi_common::events::object::TextCaretMovedEvent;
	use atspi_common::Granularity;
	use odilia_cache::CacheItem;
	use odilia_common::errors::{CacheError, OdiliaError};
	use ssip_client_async::Priority;
	use std::{
		cmp::{max, min},
		sync::atomic::Ordering,
	};
	use tracing::debug;

	#[tracing::instrument(level = "debug", ret, err)]
	pub async fn new_position(
		new_item: CacheItem,
		old_item: CacheItem,
		new_position: usize,
		old_position: usize,
	) -> Result<String, OdiliaError> {
		let new_id = new_item.object.clone();
		let old_id = old_item.object.clone();

		// if the user has moved into a new item, then also read a whole line.
		debug!("{new_id:?},{old_id:?}");
		debug!("{old_position},{new_position}");
		if new_id != old_id {
			return Ok(new_item
				.get_string_at_offset(new_position, Granularity::Line)
				.await?
				.0);
		}
		let first_position = min(new_position, old_position);
		let last_position = max(new_position, old_position);
		// if there is one character between the old and new position
		if new_position.abs_diff(old_position) == 1 {
			return Ok(new_item
				.get_string_at_offset(first_position, Granularity::Char)
				.await?
				.0);
		}
		let first_word = new_item
			.get_string_at_offset(first_position, Granularity::Word)
			.await?;
		let last_word = old_item
			.get_string_at_offset(last_position, Granularity::Word)
			.await?;
		// if words are the same
		if first_word == last_word ||
			// if the end position of the first word immediately preceeds the start of the second word
			first_word.2.abs_diff(last_word.1) == 1
		{
			return new_item.get_text(first_position, last_position);
		}
		// if the user has somehow from the beginning to the end. Usually happens with Home, the End.
		if first_position == 0 && last_position == new_item.text.len() {
			return Ok(new_item.text.clone());
		}
		Ok(new_item
			.get_string_at_offset(new_position, Granularity::Line)
			.await?
			.0)
	}

	/// this must be checked *before* writing an accessible to the hsitory.
	/// if this is checked after writing, it may give inaccurate results.
	/// that said, this is a *guess* and not a guarentee.
	/// TODO: make this a testable function, anything which queries "state" is not testable
	async fn is_tab_navigation(
		state: &ScreenReaderState,
		event: &TextCaretMovedEvent,
	) -> eyre::Result<bool> {
		let current_caret_pos = event.position;
		// if the caret position is not at 0, we know that it is not a tab navigation, this is because tab will automatically set the cursor position at 0.
		if current_caret_pos != 0 {
			return Ok(false);
		}
		// Hopefully this shouldn't happen, but technically the caret may change before any other event happens. Since we already know that the caret position is 0, it may be a caret moved event
		let last_accessible = match state.history_item(0).await {
			Some(acc) => state.get_or_create_cache_item(acc).await?,
			None => return Ok(true),
		};
		// likewise when getting the second-most recently focused accessible; we need the second-most recent accessible because it is possible that a tab navigation happened, which focused something before (or after) the caret moved events gets called, meaning the second-most recent accessible may be the only different accessible.
		// if the accessible is focused before the event happens, the last_accessible variable will be the same as current_accessible.
		// if the accessible is focused after the event happens, then the last_accessible will be different
		let previous_caret_pos = state.previous_caret_position.load(Ordering::Relaxed);
		let current_accessible = state.get_or_create_event_object_to_cache(event).await?;
		// if we know that the previous caret position was not 0, and the current and previous accessibles are the same, we know that this is NOT a tab navigation.
		if previous_caret_pos != 0 && current_accessible.object == last_accessible.object {
			return Ok(false);
		}
		// otherwise, it probably was a tab navigation
		Ok(true)
	}

	// TODO: left/right vs. up/down, and use generated speech
	#[tracing::instrument(level = "debug", skip(state), ret, err)]
	pub async fn text_cursor_moved(
		state: &ScreenReaderState,
		event: &TextCaretMovedEvent,
	) -> eyre::Result<()> {
		if is_tab_navigation(state, event).await? {
			return Ok(());
		}
		let new_item = state.get_or_create_event_object_to_cache(event).await?;

		let new_prim = new_item.object.clone();
		let text = match state.history_item(0).await {
			Some(old_prim) => {
				let old_pos = state.previous_caret_position.load(Ordering::Relaxed);
				let old_item =
					state.cache.get(&old_prim).ok_or(CacheError::NoItem)?;
				let new_pos = event.position;
				new_position(
					new_item,
					old_item,
					new_pos.try_into()
						.expect("Can not convert between i32 and usize"),
					old_pos,
				)
				.await?
			}
			None => {
				// if no previous item exists, as in the screen reader has just loaded, then read out the whole item.
				new_item.get_string_at_offset(0, Granularity::Paragraph).await?.0
			}
		};
		state.say(Priority::Text, text).await;
		state.update_accessible(new_prim).await;
		Ok(())
	}

	#[tracing::instrument(level = "debug", skip(state), ret, err)]
	pub async fn dispatch(
		state: &ScreenReaderState,
		event: &TextCaretMovedEvent,
	) -> eyre::Result<()> {
		text_cursor_moved(state, event).await?;

		state.previous_caret_position.store(
			event.position
				.try_into()
				.expect("Converting from an i32 to a usize must not fail"),
			Ordering::Relaxed,
		);
		Ok(())
	}
} // end of text_caret_moved

mod state_changed {
	use crate::state::ScreenReaderState;
	use atspi_common::{events::object::StateChangedEvent, State};
	use odilia_cache::AccessiblePrimitive;

	/// Update the state of an item in the cache using a `StateChanged` event and the `ScreenReaderState` as context.
	/// This writes to the value in-place, and does not clone any values.
	pub fn update_state(
		state: &ScreenReaderState,
		a11y: &AccessiblePrimitive,
		state_changed: State,
		active: bool,
	) -> eyre::Result<bool> {
		if active {
			Ok(state.cache.modify_item(a11y, |cache_item| {
				cache_item.states.remove(state_changed);
			})?)
		} else {
			Ok(state.cache.modify_item(a11y, |cache_item| {
				cache_item.states.insert(state_changed);
			})?)
		}
	}

	#[tracing::instrument(level = "debug", skip(state), ret, err)]
	pub async fn dispatch(
		state: &ScreenReaderState,
		event: &StateChangedEvent,
	) -> eyre::Result<()> {
		let state_value = event.enabled == 1;
		// update cache with state of item
		let a11y_prim = AccessiblePrimitive::from_event(event);
		if update_state(state, &a11y_prim, event.state, state_value)? {
			tracing::debug!("Updating of the state was not succesful! The item with id {:?} was not found in the cache.", a11y_prim.id);
		} else {
			tracing::trace!("Updated the state of accessible with ID {:?}, and state {:?} to {state_value}.", a11y_prim.id, event.state);
		}
		// enabled can only be 1 or 0, but is not a boolean over dbus
		match (event.state, event.enabled == 1) {
			(State::Focused, true) => focused(state, event).await?,
			(state, enabled) => tracing::trace!(
				"Ignoring state_changed event with unknown kind: {:?}/{}",
				state,
				enabled
			),
		}
		Ok(())
	}

	#[tracing::instrument(level = "debug", skip(state), ret, err)]
	pub async fn focused(
		state: &ScreenReaderState,
		event: &StateChangedEvent,
	) -> eyre::Result<()> {
		let accessible = state.get_or_create_event_object_to_cache(event).await?;
		if let Some(curr) = state.history_item(0).await {
			if curr == accessible.object {
				return Ok(());
			}
		}

		let (name, description, relation) = tokio::try_join!(
			accessible.name(),
			accessible.description(),
			accessible.get_relation_set(),
		)?;
		state.update_accessible(accessible.object.clone()).await;
		tracing::debug!(
			"Focus event received on: {:?} with role {}",
			accessible.object.id,
			accessible.role,
		);
		tracing::debug!("Relations: {:?}", relation);

		state.say(
			ssip_client_async::Priority::Text,
			format!("{name}, {0}. {description}", accessible.role),
		)
		.await;

		state.update_accessible(accessible.object).await;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::events::object::text_caret_moved::new_position;
	use atspi_common::{Interface, InterfaceSet, Role, State, StateSet};
	use atspi_connection::AccessibilityConnection;
	use lazy_static::lazy_static;
	use odilia_cache::{AccessiblePrimitive, Cache, CacheItem};
	use std::sync::Arc;
	use tokio_test::block_on;

	static A11Y_PARAGRAPH_STRING: &str = "The AT-SPI (Assistive Technology Service Provider Interface) enables users of Linux to use their computer without sighted assistance. It was originally developed at Sun Microsystems, before they were purchased by Oracle.";
	lazy_static! {
		static ref ZBUS_CONN: AccessibilityConnection =
			#[allow(clippy::unwrap_used)]
			block_on(AccessibilityConnection::new()).unwrap();
		static ref CACHE_ARC: Arc<Cache> =
			Arc::new(Cache::new(ZBUS_CONN.connection().clone()));
		static ref A11Y_PARAGRAPH_ITEM: CacheItem = CacheItem {
			object: AccessiblePrimitive {
				id: "/org/a11y/atspi/accessible/1".to_string(),
				sender: ":1.2".into(),
			},
			app: AccessiblePrimitive {
				id: "/org/a11y/atspi/accessible/root".to_string(),
				sender: ":1.2".into()
			},
			parent: AccessiblePrimitive {
				id: "/otg/a11y/atspi/accessible/1".to_string(),
				sender: ":1.2".into(),
			}
			.into(),
			index: Some(323),
			children_num: Some(0),
			interfaces: InterfaceSet::new(
				Interface::Accessible
					| Interface::Collection | Interface::Component
					| Interface::Hyperlink | Interface::Hypertext
					| Interface::Text
			),
			role: Role::Paragraph,
			states: StateSet::new(
				State::Enabled | State::Opaque | State::Showing | State::Visible
			),
			text: A11Y_PARAGRAPH_STRING.to_string(),
			children: Vec::new(),
			cache: Arc::downgrade(&CACHE_ARC),
		};
		static ref ANSWER_VALUES: [(CacheItem, CacheItem, u32, u32, &'static str); 9] = [
			(A11Y_PARAGRAPH_ITEM.clone(), A11Y_PARAGRAPH_ITEM.clone(), 4, 3, " "),
			(A11Y_PARAGRAPH_ITEM.clone(), A11Y_PARAGRAPH_ITEM.clone(), 3, 4, " "),
			(A11Y_PARAGRAPH_ITEM.clone(), A11Y_PARAGRAPH_ITEM.clone(), 0, 3, "The"),
			(A11Y_PARAGRAPH_ITEM.clone(), A11Y_PARAGRAPH_ITEM.clone(), 3, 0, "The"),
			(
				A11Y_PARAGRAPH_ITEM.clone(),
				A11Y_PARAGRAPH_ITEM.clone(),
				169,
				182,
				"Microsystems,"
			),
			(
				A11Y_PARAGRAPH_ITEM.clone(),
				A11Y_PARAGRAPH_ITEM.clone(),
				77,
				83,
				" Linux"
			),
			(
				A11Y_PARAGRAPH_ITEM.clone(),
				A11Y_PARAGRAPH_ITEM.clone(),
				181,
				189,
				", before"
			),
			(
				A11Y_PARAGRAPH_ITEM.clone(),
				A11Y_PARAGRAPH_ITEM.clone(),
				0,
				220,
				A11Y_PARAGRAPH_STRING,
			),
			(
				A11Y_PARAGRAPH_ITEM.clone(),
				A11Y_PARAGRAPH_ITEM.clone(),
				220,
				0,
				A11Y_PARAGRAPH_STRING,
			),
		];
	}

	macro_rules! check_answer_values {
		($idx:literal) => {
			assert_eq!(
				tokio_test::block_on(new_position(
					ANSWER_VALUES[$idx].0.clone(),
					ANSWER_VALUES[$idx].1.clone(),
					ANSWER_VALUES[$idx].2.try_into().unwrap(),
					ANSWER_VALUES[$idx].3.try_into().unwrap(),
				))
				.unwrap(),
				ANSWER_VALUES[$idx].4.to_string(),
			);
		};
	}

	#[test]
	fn test_text_navigation_one_letter() {
		check_answer_values!(0);
	}
	#[test]
	fn test_text_navigation_one_letter_back() {
		check_answer_values!(1);
	}
	#[test]
	fn test_text_navigation_one_word_back() {
		check_answer_values!(2);
	}
	#[test]
	fn test_text_navigation_one_word() {
		check_answer_values!(3);
	}
	#[test]
	fn test_text_navigation_one_word_with_punctuation_after() {
		check_answer_values!(4);
	}
	#[test]
	fn test_text_navigation_one_word_with_space() {
		check_answer_values!(5);
	}
	#[test]
	fn test_text_navigation_one_word_with_punctutation_first() {
		check_answer_values!(6);
	}
	#[test]
	fn test_text_navigation_full_item_front_to_back() {
		check_answer_values!(7);
	}
	#[test]
	fn test_text_navigation_full_item_back_to_front() {
		check_answer_values!(8);
	}
}
