use crate::ScreenReaderState;
use atspi::events::{AddAccessibleEvent, CacheEvents, RemoveAccessibleEvent};
use odilia_common::events::{ScreenReaderEvent, CacheEvent};

pub async fn dispatch(state: &ScreenReaderState, event: &CacheEvents) -> Vec<ScreenReaderEvent> {
	vec![match event {
		CacheEvents::Add(add_event) => ScreenReaderEvent::Cache(
      CacheEvent::AddItem(
        (add_event.item.name.to_string(), add_event.item.path.clone())
      )
    ),
		CacheEvents::Remove(rem_event) => ScreenReaderEvent::Cache(
      CacheEvent::RemoveItem(
        (rem_event.item.name.to_string(), rem_event.item.path.clone())
      )
    ),
	}]
}
