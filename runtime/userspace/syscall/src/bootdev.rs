// Licensed under the Apache-2.0 license

// Boot device userspace syscall library.
//
// Provides an interface to query the active boot device and to load firmware
// images. The kernel policy picks the boot transport; the app asks with
// bootdev_type() and follows the matching flow:
//
//   IPC (kernel-side load by image id):
//     image_loaded(image_id, load_addr) → wait_init_done(...)
//
//   Flash (raw reads, image format parsed in userspace):
//     read(...) for FLSH header/TOC + payload chunks (+ DMA syscall to the
//     load address) → wait_init_done(...)
//
//   Kernel-initiated requests (hardware PHY images) arrive via
//     app_ready() → wait_image_needed() before either flow.

use crate::DefaultSyscalls;
use caliptra_mcu_libtock_platform::{share, DefaultConfig, ErrorCode, Syscalls};
use caliptra_mcu_libtockasync::TockSubscribe;
use core::marker::PhantomData;

pub const DRIVER_NUM: u32 = 0x9000_1000;

/// Boot-device types returned by `bootdev_type()` (mirrors the kernel's
/// `BootDevType` enum).
pub mod boot_dev_type {
    /// Kernel loads by image_id over IPC (image_loaded flow).
    pub const IPC: u32 = 1;
    /// Raw flash reads; image format parsed in userspace (read flow).
    pub const FLASH: u32 = 2;
}

// -----------------------------------------------------------------------------
// BootDev syscall struct
// -----------------------------------------------------------------------------

pub struct BootDev<S: Syscalls = DefaultSyscalls> {
    _syscall: PhantomData<S>,
    driver_num: u32,
}

impl<S: Syscalls> Default for BootDev<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Syscalls> BootDev<S> {
    pub fn new() -> Self {
        Self {
            _syscall: PhantomData,
            driver_num: DRIVER_NUM,
        }
    }

    /// Returns `Ok(())` if the kernel boot-device capsule is present.
    pub fn exists(&self) -> Result<(), ErrorCode> {
        S::command(self.driver_num, cmd::EXISTS, 0, 0).to_result()
    }

    /// Returns `Ok(())` if the boot device is ready, `Err(ErrorCode)` otherwise.
    pub fn bootdev_ready(&self) -> Result<(), ErrorCode> {
        S::command(self.driver_num, cmd::BOOTDEV_READY, 0, 0).to_result()
    }

    /// Register this process as the boot agent.
    ///
    /// The capsule will subsequently deliver `IMAGE_NEEDED` upcalls for each
    /// kernel driver that requires a PHY image (DRAM, Display, BIOS, ...).
    /// Must be called before `wait_image_needed`.
    pub fn app_ready(&self) -> Result<(), ErrorCode> {
        S::command(self.driver_num, cmd::APP_READY, 0, 0).to_result()
    }

    /// Block until the kernel requests a hardware PHY image.
    ///
    /// Returns `(image_id, 0)` — the second value is reserved and always 0
    /// (the kernel no longer forwards a requested size).
    ///
    /// The subscribe syscall and `app_ready` command both execute synchronously
    /// before the first `.await`, guaranteeing the handler is registered before
    /// the kernel calls `schedule_upcall` inside the APP_READY handler.
    pub async fn wait_image_needed(&self) -> Result<(u32, u32), ErrorCode> {
        // 1. Register the handler with the kernel before signalling readiness.
        let sub = TockSubscribe::subscribe::<S>(self.driver_num, subscribe::IMAGE_NEEDED);

        // 2. Signal readiness; BUSY on 2nd+ calls is normal (app already registered).
        let _ = S::command(self.driver_num, cmd::IMAGE_WAITTING, 0, 0).to_result::<(), ErrorCode>();

        // 3. First yield point — upcall is delivered here.
        let result = TockSubscribe::subscribe_finish(sub).await;
        result.map(|(image_id, img_size, _)| (image_id as u32, img_size as u32))
    }

