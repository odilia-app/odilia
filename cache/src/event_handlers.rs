use std::ops::{Deref, DerefMut};

use atspi::{
	events::{
		cache::{AddAccessibleEvent, LegacyAddAccessibleEvent, RemoveAccessibleEvent},
		document::{
			self, ContentChangedEvent, LoadCompleteEvent, LoadStoppedEvent,
			PageChangedEvent, ReloadEvent,
		},
		focus::FocusEvent,
		keyboard::ModifiersEvent,
		mouse::{AbsEvent, ButtonEvent, RelEvent},
		object::{
			ActiveDescendantChangedEvent, AnnouncementEvent, AttributesChangedEvent,
			BoundsChangedEvent, ChildrenChangedEvent, ColumnDeletedEvent,
			ColumnInsertedEvent, ColumnReorderedEvent, ModelChangedEvent, Property,
			PropertyChangeEvent, RowDeletedEvent, RowInsertedEvent, RowReorderedEvent,
			StateChangedEvent, TextAttributesChangedEvent, TextCaretMovedEvent,
			TextChangedEvent, TextSelectionChangedEvent, VisibleDataChangedEvent,
		},
		terminal::{
			ApplicationChangedEvent, CharWidthChangedEvent, ColumnCountChangedEvent,
			LineChangedEvent, LineCountChangedEvent,
		},
		window::{
			ActivateEvent, CloseEvent, CreateEvent, DeactivateEvent,
			DesktopCreateEvent, DesktopDestroyEvent, DestroyEvent, LowerEvent,
			MaximizeEvent, MinimizeEvent, MoveEvent, RaiseEvent, ReparentEvent,
			ResizeEvent, RestoreEvent, RestyleEvent, ShadeEvent, UUshadeEvent,
		},
		CacheEvents, Event, ObjectEvents,
	},
	DocumentEvents, KeyboardEvents, MouseEvents, Operation, State, TerminalEvents,
	WindowEvents,
};

use crate::{
	Cache, CacheDriver, CacheError, CacheItem, CacheKey, Future, OdiliaError, RelationType,
	Relations,
};

pub trait ConstRelationType {
	const RELATION_TYPE: RelationType;
	type InnerStore;
}

//pub struct Relations<T: ConstRelationType>(Vec<CacheItem>, T);
//impl<T: ConstRelationType> Deref for Relations<T> {
//    type Target = Vec<CacheItem>;
//
//    fn deref(&self) -> &Self::Target {
//        &self.0
//    }
//}
//impl<T: ConstRelationType> DerefMut for Relations<T> {
//    fn deref_mut(&mut self) -> &mut Self::Target {
//        &mut self.0
//    }
//}
//impl<T: ConstRelationType> RequestExt for Relations<T> {
//    const REQUEST: CacheRequest = CacheRequest::Relation(T::RELATION_TYPE);
//}

#[derive(Debug)]
pub struct Item(pub CacheItem);
impl Deref for Item {
	type Target = CacheItem;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for Item {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Debug)]
pub struct Parent(pub CacheItem);

#[derive(Debug)]
pub struct Children(pub Vec<CacheItem>);
impl Deref for Children {
	type Target = Vec<CacheItem>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for Children {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Debug)]
pub enum CacheRequest {
	Item(CacheKey),
	Parent(CacheKey),
	Children(CacheKey),
	Relation(CacheKey, RelationType),
	EventHandler(Box<Event>),
}

#[derive(Debug)]
pub enum CacheResponse {
	Item(Item),
	Parent(Parent),
	Children(Children),
	Relations(Relations),
}

macro_rules! impl_relation {
	($name:ident, $relation_type:expr) => {
		pub struct $name;
		impl ConstRelationType for $name {
			const RELATION_TYPE: RelationType = $relation_type;
			type InnerStore = Vec<CacheItem>;
		}
	};
}

