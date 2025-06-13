use std::{
	marker::PhantomData,
	ops::{Deref, DerefMut},
};

use atspi::{
	events::object::{Property, PropertyChangeEvent, StateChangedEvent},
	Event,
};
use static_assertions::assert_impl_all;

use crate::{
	Cache, CacheDriver, CacheItem, CacheKey, Future, OdiliaError, RelationSet, RelationType,
	Relations,
};

pub trait ConstRelationType {
	const RELATION_TYPE: RelationType;
	type InnerStore;
}

pub struct Relationships<T: ConstRelationType>(pub T::InnerStore);

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
	EventHandler(Event),
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
	) -> impl Future<Output = Result<(), OdiliaError>> + Send;
}

impl EventHandler for PropertyChangeEvent {
	async fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> Result<(), OdiliaError> {
		let key = self.item.into();
		cache.get_or_create(&key).await?;
		let mut item = cache
			.tree
			.get_mut(&key)
			// NOTE: this is okay because we just placed the item in the cache on the line above.
			.unwrap();
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
				tracing::error!("Unable to process property vvariant {prop:?}");
			}
		}
		Ok(())
	}
}

impl EventHandler for StateChangedEvent {
	async fn handle_event<D: CacheDriver + Send>(
		self,
		cache: &mut Cache<D>,
	) -> Result<(), OdiliaError> {
		let key = self.item.into();
		cache.get_or_create(&key).await?;
		let mut item = cache
			.tree
			.get_mut(&key)
			// NOTE: this is okay because we just placed the item in the cache on the line above.
			.unwrap();
		if self.enabled {
			item.states.insert(self.state);
		} else {
			item.states.remove(self.state);
		}
		Ok(())
	}
}
