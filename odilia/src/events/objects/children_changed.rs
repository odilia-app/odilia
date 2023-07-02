use crate::{
	state::ScreenReaderState,
	traits::{IntoOdiliaCommands, IntoStateView, Command, StateView, MutableStateView, IntoMutableStateView},
};
use async_trait::async_trait;
use atspi_common::events::object::ChildrenChangedEvent;
use atspi_common::State;
use odilia_common::events::{ScreenReaderEvent};
use odilia_common::{
	cache::ExternalCacheItem,
	errors::{OdiliaError, CacheError},
	commands::{OdiliaCommand, ChangeChildCommand},
};
use odilia_cache::{CacheRef, CacheValue, CacheItem};

#[async_trait]
impl IntoOdiliaCommands for ChildrenChangedEvent {
	async fn commands(&self, state_view: &<Self as StateView>::View) -> Result<Vec<OdiliaCommand>, OdiliaError> {
		Ok(vec![ChangeChildCommand {
			index: self.index_in_parent as usize,
			new_child: self.child.clone().into(),
			add: self.operation == "insert",
			apply_to: self.item.clone().into()
		}.into()])
	}
}

impl MutableStateView for ChangeChildCommand {
	type View = CacheValue;
}

#[async_trait]
impl IntoMutableStateView for ChangeChildCommand {
	async fn create_view(&self, state: &ScreenReaderState) -> Result<<Self as MutableStateView>::View, OdiliaError> {
		state.cache.get_ref(&self.apply_to)
			.await
			.ok_or(CacheError::NoItem.into())
	}
}

#[async_trait]
impl Command for ChangeChildCommand {
	async fn execute(&self, cache_item_lock: <Self as MutableStateView>::View) -> Result<(), OdiliaError> {
		let mut cache_item = cache_item_lock.lock().await;
		let mut new_children = cache_item.children.clone();
		let child: CacheRef = self.new_child.clone().into();
		if self.add {
			// if an insert will not cause a panic
			if new_children.len() < self.index {
				tracing::trace!("The child can not be inserted at the given index: {} because the length of the children vec is {}", self.index, new_children.len());
				return Err(CacheError::Invalidated(cache_item.object.clone()).into());
			}
			new_children.insert(self.index, child);
		} else {
			let has_index = self.index < new_children.len();
			let is_contained = new_children.contains(&child);
			let found_position = new_children.iter().position(|key| key.key == child.key);
			let correct_index = found_position == Some(self.index);
			match (has_index, is_contained, correct_index) {
				(false, _, _) => {
					tracing::trace!("Invalid index to remove child: {}, but child list contains {} items.  This event will be discarded.", self.index, new_children.len());
					return Err(CacheError::Invalidated(cache_item.object.clone()).into());
				},
				(true, false, _) => {
					tracing::trace!("The child {:?} is not contained as an item in the list of children {:?}. This event will be discarded.", cache_item.object.clone(), new_children);
					return Err(CacheError::Invalidated(cache_item.object.clone()).into());
				},
				(true, true, false) => {
					tracing::trace!("Mismatch between suggested index and child to remove. Child index: {:?}, suggested index for removal: {}", found_position, self.index);
					return Err(CacheError::Invalidated(cache_item.object.clone()).into());
				},
				(true, true, true) => {
					tracing::trace!("The child to remove matches the index in the cache.");
				},
			}
			new_children.remove(self.index);
		}
		cache_item.children = new_children;
		Ok(())
	}
}

#[cfg(test)]
pub mod test {
	use super::ChangeChildCommand;
	use std::sync::Arc;
	use tokio::sync::Mutex;
	use crate::traits::{IntoOdiliaCommands, Command};
	use odilia_cache::{CacheRef, CacheValue, CacheItem};
	use odilia_common::{
		cache::{AccessiblePrimitive, CacheKey, ExternalCacheItem},
		commands::{OdiliaCommand, SetTextCommand},
		errors::{OdiliaError, CacheError},
	};
	use atspi_common::{
		StateSet, InterfaceSet, Role,
		events::{Accessible, object::TextChangedEvent},
	};