    /// Ask the kernel to load `image_id` to `addr` through the boot-device
    /// backend (IPC flow); returns `(image_id, img_size)` when READ_DONE
    /// fires.
    pub async fn image_loaded(
        &self,
        image_id: u32,
        addr: u32,
        _size: usize,
    ) -> Result<(u32, u32), ErrorCode> {
        let result = share::scope::<(), _, _>(|_handle| {
            let mut sub = TockSubscribe::subscribe::<S>(self.driver_num, subscribe::READ_DONE);

            if let Err(e) = S::command(self.driver_num, cmd::IMAGE_LOADED, image_id, addr as u32)
                .to_result::<(), ErrorCode>()
            {
                sub.cancel();
                Err(e)?;
            }

            Ok(TockSubscribe::subscribe_finish(sub))
        })?
        .await;

        // READ_DONE args: (image_id, img_size, 0) on success,
        // (image_id, 0, error) on failure.
        let (image_id, img_size, err) = result?;
        if err != 0 {
            return Err(ErrorCode::Fail);
        }
        Ok((image_id as u32, img_size as u32))
    }

    /// Ask the kernel which kind of boot device its policy selected.
    /// Returns the `BootDevType` discriminant (1 = IPC, 2 = Flash); the
    /// boot agent picks the matching flow (IPC: image_loaded / Flash: raw
    /// read + userspace image-format parsing).
    pub fn bootdev_type(&self) -> Result<u32, ErrorCode> {
        S::command(self.driver_num, cmd::BOOTDEV_TYPE, 0, 0).to_result::<u32, ErrorCode>()
    }

    /// Gets the chunk size for read operations.
    pub fn get_chunk_size(&self) -> Result<usize, ErrorCode> {
        S::command(self.driver_num, cmd::GET_CHUNK_SIZE, 0, 0)
            .to_result()
            .map(|x: u32| x as usize)
    }

    /// Internal function to read a chunk of data from the boot device.
    /// Don't use this function directly, use `read` instead.
    ///
    /// `chunk_size` is passed in (rather than queried here) because the
    /// caller already looked it up once for the whole `read()` call --
    /// re-querying it on every chunk would cost one extra command syscall
    /// per chunk for no benefit, since `read()` never asks for more than
    /// `chunk_size` bytes.
    async fn read_chunk(
        &self,
        address: usize,
        len: usize,
        buf: &mut [u8],
        chunk_size: usize,
    ) -> Result<(), ErrorCode> {
        // Check if the buffer is large enough and the length is within the chunk size
        if buf.len() < len || len > chunk_size {
            return Err(ErrorCode::NoMem);
        }

        let result = share::scope::<(), _, _>(|_handle| {
            let mut sub = TockSubscribe::subscribe_allow_rw::<S, DefaultConfig>(
                self.driver_num,
                subscribe::READ_DONE,
                rw_allow::READ,
                buf,
            );

            if let Err(e) = S::command(self.driver_num, cmd::READ, address as u32, len as u32)
                .to_result::<(), ErrorCode>()
            {
                S::unallow_rw(self.driver_num, rw_allow::READ);
                // Cancel the future if the command fails
                sub.cancel();
                Err(e)?;
            }

            Ok(TockSubscribe::subscribe_finish(sub))
        })?
        .await;

        S::unallow_rw(self.driver_num, rw_allow::READ);
        result.map(|_| ())
    }

    /// Boot-device read with the same shape as `SpiFlash::read`: copy `len`
    /// bytes starting at flash byte `address` into `buf`.
    ///
    /// The caller owns the image format; the kernel backend only executes
    /// the read.
    pub async fn read(&self, address: usize, len: usize, buf: &mut [u8]) -> Result<(), ErrorCode> {
        if buf.len() < len {
            return Err(ErrorCode::NoMem);
        }

        // Split into chunk reads
        let chunk_size = self.get_chunk_size()?;
        let mut remaining = len;
        let mut current_address = address;
        let mut offset = 0;
        while remaining > 0 {
            let n = remaining.min(chunk_size);
            self.read_chunk(current_address, n, &mut buf[offset..offset + n], chunk_size)
                .await?;
            current_address += n;
            offset += n;
            remaining -= n;
        }
        Ok(())
    }

