#![deny(warnings)]
#![allow(unused)]
//! ABI-stable protocol: device kinds, flags, message envelopes.

use serde::{Deserialize, Serialize};

pub mod report;

#[repr(u16)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum DeviceKind {
    X360 = 0x0366,
    DS4 = 0x05C4,
    DS5 = 0x0CE6,
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct Features: u32 {
        const RUMBLE  = 1 << 0;
        const TOUCH   = 1 << 1;
        const GYRO    = 1 << 2;
        const LED     = 1 << 3;
    }
}

const _: [(); 14] = [(); size_of::<PadState>()];
const _: [(); 2] = [(); align_of::<PadState>()];

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PadState {
    pub buttons: u16,
    pub lx: i16,
    pub ly: i16,
    pub rx: i16,
    pub ry: i16,
    pub lt: u16,
    pub rt: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CreateIn {
    pub kind: DeviceKind,
    pub features: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CreateOut {
    pub handle: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DestroyIn {
    pub handle: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateIn {
    pub handle: u64,
    pub state: PadState,
}

const FILE_DEVICE_UNKNOWN: u32 = 0x0000_0022;
const FILE_ANY_ACCESS: u32 = 0x0000;
const FILE_READ_ACCESS: u32 = 0x0001;
const FILE_WRITE_ACCESS: u32 = 0x0002;
const METHOD_BUFFERED: u32 = 0;
const METHOD_IN_DIRECT: u32 = 1;
const METHOD_OUT_DIRECT: u32 = 2;
const METHOD_NEITHER: u32 = 3;

const fn ctl_code(device_type: u32, function: u32, method: u32, access: u32) -> u32 {
    (device_type << 16) | (access << 14) | (function << 2) | method
}

pub const HIDRA_DEVICE_TYPE: u32 = FILE_DEVICE_UNKNOWN;
pub const HIDRA_IOCTL_BASE: u32 = 0x800;

#[allow(clippy::identity_op)]
pub const IOCTL_HIDRA_CREATE: u32 =
    ctl_code(HIDRA_DEVICE_TYPE, HIDRA_IOCTL_BASE + 0, METHOD_BUFFERED, FILE_WRITE_ACCESS);
pub const IOCTL_HIDRA_UPDATE: u32 =
    ctl_code(HIDRA_DEVICE_TYPE, HIDRA_IOCTL_BASE + 1, METHOD_BUFFERED, FILE_WRITE_ACCESS);
pub const IOCTL_HIDRA_DESTROY: u32 =
    ctl_code(HIDRA_DEVICE_TYPE, HIDRA_IOCTL_BASE + 2, METHOD_BUFFERED, FILE_WRITE_ACCESS);

#[cfg(feature = "backend-driver")]
pub const HIDRA_INTERFACE_GUID: windows::core::GUID =
    windows::core::GUID::from_u128(0x11111111_1111_1111_1111_111111111111);

pub const HIDRA_FFI_ABI_VERSION: u32 = 1;
