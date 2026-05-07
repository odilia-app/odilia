use serde::{Deserialize, Serialize};

use crate::{CacheItem, ObjectRefOwned, RelationType};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct RelationSet(Vec<(RelationType, Vec<ObjectRefOwned>)>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relations(pub RelationType, pub Vec<CacheItem>);

impl RelationSet {
	pub fn get_relations(self, rt: RelationType) -> impl Iterator<Item = ObjectRefOwned> {
		self.0.into_iter()
			.filter_map(move |(ty, ci)| if ty == rt { Some(ci) } else { None })
			.flatten()
	}
}
