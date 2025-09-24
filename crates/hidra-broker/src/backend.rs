pub mod mock;

use anyhow::Result;
use hidra_protocol::{DeviceKind, ioctl::PadState};

#[async_trait::async_trait]
pub trait Backend: Send + Sync + 'static {
    async fn create(&self, kind: DeviceKind, features: u32) -> Result<u64>;
    async fn destroy(&self, handle: u64) -> Result<()>;
    async fn update(&self, handle: u64, state: PadState) -> Result<()>;
}
