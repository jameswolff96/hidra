use super::Backend;
use anyhow::Result;
use dashmap::DashMap;
use hidra_protocol::{
    CreateIn, CreateOut, DestroyIn, DeviceKind, IOCTL_HIDRA_CREATE, IOCTL_HIDRA_DESTROY,
    IOCTL_HIDRA_UPDATE, PadState, UpdateIn,
};
use std::os::windows::io::{AsRawHandle, FromRawHandle, RawHandle};
use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
use std::{
    os::windows::io::OwnedHandle,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tracing::debug;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_OVERLAPPED, FILE_GENERIC_READ,
    FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::IO::DeviceIoControl;
use windows::core::PCWSTR;

// TODO: move to config or registry once driver publishes it
const DEVICE_SYMLINK: &str = r"\\.\HIDraBus0";

pub struct Driver {
    next: AtomicU64,
    live: DashMap<u64, DeviceKind>,
    hdev: OwnedHandle,
}

impl Driver {
    pub fn new() -> Arc<Self> {
        let h = open_device(DEVICE_SYMLINK).expect("open hidra device");
        Arc::new(Self { next: AtomicU64::new(1), live: DashMap::new(), hdev: owned_from_handle(h) })
    }
}

#[async_trait::async_trait]
impl Backend for Driver {
    async fn create(&self, kind: DeviceKind, features: u32) -> Result<u64> {
        let h = self.next.fetch_add(1, Ordering::SeqCst);
        let cin = CreateIn { kind, features };
        let mut cout = CreateOut { handle: 0 };
        ioctl(as_handle(&self.hdev), IOCTL_HIDRA_CREATE, Some(&cin), Some(&mut cout))?;
        self.live.insert(h, kind);
        debug!(host_handle = h, drv_handle = cout.handle, ?kind, features, "created");
        // You can choose to return cout.handle instead of h if you want the driver to own ids.
        Ok(h)
    }

    async fn destroy(&self, handle: u64) -> Result<()> {
        let din = DestroyIn { handle };
        ioctl(as_handle(&self.hdev), IOCTL_HIDRA_DESTROY, Some(&din), Option::<&mut ()>::None)?;
        self.live.remove(&handle);
        Ok(())
    }

    async fn update(&self, h: u64, s: PadState) -> Result<()> {
        // Optionally pack per-kind here (X360/DS4/DS5) into UpdateIn payload
        let uin = UpdateIn { handle: h, state: s };
        ioctl(as_handle(&self.hdev), IOCTL_HIDRA_UPDATE, Some(&uin), Option::<&mut ()>::None)?;
        Ok(())
    }
}

fn open_device(sym_link: &str) -> Result<HANDLE> {
    let wide: Vec<u16> = OsStr::new(sym_link).encode_wide().chain(Some(0)).collect();
    let h = unsafe {
        CreateFileW(
            PCWSTR::from_raw(wide.as_ptr()),
            FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
            None,
        )
    }?;
    Ok(h)
}

fn ioctl<TIn: Sized, TOut: Sized>(
    h: HANDLE,
    code: u32,
    inp: Option<&TIn>,
    out: Option<&mut TOut>,
) -> Result<()> {
    let mut bytes: u32 = 0;
    unsafe {
        DeviceIoControl(
            h,
            code,
            inp.map(|p| (p as *const TIn).cast()),
            std::mem::size_of::<TIn>() as u32,
            out.map(|p| (p as *mut TOut).cast()),
            std::mem::size_of::<TOut>() as u32,
            Some(&mut bytes),
            None,
        )
    }?;
    Ok(())
}

fn owned_from_handle(h: HANDLE) -> OwnedHandle {
    unsafe { OwnedHandle::from_raw_handle(h.0 as RawHandle) }
}

fn as_handle(h: &OwnedHandle) -> HANDLE {
    HANDLE(h.as_raw_handle())
}
