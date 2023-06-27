use crate::{CacheKey, CacheRef};
use atspi_common::{InterfaceSet, Role, StateSet};
use odilia_common::cache::{AccessiblePrimitive, ExternalCacheItem};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// A struct representing an accessible. To get any information from the cache other than the stored information like role, interfaces, and states, you will need to instantiate an [`atspi_proxies::accessible::AccessibleProxy`] or other `*Proxy` type from atspi to query further info.
pub struct CacheItem {
	/// The accessible object (within the application)	(so)
	pub object: AccessiblePrimitive,
	/// The application (root object(?)	  (so)
	pub app: AccessiblePrimitive,
	/// The parent object.  (so)
	pub parent: CacheRef,
	/// The accessbile index in parent.	i
	pub index: i32,
	/// Child count of the accessible  i
	pub children_num: i32,
	/// The exposed interfece(s) set.  as
	pub interfaces: InterfaceSet,
	/// Accessible role. u
	pub role: Role,
	/// The states applicable to the accessible.  au
	pub states: StateSet,
	/// The text of the accessible.
	pub text: String,
	/// The children (ids) of the accessible.
	pub children: Vec<CacheRef>,
}

impl From<CacheItem> for ExternalCacheItem {
	fn from(ci: CacheItem) -> ExternalCacheItem {
		ExternalCacheItem {
			object: ci.object.clone(),
			app: ci.app.clone(),
			parent: ci.parent.key,
			index: ci.index,
			children_num: ci.children_num,
			interfaces: ci.interfaces,
			role: ci.role,
			states: ci.states,
			text: ci.text.clone(),
			children: ci.children.into_iter().map(|cache_ref| cache_ref.key).collect(),
		}
	}
}