impl_relation!(ControllerFor, RelationType::ControllerFor);
impl_relation!(ControlledBy, RelationType::ControlledBy);
impl_relation!(LabelFor, RelationType::LabelFor);
impl_relation!(LabelledBy, RelationType::LabelledBy);
impl_relation!(DescribedBy, RelationType::DescribedBy);
impl_relation!(DescriptionFor, RelationType::DescriptionFor);
impl_relation!(Details, RelationType::Details);
impl_relation!(DetailsFor, RelationType::DetailsFor);
impl_relation!(ErrorMessage, RelationType::ErrorMessage);
impl_relation!(ErrorFor, RelationType::ErrorFor);
impl_relation!(FlowsTo, RelationType::FlowsTo);
impl_relation!(FlowsFrom, RelationType::FlowsFrom);
impl_relation!(Embeds, RelationType::Embeds);
impl_relation!(EmbeddedBy, RelationType::EmbeddedBy);
impl_relation!(PopupFor, RelationType::PopupFor);
impl_relation!(ParentWindowOf, RelationType::ParentWindowOf);
impl_relation!(SubwindowOf, RelationType::SubwindowOf);
impl_relation!(MemberOf, RelationType::MemberOf);
impl_relation!(NodeChildOf, RelationType::NodeChildOf);
impl_relation!(NodeParentOf, RelationType::NodeParentOf);

//assert_impl_all!(Parent: RequestExt);
//assert_impl_all!(Children: RequestExt);
//assert_impl_all!(Item: RequestExt);
//assert_impl_all!(Application: RequestExt);
//assert_impl_all!(Relations: RequestExt);

/// Implemented for all events which modify items in the cache in some way.
pub trait EventHandler {
	/// How does this event affect the cache.
	/// It might remove links between cache items, remove them completely, add new ones, interact the the [`CacheDriver`] directly.
	/// The only thing that is required by implementers of this function is that the cache remains in some generally consistent state;
	/// for example, if you are removing a child from its parent, you must remove the relationship on both sides.
	///
	/// Generally this should be implemented by those that understand the invariants of the accessibility tree.
	fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> impl Future<Output = Result<CacheItem, OdiliaError>> + Send;
}

impl EventHandler for PropertyChangeEvent {
	async fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> Result<CacheItem, OdiliaError> {
		let key = self.item.into();
		cache.modify_if_not_new(&key, |item: &mut CacheItem| {
			match self.value {
				Property::Role(role) => {
					item.role = role;
				}
				Property::Name(name) => {
					item.name = Some(name);
				}
				Property::Description(description) => {
					item.description = Some(description);
				}
				Property::Parent(parent) => {
					// TODO: does a separate event come in adding the current item to its children?
					item.parent = parent.into();
				}
				prop => {
					tracing::error!(
						"Unable to process property vvariant {prop:?}"
					);
				}
			}
		})
		.await
	}
}

impl EventHandler for StateChangedEvent {
	async fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> Result<CacheItem, OdiliaError> {
		let key = self.item.into();
		cache.modify_if_not_new(&key, |item: &mut CacheItem| {
			if self.enabled {
				item.states.insert(self.state);
			} else {
				item.states.remove(self.state);
			}
		})
		.await
	}
}
impl EventHandler for TextChangedEvent {
	async fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> Result<CacheItem, OdiliaError> {
		let key = self.item.into();
		let start: usize = self
			.start_pos
			.try_into()
			.expect("Positive index for text insertion/deletion");
		let len: usize = self
			.length
			.try_into()
			.expect("Positive length for text insertion/deletion");
		cache.modify_if_not_new(&key, |item: &mut CacheItem| {
        match (self.operation, item.text.as_mut()) {
            (Operation::Insert, Some(text)) => {
                let (before,after): (Vec<(usize, char)>, Vec<(usize, char)>) = text.char_indices()
                    .partition(|(i,_c)| *i < start);
                let new_text = before
                    .into_iter()
                    .map(|(_i,c)| c)
                    .chain(self.text.chars())
                    .chain(after.into_iter().map(|(_i,c)| c))
                    .collect::<String>();
                *text = new_text;
            },
            (Operation::Delete, Some(text)) => {
                let new_text = text.char_indices()
                    .filter_map(|(i,c)| if i < start || i >= start+len { Some(c) } else { None })
                    .collect::<String>();
                *text = new_text;
            },
            (Operation::Insert, None) => {
                tracing::error!("AT-SPI requested an insertion of text at index > 0, but there is currently no text to insert into!");
                item.text = Some(self.text);
            },
            (Operation::Delete, None) => {
                tracing::error!("AT-SPI requested us to delete text from an item with no text in it!");
            },
        }
    }).await
	}
}

