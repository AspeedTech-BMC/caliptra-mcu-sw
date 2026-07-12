// Licensed under the Apache-2.0 license

//! Flash boot-device backend.
//!
//! Thin adapter that registers a flash controller with `BootDevCapsule`
//! through Tock's virtualized flash stack. Reads are delegated to a
//! `flash_driver::flash_storage_to_pages::FlashStorageToPages` sitting on a
//! `virtual_flash::FlashUser`, so every access is arbitrated by the shared
//! `MuxFlash` together with the flash-partition users:
//!
//! ```text
//! BootDevCapsule ── ImageFlash::read(buf, address, len)
//!   └─ BootDevFlash (this) ── FlashStorage::read
//!        └─ FlashStorageToPages ── FlashUser ── MuxFlash ── flash_ctrl
//! ```
//!
//! The bootdev capsule only abstracts *which* boot device serves the boot
//! flow; it reads into buffers exactly like the flash-partition syscall.
//! Moving data to an image load address is not its job — the boot agent
//! composes `read` with the DMA syscall in userspace (see
//! `FlashImageLoader`). `load_image` is the IPC backend's job and reports
//! NOSUPPORT here.

use super::hil::{BootDevType, ImageFlash, ImageFlashClient, LoadStatus};
use caliptra_mcu_flash_driver::hil::{FlashStorage, FlashStorageClient};
use core::cell::Cell;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

pub struct BootDevFlash<'a> {
    storage: &'a dyn FlashStorage<'a>,
    client: OptionalCell<&'static dyn ImageFlashClient>,
    busy: Cell<bool>,
    // Partition window bound at init: physical start offset and length.
    // Userspace read addresses are partition-relative (0 = package start).
    base: usize,
    size: usize,
}

impl<'a> BootDevFlash<'a> {
    pub fn new(storage: &'a dyn FlashStorage<'a>, base: usize, size: usize) -> Self {
        Self {
            storage,
            client: OptionalCell::empty(),
            busy: Cell::new(false),
            base,
            size,
        }
    }
}

impl ImageFlash for BootDevFlash<'_> {
    fn ready(&self) -> bool {
        !self.busy.get()
    }

    fn dev_type(&self) -> BootDevType {
        BootDevType::Flash
    }

    fn set_client(&self, client: &'static dyn ImageFlashClient) {
        self.client.set(client);
    }

    fn load_image(&self, _image_id: u32, _dst_addr: u32) -> Result<LoadStatus, ErrorCode> {
        // The capsule falls through to the IPC backend for image loads.
        Err(ErrorCode::NOSUPPORT)
    }

    fn read(
        &self,
        buffer: &'static mut [u8],
        address: usize,
        length: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if address
            .checked_add(length)
            .filter(|&end| end <= self.size)
            .is_none()
        {
            return Err((ErrorCode::INVAL, buffer));
        }

        if self.busy.get() {
            return Err((ErrorCode::BUSY, buffer));
        }
        self.busy.set(true);
        self.storage
            .read(buffer, self.base + address, length)
            .inspect_err(|_| self.busy.set(false))
    }

    // Reads stay in the MCU's own address space, so addresses are used
    // exactly as provided.
    fn map_to_axi(&self, addr: u64) -> u64 {
        addr
    }

    fn map_from_axi(&self, addr: u64) -> u64 {
        addr
    }
}

impl FlashStorageClient for BootDevFlash<'_> {
    fn read_done(&self, buffer: &'static mut [u8], length: usize) {
        self.busy.set(false);
        self.client.map(move |client| {
            client.read_complete(buffer, length);
        });
    }

    fn write_done(&self, buffer: &'static mut [u8], length: usize) {
        // This backend never writes; unreachable.
        let _ = (buffer, length);
    }

    fn erase_done(&self, _length: usize) {}
}
