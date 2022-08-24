use rustc_hash::FxHasher;
use tokio::sync::Mutex;

type FxBuildHasher = std::hash::BuildHasherDefault<FxHasher>;
pub type FxReadHandleFactory<K, V> = evmap::ReadHandleFactory<K, V, (), FxBuildHasher>;
pub type FxWriteHandle<K, V> = evmap::WriteHandle<K, V, (), FxBuildHasher>;

/// The root of the accessible cache.
pub struct Cache {
    pub by_id_read: FxReadHandleFactory<u32, (String, String)>,
    pub by_id_write: Mutex<FxWriteHandle<u32, (String, String)>>,
}

impl Cache {
    pub fn new() -> Self {
        let (rh, wh) = evmap::with_hasher((), FxBuildHasher::default());

        Self {
            by_id_read: rh.factory(),
            by_id_write: Mutex::new(wh),
        }
    }
}
