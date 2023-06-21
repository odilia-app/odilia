use crate::{
	state::ScreenReaderState,
	traits::{IntoOdiliaCommands, IntoStateProduct, Command},
};
use async_trait::async_trait;
use atspi_common::events::object::{ObjectEvents, TextChangedEvent};
use odilia_common::events::{ScreenReaderEvent};
use odilia_common::{
	cache::{CacheRef, CacheValue, CacheItem},
	errors::{OdiliaError, CacheError, CommandError},
	commands::{OdiliaCommand, SetTextCommand},
	state::{OdiliaState, TextInsertState, TextDeleteState, TextState},
};

impl IntoOdiliaCommands for TextState {
	fn commands(&self) -> Result<Vec<OdiliaCommand>, OdiliaError> {
		match self {
			TextState::Insert(insert_state) => insert_state.commands(),
			TextState::Delete(delete_state) => delete_state.commands(),
		}
	}
}

impl Command for SetTextCommand {
	fn execute(&self) -> Result<(), OdiliaError> {
		let cache_item_arc = get_cache_item!(self.apply_to.item);
		{
			let mut cache_item = cache_item_arc.write();
			cache_item.text = self.new_text.clone();
		}
		Ok(())
	}
}

impl IntoStateProduct for TextChangedEvent {
	type ProductType = TextState;

	fn create(&self, state: &ScreenReaderState) -> Result<Self::ProductType, OdiliaError> {
		match self.operation.as_str() {
			"insert" | "insert/system" => {
				Ok(TextInsertState {
					start_index: self.start_pos as usize,
					text: self.text.clone(),
					apply_to: state.cache.get_key(&(self.item.clone().into())),
				}.into())
			},
			"delete" | "delete/system" => {
				Ok(TextDeleteState {
					start_index: self.start_pos as usize,
					end_index: self.start_pos as usize + self.length as usize,
					apply_to: state.cache.get_key(&(self.item.clone().into())),
				}.into())
			},
			_ => {
				Err(CommandError::InvalidKind("operation for TextChangedEvent: \"{operation}\"".to_string()).into())
			},
		}
	}
}

impl IntoOdiliaCommands for TextInsertState {
	// TODO: handle speaking if in an aria-live region
	fn commands(&self) -> Result<Vec<OdiliaCommand>, OdiliaError> {
		let cache_item_arc = get_cache_item!(self.apply_to.item);
		// get the text, then immediately drop the read guard
		let cache_text = {
			cache_item_arc
				.read()
				.text
				.clone()
		};
		let new_text = insert_text(self.start_index, &self.text, &cache_text);
		Ok(vec![
			SetTextCommand {
				new_text,
				apply_to: self.apply_to.clone(),
			}.into()
		])
	}
}
impl IntoOdiliaCommands for TextDeleteState {
	// TODO: handle speaking if in an aria-live region
	fn commands(&self) -> Result<Vec<OdiliaCommand>, OdiliaError> {
		let cache_item_arc = self.apply_to.item
			.upgrade()
			.ok_or_else(|| {
				tracing::trace!("There was a problem upgrading a Weak to Arc. This usually means the item was deleted: {:?}", self.apply_to.key);
				CacheError::NoItem
			})?;
		// get the text, then immediately drop the read guard
		let cache_text = {
			cache_item_arc
				.read()
				.text
				.clone()
		};
		let new_text = delete_text(&cache_text, self.start_index, self.end_index);
		Ok(vec![
			SetTextCommand {
				new_text,
				apply_to: self.apply_to.clone(),
			}.into()
		])
	}
}

pub fn insert_text(
	start_pos: usize,
	insert_text: &str,
	cache_text: &str,
) -> String {
	let char_num = cache_text.chars().count();
	let mut new_text = cache_text.chars().collect::<Vec<char>>();
	let insertion_index = std::cmp::min(start_pos, char_num);
	new_text.splice(insertion_index..insertion_index, insert_text.chars());
	new_text.into_iter().collect()
}

pub fn get_string_within_bounds(
	start_pos: usize,
	end_pos: usize,
) -> impl Fn((usize, char)) -> Option<char> {
	move |(index, chr)| {
		let is_after_start = index >= start_pos;
		let is_before_end = index <= end_pos;
		if is_after_start && is_before_end {
			Some(chr)
		} else {
			None
		}
	}
}

pub fn get_string_without_bounds(
	start_pos: usize,
	end_pos: usize,
) -> impl Fn((usize, char)) -> Option<char> {
	move |(index, chr)| {
		let is_before_start = index < start_pos;
		let is_after_end = index >= end_pos;
		if is_before_start || is_after_end {
			Some(chr)
		} else {
			None
		}
	}
}
pub fn delete_text(
	text: &str,
	start_pos: usize,
	end_pos: usize,
) -> String {
	text
		.char_indices()
		.filter_map(get_string_without_bounds(
			start_pos,
			end_pos,
		))
		.collect()
}

