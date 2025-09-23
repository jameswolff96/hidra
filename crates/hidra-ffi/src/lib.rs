#![deny(warnings)]
use hidra_protocol::HIDRA_FFI_ABI_VERSION;

#[no_mangle]
pub extern "C" fn hidra_abi_version() -> u32 {
HIDRA_FFI_ABI_VERSION
}
