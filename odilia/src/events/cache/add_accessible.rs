use std::sync::Arc;
use crate::{
	state::ScreenReaderState,
	traits::{IntoOdiliaCommands, Command, StateView, MutableStateView, IntoMutableStateView},
};
use async_trait::async_trait;
use atspi_common::events::AddAccessibleEvent;


use odilia_common::{
	errors::{OdiliaError},
	commands::{OdiliaCommand, AddItemCommand},
};
use odilia_cache::{Cache};

#[async_trait]
impl IntoOdiliaCommands for AddAccessibleEvent {
	async fn commands(&self, _: &<Self as StateView>::View) -> Result<Vec<OdiliaCommand>, OdiliaError> {
		Ok(vec![
			AddItemCommand {
				item: self.node_added.clone().into()
			}.into()
		].into())
	}
}

impl MutableStateView for AddItemCommand {
	type View = Arc<Cache>;
}

#[async_trait]
impl IntoMutableStateView for AddItemCommand {
	async fn create_view(&self, state: &ScreenReaderState) -> Result<<Self as MutableStateView>::View, OdiliaError> {
		Ok(Arc::clone(&state.cache))
	}
}

#[async_trait]
impl Command for AddItemCommand {
	async fn execute(&self, cache: <Self as MutableStateView>::View) -> Result<(), OdiliaError> {
		let _ = cache.add_from_atspi_cache_item(self.item.clone()).await?;
		Ok(())
	}
}
