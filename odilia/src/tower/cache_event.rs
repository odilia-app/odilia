use atspi_common::EventProperties;
use derived_deref::{Deref, DerefMut};
use odilia_cache::CacheItem;
use std::fmt::Debug;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct CacheEvent<E: EventProperties + Debug> {
	#[target]
	pub inner: E,
	pub item: CacheItem,
}
