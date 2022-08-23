use std::sync::Arc;

use tokio::sync::Mutex;

/// The root of the accessible cache.
pub struct Cache {
    pub by_id_read: evmap::ReadHandleFactory<u32, (String, String)>,
    pub by_id_write: Arc<Mutex<evmap::WriteHandle<u32, (String, String)>>>,
}

impl Cache {
    pub fn new() -> Self {
        let (rh, wh) = evmap::new();

        Self {
            by_id_read: rh.factory(),
            by_id_write: Arc::new(Mutex::new(wh)),
        }
    }
}
