#![deny(warnings)]
use anyhow::Result;
use hidra_protocol::{CreateDevice, DeviceKind};

#[derive(Debug, Clone, Copy)]
pub struct GamepadHandle(u64);

pub async fn spawn(kind: DeviceKind) -> Result<GamepadHandle> {
    // TODO: IPC to broker; for now, pretend success
    let _req = CreateDevice { kind, features: 0 };
    Ok(GamepadHandle(1))
}

pub async fn set_rumble(h: GamepadHandle, low: u8, high: u8) -> Result<()> {
    // TODO: IPC send
    println!("Set rumble: low {}, high {}, gamepad {}", low, high, h.0);
    Ok(())
}
