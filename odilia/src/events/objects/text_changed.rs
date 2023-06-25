use crate::{
	state::ScreenReaderState,
	traits::{IntoOdiliaCommands, StateView, Command, IntoStateView, MutableStateView, IntoMutableStateView},
};
use std::sync::Arc;
use async_trait::async_trait;
use atspi_common::events::object::{ObjectEvents, TextChangedEvent};
use odilia_common::events::{ScreenReaderEvent};
use odilia_common::{
	cache::{CacheKey, ExternalCacheItem},
	errors::{OdiliaError, CacheError},
	commands::{OdiliaCommand, SetTextCommand},
};
use odilia_cache::{CacheRef, CacheValue, CacheItem};

impl MutableStateView for SetTextCommand {
	type View = CacheValue;
}
impl IntoMutableStateView for SetTextCommand {
	fn create_view(&self, state: &ScreenReaderState) -> Result<<Self as MutableStateView>::View, OdiliaError> {
		state.cache.get_ref(&self.apply_to)
			.ok_or(CacheError::NoItem.into())
	}
}
impl Command for SetTextCommand {
	fn execute(&self, cache_lock: <Self as MutableStateView>::View) -> Result<(), OdiliaError> {
		let mut cache_item = cache_lock.lock();
		cache_item.text = self.new_text.clone();
		Ok(())
	}
}

impl IntoOdiliaCommands for TextChangedEvent {
	// TODO: handle speaking if in an aria-live region
	fn commands(&self, state_view: &<Self as StateView>::View) -> Result<Vec<OdiliaCommand>, OdiliaError> {
		let new_text = match self.operation.as_str() {
			"insert" | "insert/system" => insert_text(self.start_pos as usize, &self.text, &state_view.text),
			"delete" | "delete/system" => delete_text(&state_view.text, self.start_pos as usize, (self.start_pos + self.length) as usize),
			_ => return Err(OdiliaError::UnknownKind(format!("Unknown kind for TextChangedEvent: {:?}", self.operation))),
		};
		Ok(vec![
			SetTextCommand {
				new_text,
				apply_to: state_view.object.clone(),
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
	use crate::traits::{IntoOdiliaCommands, Command};
	use odilia_common::{
		cache::{AccessiblePrimitive, CacheKey, ExternalCacheItem},
		commands::{OdiliaCommand, SetTextCommand},
		errors::OdiliaError,
	};
	use odilia_cache::{CacheRef, CacheValue, CacheItem};
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
				object: AccessiblePrimitive {
					id: "/none".to_string(),
					sender: ":0.0".to_string().into(),
				},
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

	macro_rules! text_test {
		($test_name:ident, $event:expr, $new_text:literal) => {
			#[test]
			fn $test_name() -> Result<(), OdiliaError> {
				let cache_item_arc = default_cache_item!();
				let cache_item = cache_item_arc.read().clone();
				let event = $event;
				let first_command: SetTextCommand = event.commands(&cache_item.into())?[0].clone().try_into()?;
				assert_eq!(first_command.new_text, $new_text);
				Ok(())
			}
		}
	}

	text_test!(
		test_insert_text_at_beginning, 
		TextChangedEvent {
			operation: "insert".to_string(),
			start_pos: 0,
			length: 3,
			item: Accessible::default(),
			text: "1. ".to_string(),
		},
		"1. The Industrial Revolution and its consequences have been a disaster for the human race"
	);
	text_test!(
		test_insert_text_at_end, 
		TextChangedEvent {
			operation: "insert".to_string(),
			start_pos: 86,
			length: 1,
			item: Accessible::default(),
			text: ".".to_string(),
		},
		"The Industrial Revolution and its consequences have been a disaster for the human race."
	);
	text_test!(
		test_insert_text_in_middle, 
		TextChangedEvent {
			operation: "insert".to_string(),
			start_pos: 12,
			length: 13,
			item: Accessible::default(),
			text: "random insert".to_string(),
		},
		"The Industrirandom insertal Revolution and its consequences have been a disaster for the human race"
	);
	text_test!(
		test_delete_text_in_middle, 
		TextChangedEvent {
			operation: "delete".to_string(),
			start_pos: 81,
			length: 3,
			item: Accessible::default(),
			text: " ra".to_string(),
		},
		"The Industrial Revolution and its consequences have been a disaster for the humance"
	);
	text_test!(
		test_insert_mismached_length_text_in_middle, 
		TextChangedEvent {
			operation: "insert".to_string(),
			start_pos: 3,
			length: 5,
			item: Accessible::default(),
			text: " LOL".to_string(),
		},
		"The LOL Industrial Revolution and its consequences have been a disaster for the human race"
	);
	text_test!(
		test_delete_text_negative_case_start_index_and_overflow, 
		TextChangedEvent {
			operation: "delete".to_string(),
			start_pos: -1,
			length: 5,
			item: Accessible::default(),
			text: "".to_string(),
		},
		"The Industrial Revolution and its consequences have been a disaster for the human race"
	);
	text_test!(
		test_insert_text_negative_case_start_index_and_overflow, 
		TextChangedEvent {
			operation: "insert".to_string(),
			start_pos: -5,
			length: 3,
			item: Accessible::default(),
			text: "???".to_string(),
		},
		"The Industrial Revolution and its consequences have been a disaster for the human race???"
	);
	text_test!(
		test_insert_text_in_middle_again,
		TextChangedEvent {
			operation: "insert".to_string(),
			start_pos: 2,
			length: 3,
			item: Accessible::default(),
			text: "???".to_string(),
		},
		"Th???e Industrial Revolution and its consequences have been a disaster for the human race"
	);
	text_test!(
		test_wrong_length_insert,
		TextChangedEvent {
			operation: "insert".to_string(),
			start_pos: 2,
			length: 8,
			item: Accessible::default(),
			text: "???".to_string(),
		},
		"Th???e Industrial Revolution and its consequences have been a disaster for the human race"
	);
	text_test!(
		test_mismatched_text_with_delete,
		TextChangedEvent {
			operation: "delete".to_string(),
			start_pos: 0,
			length: 8,
			item: Accessible::default(),
			text: "The".to_string(),
		},
		"strial Revolution and its consequences have been a disaster for the human race"
	);
}
