#[repr(u16)]
#[derive(Clone, Copy, Debug)]
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
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
