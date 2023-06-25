use crate::{
	state::ScreenReaderState,
	traits::{IntoOdiliaCommands, IntoStateView, Command, StateView},
};
use atspi_common::events::object::StateChangedEvent;
use atspi_common::State;
use odilia_common::events::{ScreenReaderEvent};
use odilia_common::{
	cache::ExternalCacheItem,
	errors::{OdiliaError, CacheError},
	commands::{OdiliaCommand, SetStateCommand},
};
use odilia_cache::{CacheRef, CacheValue, CacheItem};

impl IntoStateView for StateChangedEvent {
	fn create_view(&self, state: &ScreenReaderState) -> Result<<Self as StateView>::View, OdiliaError> {
		Ok(state.cache.get_from(&self.item)?.into())
	}
}

impl IntoOdiliaCommands for StateChangedEvent {
	fn commands(&self, state_view: &<Self as StateView>::View) -> Result<Vec<OdiliaCommand>, OdiliaError> {
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