// Pre-fetches the entire application's worth of data upon load complete.
impl EventHandler for LoadCompleteEvent {
	async fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> Result<CacheItem, OdiliaError> {
		let key: CacheKey = self.item.into();
		cache.prefetch_app(&key).await
	}
}

impl EventHandler for ChildrenChangedEvent {
	async fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> Result<CacheItem, OdiliaError> {
		let key = self.item.into();
		let index = self
			.index_in_parent
			.try_into()
			.expect("Positive index for child insertion/deletion");
		cache.modify_if_not_new(&key, |cache_item: &mut CacheItem| match self.operation {
			Operation::Insert => {
				cache_item.children.insert(index, self.child.into());
			}
			Operation::Delete => {
				cache_item.children.remove(index);
			}
		})
		.await
	}
}

macro_rules! impl_empty_event_handler {
	($event:ty) => {
		impl EventHandler for $event {
			async fn handle_event<D: CacheDriver + Send>(
				self,
				cache: &mut Cache<D>,
			) -> Result<CacheItem, OdiliaError> {
				let key = self.item.into();
				cache.get_or_create(&key).await
			}
		}
	};
}

impl_empty_event_handler!(AttributesChangedEvent);
impl_empty_event_handler!(BoundsChangedEvent);
impl_empty_event_handler!(VisibleDataChangedEvent);
impl_empty_event_handler!(TextCaretMovedEvent);
impl_empty_event_handler!(TextAttributesChangedEvent);
impl_empty_event_handler!(RowInsertedEvent);
impl_empty_event_handler!(RowDeletedEvent);
impl_empty_event_handler!(RowReorderedEvent);
impl_empty_event_handler!(ColumnInsertedEvent);
impl_empty_event_handler!(ColumnDeletedEvent);
impl_empty_event_handler!(ColumnReorderedEvent);
impl_empty_event_handler!(ModelChangedEvent);
impl_empty_event_handler!(ActiveDescendantChangedEvent);
impl_empty_event_handler!(AnnouncementEvent);
impl_empty_event_handler!(TextSelectionChangedEvent);
impl_empty_event_handler!(ReloadEvent);
impl_empty_event_handler!(LoadStoppedEvent);
impl_empty_event_handler!(ContentChangedEvent);
impl_empty_event_handler!(document::AttributesChangedEvent);
impl_empty_event_handler!(PageChangedEvent);
impl_empty_event_handler!(MinimizeEvent);
impl_empty_event_handler!(MaximizeEvent);
impl_empty_event_handler!(RestoreEvent);
impl_empty_event_handler!(CloseEvent);
impl_empty_event_handler!(CreateEvent);
impl_empty_event_handler!(ReparentEvent);
impl_empty_event_handler!(DesktopCreateEvent);
impl_empty_event_handler!(DesktopDestroyEvent);
impl_empty_event_handler!(DestroyEvent);
impl_empty_event_handler!(ActivateEvent);
impl_empty_event_handler!(DeactivateEvent);
impl_empty_event_handler!(RaiseEvent);
impl_empty_event_handler!(LowerEvent);
impl_empty_event_handler!(MoveEvent);
impl_empty_event_handler!(ResizeEvent);
impl_empty_event_handler!(ShadeEvent);
impl_empty_event_handler!(UUshadeEvent);
impl_empty_event_handler!(RestyleEvent);
impl_empty_event_handler!(LineChangedEvent);
impl_empty_event_handler!(ColumnCountChangedEvent);
impl_empty_event_handler!(LineCountChangedEvent);
impl_empty_event_handler!(CharWidthChangedEvent);
impl_empty_event_handler!(ApplicationChangedEvent);
impl_empty_event_handler!(AbsEvent);
impl_empty_event_handler!(RelEvent);
impl_empty_event_handler!(ButtonEvent);
impl_empty_event_handler!(ModifiersEvent);

