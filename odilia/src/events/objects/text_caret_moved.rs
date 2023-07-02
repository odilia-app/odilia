use std::sync::atomic::{AtomicI32, Ordering};
use crate::{
	state::ScreenReaderState,
	traits::{IntoOdiliaCommands, StateView, Command, IntoStateView, MutableStateView, IntoMutableStateView},
};
use std::sync::Arc;
use async_trait::async_trait;
use atspi_common::events::object::{ObjectEvents, TextCaretMovedEvent};
use odilia_common::events::{ScreenReaderEvent};
use odilia_common::{
	cache::{CacheKey, ExternalCacheItem},
	errors::{OdiliaError, CacheError},
	commands::{OdiliaCommand, SetCaretPositionCommand},
};
use odilia_cache::{CacheRef, CacheValue, CacheItem};

impl MutableStateView for SetCaretPositionCommand {
	type View = Arc<AtomicI32>;
}

#[async_trait]
impl IntoMutableStateView for SetCaretPositionCommand {
	async fn create_view(&self, state: &ScreenReaderState) -> Result<<Self as MutableStateView>::View, OdiliaError> {
		Ok(Arc::clone(&state.caret_position))
	}
}

#[async_trait]
impl Command for SetCaretPositionCommand {
	async fn execute(&self, previous_pos: <Self as MutableStateView>::View) -> Result<(), OdiliaError> {
		previous_pos.store(self.new_position, Ordering::Relaxed);
		Ok(())
	}
}

#[async_trait]
impl IntoOdiliaCommands for TextCaretMovedEvent {
	// TODO: handle speaking if in an aria-live region
	async fn commands(&self, state_view: &<Self as StateView>::View) -> Result<Vec<OdiliaCommand>, OdiliaError> {
		Ok(vec![
			SetCaretPositionCommand {
				new_position: self.position,
			}.into()
		])
	}
}
