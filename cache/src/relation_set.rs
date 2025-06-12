use serde::{Deserialize, Serialize};

use crate::{Cache, CacheDriver, CacheItem, CacheKey, ObjectRef, OdiliaError, RelationType};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct RelationSet(Vec<(RelationType, Vec<ObjectRef>)>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relations(pub RelationType, pub Vec<CacheItem>);

impl Relations {
	async fn fill_with<D: CacheDriver>(
		ors: RelationSet,
		rt: RelationType,
		cache: &mut Cache<D>,
	) -> Result<Relations, OdiliaError> {
		let rts: Vec<CacheKey> = ors.get_relations(rt).map(Into::into).collect();
		let mut rts2 = Vec::new();
		for key in rts {
			let item = cache.get_or_create(&key).await?;
			rts2.push(item);
		}
		Ok(Relations(rt, rts2))
	}
}

impl RelationSet {
	pub fn get_relations(self, rt: RelationType) -> impl Iterator<Item = ObjectRef> {
		self.0.into_iter()
			.filter_map(move |(ty, ci)| if ty == rt { Some(ci) } else { None })
			.flatten()
	}
	pub fn label_for(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::LabelFor)
	}
	pub fn labelled_by(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::LabelledBy)
	}
	pub fn controller_for(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::ControllerFor)
	}
	pub fn controlled_by(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::ControlledBy)
	}
	pub fn member_of(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::MemberOf)
	}
	pub fn tooltip_for(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::TooltipFor)
	}
	pub fn children_of(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::NodeChildOf)
	}
	pub fn parent_of(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::NodeParentOf)
	}
	pub fn extended(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::Extended)
	}
	pub fn flows_to(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::FlowsTo)
	}
	pub fn flows_from(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::FlowsFrom)
	}
	pub fn subwindow_of(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::SubwindowOf)
	}
	pub fn embeds(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::Embeds)
	}
	pub fn embedded_by(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::EmbeddedBy)
	}
	pub fn popup_for(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::PopupFor)
	}
	pub fn parent_window_of(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::ParentWindowOf)
	}
	pub fn described_by(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::DescribedBy)
	}
	pub fn description_for(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::DescriptionFor)
	}
	pub fn details(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::Details)
	}
	pub fn details_for(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::DetailsFor)
	}
	pub fn error_message(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::ErrorMessage)
	}
	pub fn error_for(self) -> impl Iterator<Item = ObjectRef> {
		self.get_relations(RelationType::ErrorFor)
	}
}
