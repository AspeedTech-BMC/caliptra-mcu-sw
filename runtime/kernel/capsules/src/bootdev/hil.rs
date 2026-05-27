// Licensed under the Apache-2.0 license

use kernel::ErrorCode;

/// Implemented by any storage backend that can load firmware images.
/// Backends register with `BootDevCapsule::register_backend`.
pub trait ImageFlash {
    /// Synchronously load `image_id` into memory at `dst_addr`.
    /// Returns the image size in bytes on success.
    fn load_image(&self, image_id: u32, dst_addr: u32) -> Result<u32, ErrorCode>;
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