/// Implemented for applications which emit the legacy [`FocusEvent`] event type.
/// This uses the same implementation as the [`StateChangedEvent`] [`EventHandler`] impl for the `Focused` state set to true.
impl EventHandler for FocusEvent {
	async fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> Result<CacheItem, OdiliaError> {
		let key = self.item.into();
		cache.modify_if_not_new(&key, |item: &mut CacheItem| {
			item.states.insert(State::Focused);
		})
		.await
	}
}

impl EventHandler for AddAccessibleEvent {
	async fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> Result<CacheItem, OdiliaError> {
		cache.get_or_create_from_cache_item(self.node_added).await
	}
}
impl EventHandler for RemoveAccessibleEvent {
	async fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> Result<CacheItem, OdiliaError> {
		let key = self.item.into();
		let item = cache.remove(&key);
		item.ok_or(CacheError::NoItem.into())
	}
}
impl EventHandler for LegacyAddAccessibleEvent {
	async fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> Result<CacheItem, OdiliaError> {
		cache.get_or_create_from_legacy_cache_item(self.node_added).await
	}
}

impl EventHandler for Event {
	#[allow(clippy::too_many_lines)]
	async fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> Result<CacheItem, OdiliaError> {
		match self {
			Event::Object(ObjectEvents::PropertyChange(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::StateChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::TextChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::AttributesChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::BoundsChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::VisibleDataChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::TextCaretMoved(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::TextAttributesChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::RowInserted(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::RowDeleted(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::RowReordered(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::ColumnInserted(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::ColumnDeleted(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::ColumnReordered(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::ModelChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::ActiveDescendantChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::Announcement(event)) => {
				event.handle_event(cache).await
			}
			Event::Object(ObjectEvents::TextSelectionChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Cache(CacheEvents::Add(event)) => event.handle_event(cache).await,
			Event::Cache(CacheEvents::LegacyAdd(event)) => {
				event.handle_event(cache).await
			}
			Event::Cache(CacheEvents::Remove(event)) => event.handle_event(cache).await,
			Event::Document(DocumentEvents::LoadComplete(event)) => {
				event.handle_event(cache).await
			}
			Event::Document(DocumentEvents::Reload(event)) => {
				event.handle_event(cache).await
			}
			Event::Document(DocumentEvents::LoadStopped(event)) => {
				event.handle_event(cache).await
			}
			Event::Document(DocumentEvents::ContentChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Document(DocumentEvents::AttributesChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Document(DocumentEvents::PageChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Minimize(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Maximize(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Restore(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Close(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Create(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Reparent(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::DesktopCreate(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::DesktopDestroy(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Destroy(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Activate(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Deactivate(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Raise(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Lower(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Move(event)) => event.handle_event(cache).await,
			Event::Window(WindowEvents::Resize(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Shade(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::UUshade(event)) => {
				event.handle_event(cache).await
			}
			Event::Window(WindowEvents::Restyle(event)) => {
				event.handle_event(cache).await
			}
			Event::Terminal(TerminalEvents::LineChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Terminal(TerminalEvents::ColumnCountChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Terminal(TerminalEvents::LineCountChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Terminal(TerminalEvents::ApplicationChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Terminal(TerminalEvents::CharWidthChanged(event)) => {
				event.handle_event(cache).await
			}
			Event::Mouse(MouseEvents::Abs(event)) => event.handle_event(cache).await,
			Event::Mouse(MouseEvents::Rel(event)) => event.handle_event(cache).await,
			Event::Mouse(MouseEvents::Button(event)) => event.handle_event(cache).await,
			Event::Keyboard(KeyboardEvents::Modifiers(event)) => {
				event.handle_event(cache).await
			}
			ev => {
				tracing::error!("Unable to handle event: {ev:?}");
				Err(OdiliaError::Generic(format!("Unable to handle event: {ev:?}")))
			}
		}
	}
}
