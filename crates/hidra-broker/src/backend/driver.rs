#![allow(unused)]
use super::Backend;
use anyhow::Result;
use dashmap::DashMap;
use hidra_protocol::{DeviceKind, ioctl::PadState};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tracing::debug;

pub struct Driver {
    next: AtomicU64,
    live: DashMap<u64, DeviceKind>,
}

impl Driver {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { next: AtomicU64::new(1), live: DashMap::new() })
    }
}

#[async_trait::async_trait]
impl Backend for Driver {
    async fn create(&self, kind: DeviceKind, _features: u32) -> Result<u64> {
        let h = self.next.fetch_add(1, Ordering::SeqCst);
        self.live.insert(h, kind);
        // TODO: open device interface / send IOCTL_HIDRA_CREATE
        Ok(h)
    }
    async fn destroy(&self, handle: u64) -> Result<()> {
        self.live.remove(&handle);
        // TODO: IOCTL_HIDRA_DESTROY
        Ok(())
    }
    async fn update(&self, h: u64, s: PadState) -> Result<()> {
        // TODO: choose packer by kind and call DeviceIoControl(IOCTL_HIDRA_UPDATE)
        debug!(handle = h, ?s, "driver update (stub)");
        Ok(())
    }
}
