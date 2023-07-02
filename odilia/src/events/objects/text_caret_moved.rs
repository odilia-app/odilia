use std::sync::atomic::{AtomicI32, Ordering};
use crate::{
	state::ScreenReaderState,
	traits::{IntoOdiliaCommands, StateView, Command, MutableStateView, IntoMutableStateView},
};
use std::sync::Arc;
use async_trait::async_trait;
use atspi_common::events::object::{TextCaretMovedEvent};

use odilia_common::{
	errors::{OdiliaError},
	commands::{OdiliaCommand, SetCaretPositionCommand},
};


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
	async fn commands(&self, _state_view: &<Self as StateView>::View) -> Result<Vec<OdiliaCommand>, OdiliaError> {
		Ok(vec![
			SetCaretPositionCommand {
				new_position: self.position,
			}.into()
		])
	}
}