    /// Block until the kernel driver signals its hardware init is complete.
    ///
    /// `load_addr`/`size` describe where the image was placed; they travel
    /// alongside IMAGE_INITIAL in the IMAGE_INFO allow buffer (a command
    /// only carries two arguments, so the triple goes through an allowed
    /// buffer), letting the kernel hand real values to the drivers when
    /// the transfer ran in userspace (flash boot).
    ///
    /// Returns `Ok(())` when the driver reports a successful init;
    /// `Err(Fail)` when it reports failure (INIT_DONE result != 0).
    pub async fn wait_init_done(
        &self,
        image_id: u32,
        load_addr: u64,
        size: u32,
    ) -> Result<(), ErrorCode> {
        // Little-endian { load_addr: u64, image_id: u32, img_size: u32 };
        // must stay in sync with the kernel capsule's IMAGE_INFO layout.
        let mut info = [0u8; 16];
        info[0..8].copy_from_slice(&(load_addr).to_le_bytes());
        info[8..12].copy_from_slice(&image_id.to_le_bytes());
        info[12..16].copy_from_slice(&size.to_le_bytes());

        let result = share::scope::<(), _, _>(|_handle| {
            let mut sub = TockSubscribe::subscribe_allow_ro::<S, DefaultConfig>(
                self.driver_num,
                subscribe::INIT_DONE,
                ro_allow::IMAGE_INFO,
                &info,
            );

            if let Err(e) =
                S::command(self.driver_num, cmd::IMAGE_INITIAL, 0, 0).to_result::<(), ErrorCode>()
            {
                S::unallow_ro(self.driver_num, ro_allow::IMAGE_INFO);
                sub.cancel();
                Err(e)?;
            }
            Ok(TockSubscribe::subscribe_finish(sub))
        })?
        .await;

        S::unallow_ro(self.driver_num, ro_allow::IMAGE_INFO);

        // INIT_DONE args: (image_id, result, 0) — result 0 = success.
        let (_, status, _) = result?;
        if status != 0 {
            return Err(ErrorCode::Fail);
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Command IDs
// -----------------------------------------------------------------------------

mod cmd {
    /// Returns success if the capsule is present.
    pub const EXISTS: u32 = 0;
    /// Register process as boot agent; triggers deferred IMAGE_NEEDED upcalls.
    pub const APP_READY: u32 = 1;
    /// Load an image via the boot-device backend: r2 = image_id,
    /// r3 = load addr (low 32 bits). Fires READ_DONE when done.
    pub const IMAGE_LOADED: u32 = 2;
    /// Request the next kernel-initiated image: fires IMAGE_NEEDED with the
    /// next pending request, or the image_id == 0 sentinel if none.
    pub const IMAGE_WAITTING: u32 = 3;
    /// Trigger image initialization. The command registers are unused; the
    /// image is identified by the IMAGE_INFO allow buffer shared alongside.
    pub const IMAGE_INITIAL: u32 = 4;
    /// Returns success if the boot device is ready.
    pub const BOOTDEV_READY: u32 = 5;
    /// Read, same shape as the flash-partition READ: r2 = flash byte
    /// address, r3 = length; data lands in the allowed read buffer and
    /// READ_DONE fires when done.
    pub const READ: u32 = 6;
    /// Returns the read chunk size in bytes.
    pub const GET_CHUNK_SIZE: u32 = 7;
    /// Returns the BootDevType of the backend servicing requests.
    pub const BOOTDEV_TYPE: u32 = 8;
}

// -----------------------------------------------------------------------------
// Subscribe and allow-buffer IDs
// -----------------------------------------------------------------------------
mod subscribe {
    /// Kernel → app: hardware driver needs image. args: (image_id, 0, 0);
    /// image_id == 0 is the "no more pending requests" sentinel.
    pub const IMAGE_NEEDED: u32 = 0;
    /// Kernel → app: hardware init complete. args: (image_id, result, 0);
    /// result 0 = success.
    pub const INIT_DONE: u32 = 1;
    /// Kernel → app: transfer complete. Args depend on the flow:
    /// IMAGE_LOADED → (image_id, img_size, 0) on success,
    ///                (image_id, 0, error) on failure;
    /// READ         → (length, 0, 0).
    pub const READ_DONE: u32 = 2;
}

/// Ids for read-only allow buffers
mod ro_allow {
    /// Image info shared alongside IMAGE_INITIAL, little-endian packed:
    /// { load_addr: u64, image_id: u32, img_size: u32 }.
    pub const IMAGE_INFO: u32 = 0;
}

/// Ids for read-write allow buffers
mod rw_allow {
    /// Buffer for READ data.
    pub const READ: u32 = 0;
}
