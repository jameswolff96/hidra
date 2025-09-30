use super::Backend;
use anyhow::Result;
use dashmap::DashMap;
use hidra_protocol::{
    CreateIn, CreateOut, DestroyIn, DeviceKind, HIDRA_INTERFACE_GUID, IOCTL_HIDRA_CREATE,
    IOCTL_HIDRA_DESTROY, IOCTL_HIDRA_UPDATE, PadState, UpdateIn,
};
use std::os::windows::io::{AsRawHandle, FromRawHandle, RawHandle};
use std::{
    os::windows::io::OwnedHandle,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tracing::debug;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, SP_DEVICE_INTERFACE_DATA,
    SP_DEVICE_INTERFACE_DETAIL_DATA_W, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW,
    SetupDiGetDeviceInterfaceDetailW,
};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::IO::DeviceIoControl;
use windows::core::{GUID, PCWSTR, PWSTR};

pub struct Driver {
    next: AtomicU64,
    live: DashMap<u64, DeviceKind>,
    hdev: OwnedHandle,
}

impl Driver {
    pub fn new() -> Arc<Self> {
        let h = open_by_interface_guid(&HIDRA_INTERFACE_GUID).expect("unable to open handle");
        Arc::new(Self { next: AtomicU64::new(1), live: DashMap::new(), hdev: h })
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

fn open_by_interface_guid(iface: &GUID) -> windows::core::Result<OwnedHandle> {
    unsafe {
        let hdev =
            SetupDiGetClassDevsW(Some(iface), None, None, DIGCF_DEVICEINTERFACE | DIGCF_PRESENT)?;
        let mut idx = 0;
        loop {
            let mut di = SP_DEVICE_INTERFACE_DATA {
                cbSize: std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
                ..Default::default()
            };
            SetupDiEnumDeviceInterfaces(hdev, None, iface, idx, &mut di)?;
            // query required size
            let mut req = 0u32;
            let _ = SetupDiGetDeviceInterfaceDetailW(hdev, &di, None, 0, Some(&mut req), None);
            let mut buf = vec![0u8; req as usize];
            let p = buf.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
            (*p).cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;
            SetupDiGetDeviceInterfaceDetailW(hdev, &di, Some(p), req, None, None)?;
            let path = {
                let pw = PWSTR((*p).DevicePath.as_mut_ptr());
                let len = (0..).take_while(|&i| *pw.0.add(i) != 0).count();
                String::from_utf16_lossy(std::slice::from_raw_parts(pw.0, len))
            };
            // open the device path
            let access = FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0;
            let share = FILE_SHARE_READ | FILE_SHARE_WRITE;
            let attrs = FILE_ATTRIBUTE_NORMAL;
            let h = CreateFileW(
                PCWSTR(
                    path.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>().as_ptr(),
                ),
                access,
                share,
                None,
                OPEN_EXISTING,
                attrs,
                None,
            )?;
            if h != INVALID_HANDLE_VALUE {
                return Ok(owned_from_handle(h));
            }
            idx += 1;
        }
    }
}

fn owned_from_handle(h: HANDLE) -> OwnedHandle {
    unsafe { OwnedHandle::from_raw_handle(h.0 as RawHandle) }
}

fn as_handle(h: &OwnedHandle) -> HANDLE {
    HANDLE(h.as_raw_handle())
}
