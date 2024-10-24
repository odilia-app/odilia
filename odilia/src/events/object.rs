mod text_changed {

	use odilia_common::{
		errors::OdiliaError,
		result::OdiliaResult,
		types::{AriaAtomic, AriaLive},
	};

	use std::collections::HashMap;

	/// Get the live state of a set of attributes.
	/// Although the function only currently tests one attribute, in the future it may be important to inspect many attributes, compare them, or do additional logic.
	#[tracing::instrument(level = "trace", ret)]
	pub fn get_live_state(attributes: &HashMap<String, String>) -> OdiliaResult<AriaLive> {
		match attributes.get("live") {
			None => Err(OdiliaError::NoAttributeError("live".to_string())),
			Some(live) => Ok(serde_plain::from_str(live)?),
		}
	}

	#[tracing::instrument(level = "trace", ret)]
	pub fn get_atomic_state(attributes: &HashMap<String, String>) -> OdiliaResult<AriaAtomic> {
		match attributes.get("atomic") {
			None => Err(OdiliaError::NoAttributeError("atomic".to_string())),
			Some(atomic) => Ok(serde_plain::from_str(atomic)?),
		}
	}
}

mod children_changed {}

mod text_caret_moved {

	use atspi_common::Granularity;
	use odilia_cache::CacheItem;

	use odilia_common::errors::OdiliaError;
	use std::cmp::{max, min};
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
} // end of text_caret_moved

mod state_changed {}

#[cfg(test)]
mod tests {

	use atspi_common::{Interface, InterfaceSet, Role, State, StateSet};
	use atspi_connection::AccessibilityConnection;
	use lazy_static::lazy_static;
	use odilia_cache::{Cache, CacheItem};
	use odilia_common::cache::AccessiblePrimitive;
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
}