	macro_rules! default_cache_item {
		() => {
			Arc::new(Mutex::new(CacheItem {
				object: AccessiblePrimitive {
					id: "/none".to_string(),
					sender: ":0.0".to_string().into(),
				},
				app: AccessiblePrimitive::default(),
				children: vec![
					AccessiblePrimitive {
						id: "/child/1".to_string(),
						sender: ":0.0".to_string().into(),
					}.into(),
					AccessiblePrimitive {
						id: "/child/2".to_string(),
						sender: ":0.0".to_string().into(),
					}.into(),
					AccessiblePrimitive {
						id: "/child/3".to_string(),
						sender: ":0.0".to_string().into(),
					}.into(),
				],
				children_num: 0,
				index: 0,
				interfaces: InterfaceSet::empty(),
				parent: CacheRef::default(),
				role: Role::Invalid,
				states: StateSet::empty(),
				text: String::new(),
			}))
		}
	}

	#[tokio::test]
	async fn test_add_child() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_item = cache_item_arc.lock().await;
		let child_4 = ChangeChildCommand {
				new_child: AccessiblePrimitive {
					id: "/child/4".to_string(),
					sender: ":0.0".to_string().into(),
				}.into(),
				index: 3,
				add: true,
				apply_to: cache_item.object.clone(),
		};
		// this is required so that mutex can be aquired in execute
		drop(cache_item);
		let execution_return = child_4.execute(Arc::clone(&cache_item_arc)).await;
		assert!(execution_return.is_ok(), "Command failed to run: {execution_return:?}");
		let cache_item = cache_item_arc.lock().await;
		assert_eq!(
			cache_item.children.len(),
			4,
			"The cache item should have 4 children"
		);
		assert_eq!(
			cache_item.children.get(3).unwrap().key,
			child_4.new_child,
			"The cache item was not found at the right location."
		);
		Ok(())
	}

	#[tokio::test]
	async fn test_remove_child() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_item = cache_item_arc.lock().await;
		let child_2 = ChangeChildCommand {
				new_child: cache_item.children.get(2).unwrap().key.clone(),
				index: 2,
				add: false,
				apply_to: cache_item.object.clone(),
		};
		// this is required so that mutex can be aquired in execute
		drop(cache_item);
		let execution_return = child_2.execute(Arc::clone(&cache_item_arc)).await;
		assert!(execution_return.is_ok(), "Command failed to run: {execution_return:?}");
		let cache_item = cache_item_arc.lock().await;
		assert_eq!(
			cache_item.children.len(),
			2,
			"The cache item should have 2 children"
		);
		Ok(())
	}

	#[tokio::test]
	async fn test_remove_index_too_high_child() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_item = cache_item_arc.lock().await;
		let child_2 = ChangeChildCommand {
				new_child: cache_item.children.get(2).unwrap().key.clone(),
				index: 3,
				add: false,
				apply_to: cache_item.object.clone(),
		};
		// this is required so that mutex can be aquired in execute
		drop(cache_item);
		let execution_return = child_2.execute(Arc::clone(&cache_item_arc)).await;
		let cache_item = cache_item_arc.lock().await;
		assert_eq!(Err(CacheError::Invalidated(cache_item.object.clone()).into()), execution_return);
		Ok(())
	}

	#[tokio::test]
	async fn test_remove_invalid_child() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_item = cache_item_arc.lock().await;
		let child_2 = ChangeChildCommand {
				new_child: AccessiblePrimitive {
					id: "/some/fake/child".to_string(),
					sender: ":0.0".to_string().into(),
				}.into(),
				index: 2,
				add: false,
				apply_to: cache_item.object.clone(),
		};
		// this is required so that mutex can be aquired in execute
		drop(cache_item);
		let execution_return = child_2.execute(Arc::clone(&cache_item_arc)).await;
		let cache_item = cache_item_arc.lock().await;
		assert_eq!(Err(CacheError::Invalidated(cache_item.object.clone()).into()), execution_return);
		Ok(())
	}

	#[tokio::test]
	async fn test_remove_invalid_index_child() -> Result<(), OdiliaError> {
		let cache_item_arc = default_cache_item!();
		let cache_item = cache_item_arc.lock().await;
		let child_2 = ChangeChildCommand {
				new_child: cache_item.children.get(1).unwrap().key.clone(),
				index: 2,
				add: false,
				apply_to: cache_item.object.clone(),
		};
		// this is required so that mutex can be aquired in execute
		drop(cache_item);
		let execution_return = child_2.execute(Arc::clone(&cache_item_arc)).await;
		let cache_item = cache_item_arc.lock().await;
		assert_eq!(Err(CacheError::Invalidated(cache_item.object.clone()).into()), execution_return);
		Ok(())
	}
}
