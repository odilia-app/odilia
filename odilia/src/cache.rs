use rustc_hash::FxHasher;
use tokio::sync::Mutex;
use std::sync::Arc;
use atspi::{
	accessible::Role,
	InterfaceSet,
	StateSet,
};
use evmap::ShallowCopy;
use std::mem::ManuallyDrop;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct CacheItem {
    // The accessible object (within the application)   (so)
    pub object: i32,
    // The application (root object(?)    (so)
    pub app: i32,
    // The parent object.  (so)
    pub parent: i32,
    // The accessbile index in parent.  i
    pub index: i32,
    // Child count of the accessible  i
    pub children: i32,
    // The exposed interfece(s) set.  as
    pub ifaces: InterfaceSet,
    // Accessible role. u
    pub role: Role,
    // The states applicable to the accessible.  au
    pub states: StateSet,
}
impl ShallowCopy for CacheItem {
	unsafe fn shallow_copy(&self) -> ManuallyDrop<Self> {
		ManuallyDrop::new(*self)
	}
}

type FxBuildHasher = std::hash::BuildHasherDefault<FxHasher>;
pub type FxReadHandleFactory<K, V> = evmap::ReadHandleFactory<K, V, (), FxBuildHasher>;
pub type FxWriteHandle<K, V> = evmap::WriteHandle<K, V, (), FxBuildHasher>;

/// The root of the accessible cache.
pub struct Cache {
    pub by_id_read: FxReadHandleFactory<i32, CacheItem>,
    pub by_id_write: Mutex<FxWriteHandle<i32, CacheItem>>,
}

impl Cache {
    pub fn new() -> Self {
        let (rh, wh) = evmap::with_hasher((), FxBuildHasher::default());

        Self { by_id_read: rh.factory(), by_id_write: Mutex::new(wh) }
    }
}
