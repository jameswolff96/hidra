#![deny(warnings)]
//! ABI-stable protocol: device kinds, flags, message envelopes.

pub mod ioctl;

use serde::{Deserialize, Serialize};

#[repr(u16)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum DeviceKind {
    X360 = 0x0366,
    DS4 = 0x05C4,
    DS5 = 0x0CE6,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CreateDevice {
    pub kind: DeviceKind,
    pub features: u32, // bitflags: rumble,touchpad,gyro,led,...
}

pub const HIDRA_FFI_ABI_VERSION: u32 = 1;
