// Licensed under the Apache-2.0 license

use kernel::ErrorCode;

/// Kind of boot-device backend, reported to userspace via the
/// BOOTDEV_TYPE command so the boot agent can pick its flow:
/// IPC → `load_image` (IMAGE_LOADED), Flash → raw `read` (READ)
/// with the image format parsed in userspace.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BootDevType {
    Ipc = 1,
    Flash = 2,
}

/// Outcome of a successfully started `load_image` request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadStatus {
    /// The load finished within the `load_image` call itself; carries the
    /// image size in bytes.
    Complete(u32),
    /// The load was started; completion is delivered later through
    /// `ImageFlashClient::load_complete`.
    Pending,
}

/// Implemented by `BootDevCapsule` to receive completion of an asynchronous
/// `load_image` or `read` request.
pub trait ImageFlashClient {
    /// The load for `image_id` finished. `Ok(size)` carries the image size
    /// in bytes.
    fn load_complete(&self, image_id: u32, result: Result<u32, ErrorCode>);
    /// A split-phase `read` finished: the buffer is handed back with the
    /// number of bytes that were actually read (mirrors
    /// `FlashStorageClient::read_done`).
    fn read_complete(&self, buffer: &'static mut [u8], length: usize);
}

/// Implemented by any storage backend that can load firmware images.
/// Backends register with `BootDevCapsule::register_backend`.
///
/// Backends may service `load_image` either synchronously (returning
/// `LoadStatus::Complete`, e.g. the IPC backend) or split-phase (returning
/// `LoadStatus::Pending` and signalling the registered `ImageFlashClient`
/// when done, e.g. a backend built on Tock's split-phase flash HIL).
pub trait ImageFlash {
    /// Returns whether this backend is ready to service `load_image` requests.
    fn ready(&self) -> bool;
    /// Which kind of boot device this backend is (for BOOTDEV_TYPE).
    fn dev_type(&self) -> BootDevType;
    /// Register the completion client for asynchronous loads. Backends that
    /// only complete synchronously may keep the default no-op.
    fn set_client(&self, client: &'static dyn ImageFlashClient) {
        let _ = client;
    }
    /// Load `image_id` into memory at `dst_addr`.
    /// Returns `LoadStatus::Complete(size)` when the load finished within
    /// this call, or `LoadStatus::Pending` when it was started and will be
    /// signalled through `ImageFlashClient::load_complete`.
    fn load_image(&self, image_id: u32, dst_addr: u32) -> Result<LoadStatus, ErrorCode>;
    /// Read `length` bytes starting at flash byte `address` into `buffer`
    /// (same shape as `FlashStorage::read`); completion is delivered via
    /// `ImageFlashClient::read_complete`. Only meaningful for flash-type
    /// backends — the capsule dispatches READ to devices whose `dev_type()`
    /// is `BootDevType::Flash`, so other backends keep the default.
    fn read(
        &self,
        buffer: &'static mut [u8],
        address: usize,
        length: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        let _ = (address, length);
        Err((ErrorCode::NOSUPPORT, buffer))
    }
    /// Translate an SS-MCU address to the AXI physical address space.
    fn map_to_axi(&self, addr: u64) -> u64;
    /// Translate an AXI address to the SS-MCU address space.
    fn map_from_axi(&self, addr: u64) -> u64;
}

/// Implemented by BootDevCapsule. Kernel drivers call this to request images;
/// the capsule relays the request to the userspace app.
pub trait KernelImageLoader {
    fn request_image(
        &self,
        image_id: u32,
        img_size: usize,
        client: &'static dyn KernelImageClient,
    ) -> Result<(), ErrorCode>;
    /// Called by the driver after its hardware init completes (e.g. DRAM training).
    fn notify_init_done(&self, image_id: u32, success: bool);
    /// Register a passive watcher: `image_ready` is called when `image_id`
    /// is loaded, without triggering an IMAGE_NEEDED upcall to userspace.
    fn watch_image(
        &self,
        image_id: u32,
        client: &'static dyn KernelImageClient,
    ) -> Result<(), ErrorCode>;
}

/// Implemented by each kernel hardware driver (DramCtrl, DisplayCtrl, …).
pub trait KernelImageClient {
    /// Firmware bytes for the previously requested image are at `addr`, `size` bytes.
    fn image_ready(&self, image_id: u32, addr: u64, size: u32);
    fn image_error(&self, err: ErrorCode);
}
