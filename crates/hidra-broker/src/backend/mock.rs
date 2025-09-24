use super::Backend;
use anyhow::Result;
use dashmap::DashMap;
use hidra_protocol::{DeviceKind, ioctl::PadState};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub struct Mock {
    next: AtomicU64,
    live: DashMap<u64, DeviceKind>,
}

impl Mock {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { next: AtomicU64::new(1), live: DashMap::new() })
    }
}

#[async_trait::async_trait]
impl Backend for Mock {
    async fn create(&self, kind: DeviceKind, _features: u32) -> Result<u64> {
        let h = self.next.fetch_add(1, Ordering::SeqCst);
        self.live.insert(h, kind);
        Ok(h)
    }
    async fn destroy(&self, handle: u64) -> Result<()> {
        self.live.remove(&handle);
        Ok(())
    }
    async fn update(&self, _h: u64, _s: PadState) -> Result<()> {
        Ok(())
    }
}
