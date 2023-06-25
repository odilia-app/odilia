//! Cache Types
//!
//! Provides basic types for caching, sets key and value types, the hashing method to use, and which synchronization primitives to wrap values in.
//! These are the values used within the Odilia host.
//! And most of these values, if they need to cross FFI boundaries will need to be converted to simpler types.

use crate::CacheItem;
use std::sync::{Arc, Weak};
use fxhash::FxBuildHasher;
use dashmap::DashMap;
//use tokio::sync::Mutex;
use tokio::sync::{Mutex, RwLock};
use odilia_common::cache::{CacheKey};

/// This is the type alis refeering to the value for all cache items.
/// This includes thread-safe and concurrency-safe wrappers.
pub type CacheValue = Arc<Mutex<CacheItem>>;
/// This is the type alis refereing to a weak version of the value for all cache items.
/// This can be upgraded to a [`CacheValue`] with `.upgrade()`, where it may or may not be found.
pub type WeakCacheValue = Weak<Mutex<CacheItem>>;
/// The `InnerCache` type alias defines the data structure to be used to hold the entire cache.
pub type InnerCache = DashMap<CacheKey, CacheValue, FxBuildHasher>;
/// A wrapped [`InnerCache`] in a thread-safe type.
pub type ThreadSafeCache = Arc<RwLock<InnerCache>>;
