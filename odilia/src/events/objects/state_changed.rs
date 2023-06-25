use crate::{
	state::ScreenReaderState,
	traits::{IntoOdiliaCommands, IntoStateView, Command, StateView, MutableStateView, IntoMutableStateView},
};
use async_trait::async_trait;
use atspi_common::events::object::StateChangedEvent;
use atspi_common::State;
use odilia_common::events::{ScreenReaderEvent};
use odilia_common::{
	cache::ExternalCacheItem,
	errors::{OdiliaError, CacheError},
	commands::{OdiliaCommand, SetStateCommand},
};
use odilia_cache::{CacheRef, CacheValue, CacheItem};

#[async_trait]
impl IntoStateView for StateChangedEvent {
	async fn create_view(&self, state: &ScreenReaderState) -> Result<<Self as StateView>::View, OdiliaError> {
		Ok(state.cache.get_from(&self.item).await?.into())
	}
}

#[async_trait]
impl IntoOdiliaCommands for StateChangedEvent {
	async fn commands(&self, state_view: &<Self as StateView>::View) -> Result<Vec<OdiliaCommand>, OdiliaError> {
		let state_non_string: State = serde_plain::from_str(&self.state)?;
		let mut new_states = state_view.states;
		if self.enabled == 1 {
			new_states.insert(state_non_string);
		} else {
			new_states.remove(state_non_string);
		}
		Ok(vec![
			SetStateCommand {
				new_states,
				apply_to: state_view.object.clone(),
			}.into()
		])
	}
}

impl MutableStateView for SetStateCommand {
	type View = CacheValue;
}

#[async_trait]
impl IntoMutableStateView for SetStateCommand {
	async fn create_view(&self, state: &ScreenReaderState) -> Result<<Self as MutableStateView>::View, OdiliaError> {
		state.cache.get_ref(&self.apply_to)
			.ok_or(CacheError::NoItem.into())
	}
}

#[async_trait]
impl Command for SetStateCommand {
	async fn execute(&self, state_view: <Self as MutableStateView>::View) -> Result<(), OdiliaError> {
		let mut item = state_view.lock().await;
		item.states = self.new_states;
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use std::sync::Arc;
	use tokio::sync::Mutex;
	use crate::traits::{IntoOdiliaCommands, Command};
	use odilia_common::{
		cache::{AccessiblePrimitive, CacheKey, ExternalCacheItem},
		commands::{OdiliaCommand, SetStateCommand},
		errors::OdiliaError,
	};
	use odilia_cache::{CacheRef, CacheValue, CacheItem};
	use atspi_common::{
		StateSet, InterfaceSet, Role, State,
		events::{Accessible, object::StateChangedEvent},
	};

	macro_rules! arc_rw {
		($id:ident) => {
			Arc::new(RwLock::new($id))
		}
	}
	// TODO: remove when default is merged upstream
	macro_rules! default_cache_item {
		() => {
			Arc::new(Mutex::new(CacheItem {
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
				key: Mutex::lock(&*$cache_item).await.object.clone(),
				item: Arc::downgrade(&$cache_item),
			}
		}
	}
	macro_rules! text_state {
		($test_name:ident, $event:expr, $start_states:expr, $new_state:expr) => {
			#[tokio::test]
			async fn $test_name() -> Result<(), OdiliaError> {
				let cache_item_arc = default_cache_item!();
				let mut cache_item = cache_item_arc.lock().await.clone();
				cache_item.states = $start_states;
				let event = $event;
				let first_command: SetStateCommand = event.commands(&cache_item.into()).await?[0].clone().try_into()?;
				assert_eq!(first_command.new_states, $new_state);
				Ok(())
			}
		}
	}

	text_state!(
		test_state_add, 
		StateChangedEvent {
			state: "focused".to_string(),
			item: Accessible::default(),
			enabled: 1,
		},
		StateSet::empty(),
		StateSet::from(State::Focused)
	);
	text_state!(
		test_state_remove_existing, 
		StateChangedEvent {
			state: "focused".to_string(),
			item: Accessible::default(),
			enabled: 0,
		},
		StateSet::empty(),
		StateSet::empty()
	);
	text_state!(
		test_state_add_existing, 
		StateChangedEvent {
			state: "focused".to_string(),
			item: Accessible::default(),
			enabled: 1,
		},
		StateSet::from(State::Focused),
		StateSet::from(State::Focused)
	);
	text_state!(
		test_state_remove, 
		StateChangedEvent {
			state: "focused".to_string(),
			item: Accessible::default(),
			enabled: 0,
		},
		StateSet::from(State::Focused),
		StateSet::empty()
	);
}
