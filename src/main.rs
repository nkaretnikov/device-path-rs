#![no_std]
#![no_main]
#![feature(asm)]
#![feature(const_slice_len)]
#![feature(slice_patterns)]
#![feature(optin_builtin_traits)]
#![feature(core_intrinsics)]

extern crate alloc;
#[macro_use]
extern crate log;
extern crate uefi;
extern crate uefi_exts;
extern crate uefi_macros;

use alloc::fmt::format;
use core::fmt;
use core::convert::TryFrom;
use core::intrinsics::type_name;
use crate::alloc::vec::Vec;
use uefi_exts::BootServicesExt;
use uefi::prelude::*;
use uefi_services::init;

// XXX: Can't figure out how to import `Protocol` otherwise.
use uefi::proto::*;
use uefi::*;

// XXX: Requires removing `pub(crate)` in uefi-rs.
use uefi::data_types::{Guid, unsafe_guid, Identify};

// XXX: Work around `_fltused` being undefined:
// https://github.com/rust-lang/rust/issues/62785
#[used]
#[no_mangle]
pub static _fltused: i32 = 0;

pub const BLOCK_IO_PROTOCOL_REVISION2: u64 = 0x20001;
pub const BLOCK_IO_PROTOCOL_REVISION3: u64 = (2 << 16) | 31;

// XXX: Why doesn't uefi-rs use `repr` for `Protocol`s?

#[repr(C)]
#[unsafe_guid("964e5b21-6459-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct BlockIO<'boot> {
    pub revision: u64,
    pub media: &'boot BlockIOMedia,

    pub reset: extern "win64" fn(
        /* in  */ this: &mut BlockIO,
        /* in  */ extended_verification: bool,
    ) -> Status,

    pub read_blocks: extern "win64" fn(
        /* in  */ this: &mut BlockIO,
        /* in  */ media_id: u32,
        /* in  */ lba: LBA,
        /* in  */ buffer_size: usize,  // bytes
        /* out */ buffer: *mut u8,
    ) -> Status,

    pub write_blocks: extern "win64" fn(
        /* in  */ this: &mut BlockIO,
        /* in  */ media_id: u32,
        /* in  */ lba: LBA,
        /* in  */ buffer_size: usize,  // bytes
        /* in  */ buffer: *mut u8,
    ) -> Status,

    pub flush_blocks: extern "win64" fn(
        /* in  */ this: &mut BlockIO,
    ) -> Status,
}

#[repr(C)]
pub struct BlockIOMedia {
    pub media_id: u32,
    pub removable_media: bool,
    pub media_present: bool,
    pub logical_partition: bool,
    pub read_only: bool,
    pub write_caching: bool,
    pub block_size: u32,
    pub io_align: u32,
    pub last_block: LBA,
    pub lowest_aligned_lba: LBA,  // added in Revision 2
    pub logical_blocks_per_physical_block: u32,  // added in Revision 2
    pub optimal_transfer_length_granularity: u32, // added in Revision 3
}

type LBA = u64;

// Data is stored after this structure, so this needs to be `packed`.
// XXX: `packed` can cause undefined behavior:
// https://github.com/rust-lang/rust/issues/27060
#[repr(C)]
#[repr(packed)]
#[unsafe_guid("09576e91-6d3f-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct DevicePath {
    pub r#type: u8,
    pub sub_type: u8,
    pub length: [u8; 2],
}

impl DevicePath {
    pub extern "C" fn len(&self) -> u16 {
        (self.length[0] as u16) | ((self.length[1] as u16) << 8)
    }
}

#[repr(C)]
#[unsafe_guid("8b843e20-8132-4852-90cc-551a4e4a7f1c")]
#[derive(Protocol)]
pub struct DevicePathToText {
    pub convert_device_node_to_text: extern "win64" fn(
        /* in */ device_node: *const DevicePath,
        /* in */ display_only: bool,
        /* in */ allow_shortcuts: bool,
    ) -> *mut Char16,

    pub convert_device_path_to_text: extern "win64" fn(
        /* in */ device_path: *const DevicePath,
        /* in */ display_only: bool,
        /* in */ allow_shortcuts: bool,
    ) -> *mut Char16,
}

// XXX: `Display` is not defined for `CStr16`.
struct DCStr16(*mut Char16);

impl fmt::Display for DCStr16 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = unsafe { CStr16::from_ptr(self.0) };
        for x in s.to_u16_slice() {
            if let Ok(c) = Char16::try_from(*x) {
                write!(fmt, "{}", c)?;
            }
        }

        Ok(())
    }
}

pub extern "C" fn find_handles<Protocol>(
    boot_services: &BootServices,
) -> Vec<Handle>
    where Protocol: uefi::proto::Protocol
{
    let type_name = type_name::<Protocol>();
    let err = format(format_args!(
            "Failed to retrieve list of {} handles", type_name));

    boot_services
        .find_handles::<Protocol>()
        .expect_success(err.as_str())
}

pub extern "C" fn handle_protocol<'a, Protocol>(
    boot_services: &BootServices,
    handle: &'a Handle,
) -> &'a Protocol
    where Protocol: uefi::proto::Protocol
{
    let type_name = type_name::<Protocol>();
    let err = format(format_args!("Failed to handle {} protocol", type_name));

    let protocol = boot_services
        .handle_protocol::<Protocol>(*handle)
        .expect_success(err.as_str());
    unsafe { &mut *protocol.get() }
}

#[no_mangle]
pub extern "C" fn efi_main(
    _image: uefi::Handle,
    system_table: SystemTable<Boot>,
) -> Status {
    // Initialize utilities (such as logging and memory allocation).
    init(&system_table).expect_success("Failed to initialize utilities");

    let boot_services = system_table.boot_services();

    // DevicePathToText.
    let dptt_handles: Vec<Handle> =
        find_handles::<DevicePathToText>(boot_services);
    assert_eq!(dptt_handles.len(), 1);
    let dptt_handle = dptt_handles[0];
    let device_path_to_text =
        handle_protocol::<DevicePathToText>(boot_services, &dptt_handle);

    // BlockIO.
    let io_handles: Vec<Handle> = find_handles::<BlockIO>(boot_services);
    for io_handle in &io_handles {
        let block_io = handle_protocol::<BlockIO>(boot_services, io_handle);
        let device_path = handle_protocol::<DevicePath>(boot_services, io_handle);
        let text =
            (device_path_to_text.convert_device_path_to_text)(
                device_path, true, true);

        if block_io.media.removable_media {
            info!("removable: {}", DCStr16(text));
        } else {
            info!("non-removable: {}", DCStr16(text));
        }
    }

    Status::SUCCESS
}