#[cfg(test)]
mod test {
	use std::sync::Arc;
	use parking_lot::RwLock;
	use super::{TextInsertState, TextDeleteState};
	use crate::traits::{IntoOdiliaCommands, Command};
	use odilia_common::{
		cache::{CacheRef, CacheValue, CacheItem, AccessiblePrimitive},
		commands::OdiliaCommand,
		errors::OdiliaError,
	};
	use atspi_common::{
		StateSet, InterfaceSet, Role,
		events::{Accessible, object::TextChangedEvent},
	};

	macro_rules! arc_rw {
		($id:ident) => {
			Arc::new(RwLock::new($id))
		}
	}
	// TODO: remove when default is merged upstream
	macro_rules! default_cache_item {
		() => {
			Arc::new(RwLock::new(CacheItem {
				object: AccessiblePrimitive::default(),
				app: AccessiblePrimitive::default(),
				children: Vec::new(),
				children_num: 0,
				index: 0,
				interfaces: InterfaceSet::empty(),
				parent: CacheRef::default(),
				role: Role::Invalid,
				states: StateSet::empty(),
				text: "The Industrial Revolution and its consequences have been a disaster for the human race".to_string(),
			}))
		}
	}

	macro_rules! cache_ref {
		($cache_item:ident) => {
			CacheRef {
				key: RwLock::read(&*$cache_item).object.clone(),
				item: Arc::downgrade(&$cache_item),
			}
		}
	}

	macro_rules! execute_state {
		($state_struct:ident) => {
			$state_struct.commands()?
				.iter()
				.map(<OdiliaCommand as Command>::execute)
				.collect::<Result<(), OdiliaError>>()
		}
	}

	#[test]
	fn test_insert_text_at_beginning() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_ref = cache_ref!(cache_item_arc);
		let event = TextInsertState {
			apply_to: cache_ref,
			start_index: 0,
			text: "1. ".to_string(),
		};
		let _ = execute_state!(event);
		assert_eq!(cache_item_arc.read().text, "1. The Industrial Revolution and its consequences have been a disaster for the human race");
		Ok(())
	}
	#[test]
	fn test_insert_text_at_end() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_ref = cache_ref!(cache_item_arc);
		let event = TextInsertState {
			apply_to: cache_ref,
			start_index: 86,
			text: ".".to_string(),
		};
		let _ = execute_state!(event);
		assert_eq!(cache_item_arc.read().text, "The Industrial Revolution and its consequences have been a disaster for the human race.");
		Ok(())
	}
	#[test]
	fn test_insert_text_in_middle() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_ref = cache_ref!(cache_item_arc);
		let event = TextInsertState {
			apply_to: cache_ref,
			start_index: 12,
			text: "random insert".to_string(),
		};
		let _ = execute_state!(event);
		assert_eq!(cache_item_arc.read().text, "The Industrirandom insertal Revolution and its consequences have been a disaster for the human race");
		Ok(())
	}
	#[test]
	fn test_delete_text_in_middle() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_ref = cache_ref!(cache_item_arc);
		let event = TextDeleteState {
			apply_to: cache_ref,
			end_index: 81 + 2,
			start_index: 81,
		};
		let _ = execute_state!(event);
		assert_eq!(cache_item_arc.read().text, "The Industrial Revolution and its consequences have been a disaster for the humanace");
		Ok(())
	}
	#[test]
	fn test_delete_text_negative_case_start_index() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_ref = cache_ref!(cache_item_arc);
		let event = TextDeleteState {
			apply_to: cache_ref,
			end_index: 1,
			start_index: (-1 as i32) as usize,
		};
		let _ = execute_state!(event);
		// should leave text exactly the same
		assert_eq!(cache_item_arc.read().text, "The Industrial Revolution and its consequences have been a disaster for the human race");
		Ok(())
	}
	#[test]
	fn test_delete_text_negative_case_start_and_end_index() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_ref = cache_ref!(cache_item_arc);
		let event = TextInsertState {
			apply_to: cache_ref,
			start_index: (-1 as i32) as usize,
			text: "???".to_string(),
		};
		let _ = execute_state!(event);
		assert_eq!(cache_item_arc.read().text, "The Industrial Revolution and its consequences have been a disaster for the human race???");
		Ok(())
	}
	#[test]
	fn test_insert_text_negative_start() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_ref = cache_ref!(cache_item_arc);
		let event = TextInsertState {
			apply_to: cache_ref,
			// is this the correct way to do it?
			// TODO: this makes it place the text at the end, regardless of the end index
			start_index: (-5 as i32) as usize,
			text: "??".to_string(),
		};
		let _ = execute_state!(event);
		// should stay the same
		assert_eq!(cache_item_arc.read().text, "The Industrial Revolution and its consequences have been a disaster for the human race??");
		Ok(())
	}
	#[test]
	fn test_insert_text_negative_end() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_ref = cache_ref!(cache_item_arc);
		let event = TextInsertState {
			apply_to: cache_ref,
			// this places the text in `text` at the position, with no other conditions
			// TODO: should this be the case
			start_index: 2 as usize,
			text: "??".to_string(),
		};
		let _ = execute_state!(event);
		// should stay the same
		assert_eq!(cache_item_arc.read().text, "Th??e Industrial Revolution and its consequences have been a disaster for the human race");
		Ok(())
	}
}
