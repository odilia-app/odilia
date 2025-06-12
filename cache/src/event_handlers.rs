use std::{
	marker::PhantomData,
	ops::{Deref, DerefMut},
};

use atspi::Event;
use static_assertions::assert_impl_all;

use crate::{CacheItem, CacheKey, OdiliaError, RelationSet, RelationType, Relations};

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
