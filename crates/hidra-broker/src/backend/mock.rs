use super::Backend;
use anyhow::Result;
use dashmap::DashMap;
use hidra_protocol::{
    DeviceKind, PadState,
    report::{DS4Report, DS5Report, X360Report},
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tracing::debug;

pub struct Mock {
    next: AtomicU64,
    live: DashMap<u64, Info>,
}

#[derive(Clone, Copy)]
struct Info {
    kind: DeviceKind,
    #[allow(dead_code)] //TODO: remove once features are implemented
    features: u32,
}

impl Mock {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { next: AtomicU64::new(1), live: DashMap::new() })
    }
}

#[async_trait::async_trait]
impl Backend for Mock {
    async fn create(&self, kind: DeviceKind, features: u32) -> Result<u64> {
        let h = self.next.fetch_add(1, Ordering::SeqCst);
        self.live.insert(h, Info { kind, features });
        Ok(h)
    }

    async fn destroy(&self, handle: u64) -> Result<()> {
        self.live.remove(&handle);
        Ok(())
    }

    async fn update(&self, h: u64, s: PadState) -> Result<()> {
        if let Some(info) = self.live.get(&h) {
            match info.kind {
                DeviceKind::X360 => {
                    let rpt = X360Report::from(&s);
                    debug!(handle = h, ?s, report = ?rpt, "mock update X360");
                }
                DeviceKind::DS4 => {
                    let rpt = DS4Report::from(&s);
                    debug!(handle = h, ?s, report = ?rpt, "mock update DS4");
                }
                DeviceKind::DS5 => {
                    let rpt = DS5Report::from(&s);
                    debug!(handle = h, ?s, report = ?rpt, "mock update DS5");
                }
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("invalid handle: {}", h))
        }
    }
}
