// Licensed under the Apache-2.0 license

pub mod flash;
pub mod hil;
pub mod ipc;

use self::hil::{
    BootDevType, ImageFlash, ImageFlashClient, KernelImageClient, KernelImageLoader, LoadStatus,
};
use core::cell::Cell;
use core::cmp;
use core::marker::PhantomData;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

pub const DRIVER_NUM: usize = 0x9000_1000;

/// Size of the internal buffer used to shuttle READ data into the app's
/// allowed buffer; also the read chunk size reported to userspace.
pub const BUF_LEN: usize = 512;

mod upcall {
    /// Kernel → app: driver needs image. args: (image_id, img_size, 0).
    /// image_id == 0 is a sentinel meaning no more pending requests.
    pub const IMAGE_NEEDED: usize = 0;
    /// Kernel → app: hardware init complete. args: (image_id, result, all_done).
    pub const INIT_DONE: usize = 1;
    /// Kernel → app: READ transfer done. args: (status, len, 0).
    pub const READ_DONE: usize = 2;
    pub const COUNT: u8 = 3;
}

mod cmd {
    pub const EXISTS: usize = 0;
    pub const APP_READY: usize = 1;
    pub const IMAGE_LOADED: usize = 2;
    pub const IMAGE_WAITTING: usize = 3;
    pub const IMAGE_INITIAL: usize = 4;
    pub const BOOTDEV_READY: usize = 5;
    /// Boot-device read, same shape as the flash-partition READ:
    /// r2 = flash byte address, r3 = length in bytes; data lands in the
    /// allowed read buffer and READ_DONE fires when done. The app owns the
    /// image format; the kernel only executes the read.
    pub const READ: usize = 6;
    /// Returns the read chunk size in bytes (BUF_LEN).
    pub const GET_CHUNK_SIZE: usize = 7;
    /// Returns the BootDevType of the backend that will service requests.
    pub const BOOTDEV_TYPE: usize = 8;
}

/// Ids for read-write allow buffers
mod rw_allow {
    /// Buffer for READ data.
    pub const READ: usize = 0;
    /// The number of allow buffers the kernel stores for this grant
    pub const COUNT: u8 = 1;
}

#[derive(Default)]
pub struct App {}

const MAX_PENDING: usize = 32;
const MAX_WATCHERS: usize = 8;
const MAX_FLASH_DEVS: usize = 8;

#[derive(Copy, Clone)]
struct PendingEntry {
    image_id: u32,
    client: &'static dyn KernelImageClient,
    img_size: u32,
    load_addr: u64,
}

pub struct BootDevCapsule<'a> {
    // Additional devices may be registered later via register_flash_dev().
    flash_devs: Cell<[Option<&'static dyn ImageFlash>; MAX_FLASH_DEVS]>,
    flash_dev_count: Cell<usize>,
    /// None = auto (poll all, first available wins); Some(i) = use flash_devs[i] directly.
    /// Set by board init via set_flash_dev_policy(), not by userspace.
    flash_dev_policy: Cell<Option<usize>>,
    // FIFO queue of pending requests; load_addr filled in by IMAGE_LOADED
    pending_queue: Cell<[Option<PendingEntry>; MAX_PENDING]>,
    pending_head: Cell<usize>,
    pending_count: Cell<usize>,
    // image_id currently waiting for IMAGE_LOADED; None when idle
    active_image_id: Cell<Option<u32>>,
    // Passive watchers: notified via image_ready() when a matching image is loaded,
    // without triggering an IMAGE_NEEDED upcall to userspace.
    watchers: Cell<[Option<PendingEntry>; MAX_WATCHERS]>,
    watcher_count: Cell<usize>,
    // Internal buffer for shuttling READ data into the app's allowed buffer.
    buffer: TakeCell<'static, [u8]>,
    // App with a READ in flight.
    read_app: OptionalCell<ProcessId>,
    // Userspace app
    apps: Grant<
        App,
        UpcallCount<{ upcall::COUNT }>,
        AllowRoCount<0>,
        AllowRwCount<{ rw_allow::COUNT }>,
    >,
    active_app: OptionalCell<ProcessId>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> BootDevCapsule<'a> {
    pub fn new(
        devs: &[&'static dyn ImageFlash],
        grant: Grant<
            App,
            UpcallCount<{ upcall::COUNT }>,
            AllowRoCount<0>,
            AllowRwCount<{ rw_allow::COUNT }>,
        >,
        buffer: &'static mut [u8],
    ) -> Self {
        let mut flash_devs = [None; MAX_FLASH_DEVS];
        let count = devs.len().min(MAX_FLASH_DEVS);
        for i in 0..count {
            flash_devs[i] = Some(devs[i]);
        }

        Self {
            flash_devs: Cell::new(flash_devs),
            flash_dev_count: Cell::new(count),
            flash_dev_policy: Cell::new(None),
            pending_queue: Cell::new([None; MAX_PENDING]),
            pending_head: Cell::new(0),
            pending_count: Cell::new(0),
            active_image_id: Cell::new(None),
            watchers: Cell::new([None; MAX_WATCHERS]),
            watcher_count: Cell::new(0),
            buffer: TakeCell::new(buffer),
            read_app: OptionalCell::empty(),
            apps: grant,
            active_app: OptionalCell::empty(),
            _phantom: PhantomData,
        }
    }

    fn fire_next_image_needed(&self) {
        let count = self.pending_count.get();
        if count == 0 {
            return;
        }
        let head = self.pending_head.get();
        let queue = self.pending_queue.get();
        let entry = match queue[head % MAX_PENDING] {
            Some(e) => e,
            None => return,
        };
        self.active_image_id.set(Some(entry.image_id));
        if let Some(pid) = self.active_app.get() {
            let _ = self.apps.enter(pid, |_, kd| {
                kd.schedule_upcall(
                    upcall::IMAGE_NEEDED,
                    (entry.image_id as usize, entry.img_size as usize, 0),
                )
                .ok();
            });
        }
    }

    fn fire_image_needed_sentinel(&self, pid: ProcessId) {
        let _ = self.apps.enter(pid, |_, kd| {
            kd.schedule_upcall(upcall::IMAGE_NEEDED, (0, 0, 0)).ok();
        });
    }

    /// Selects which backend services requests: `None` = auto (first
    /// backend that accepts wins), `Some(i)` = only `flash_devs[i]`.
    /// Called by board init, not by userspace.
    pub fn set_flash_dev_policy(&self, policy: Option<usize>) {
        self.flash_dev_policy.set(policy);
    }

    /// Returns the effective flash device list based on the current policy.
    /// - `None` policy: all registered devices (full list, up to `flash_dev_count`)
    /// - `Some(idx)` policy: only the device at that index, placed at position 0
    fn get_flash_devs(&self) -> [Option<&'static dyn ImageFlash>; MAX_FLASH_DEVS] {
        let all = self.flash_devs.get();
        let count = self.flash_dev_count.get();
        match self.flash_dev_policy.get() {
            None => all,
            Some(idx) if idx < count => {
                let mut result = [None; MAX_FLASH_DEVS];
                result[0] = all[idx];
                result
            }
            _ => [None; MAX_FLASH_DEVS],
        }
    }

    /// Persist the confirmed image size into the active pending-queue entry
    /// and unblock the userspace `image_loaded()` call with a READ_DONE
    /// upcall. Shared by the synchronous (`LoadStatus::Complete`) and
    /// asynchronous (`ImageFlashClient::load_complete`) completion paths.
    fn finish_load(&self, image_id: u32, img_size: u32) {
        let mut queue = self.pending_queue.get();
        let head = self.pending_head.get();
        if let Some(ref mut entry) = queue[head % MAX_PENDING] {
            entry.img_size = img_size;
        }
        self.pending_queue.set(queue);

        if let Some(pid) = self.active_app.get() {
            let _ = self.apps.enter(pid, |_, kd| {
                kd.schedule_upcall(upcall::READ_DONE, (image_id as usize, img_size as usize, 0))
                    .ok();
            });
        }
    }
}

impl ImageFlashClient for BootDevCapsule<'_> {
    fn load_complete(&self, image_id: u32, result: Result<u32, ErrorCode>) {
        match result {
            Ok(img_size) => self.finish_load(image_id, img_size),
            Err(err) => {
                // Signal failure to the waiting app: READ_DONE with size 0
                // and the error code in the third argument.
                if let Some(pid) = self.active_app.get() {
                    let _ = self.apps.enter(pid, |_, kd| {
                        kd.schedule_upcall(upcall::READ_DONE, (image_id as usize, 0, err as usize))
                            .ok();
                    });
                }
            }
        }
    }

    // A backend READ finished: copy the data into the app's allowed buffer,
    // bank the internal buffer, and signal the app (mirrors
    // FlashPartition::read_done).
    fn read_complete(&self, buffer: &'static mut [u8], length: usize) {
        if let Some(pid) = self.read_app.take() {
            let _ = self.apps.enter(pid, move |_, kernel_data| {
                let _ = kernel_data
                    .get_readwrite_processbuffer(rw_allow::READ)
                    .and_then(|read| {
                        read.mut_enter(|app_buffer| {
                            let read_len = cmp::min(app_buffer.len(), length);

                            let d = &app_buffer[0..read_len];
                            d.copy_from_slice(&buffer[0..read_len]);
                        })
                    });

                self.buffer.replace(buffer);

                kernel_data
                    .schedule_upcall(upcall::READ_DONE, (length, 0, 0))
                    .ok();
            });
        } else {
            self.buffer.replace(buffer);
        }
    }
}

impl KernelImageLoader for BootDevCapsule<'_> {
    fn request_image(
        &self,
        image_id: u32,
        img_size: usize,
        client: &'static dyn KernelImageClient,
    ) -> Result<(), ErrorCode> {
        let count = self.pending_count.get();
        if count >= MAX_PENDING {
            return Err(ErrorCode::BUSY);
        }
        let mut queue = self.pending_queue.get();
        let tail = (self.pending_head.get() + count) % MAX_PENDING;

        queue[tail] = Some(PendingEntry {
            image_id,
            client,
            img_size: img_size as u32,
            load_addr: 0,
        });
        self.pending_queue.set(queue);
        self.pending_count.set(count + 1);

        // If app is registered and no active request, start immediately
        if self.active_app.is_some() && self.active_image_id.get().is_none() {
            self.fire_next_image_needed();
        }

        Ok(())
    }

    fn watch_image(
        &self,
        image_id: u32,
        client: &'static dyn KernelImageClient,
    ) -> Result<(), ErrorCode> {
        let count = self.watcher_count.get();
        if count >= MAX_WATCHERS {
            return Err(ErrorCode::NOMEM);
        }
        let mut watchers = self.watchers.get();
        watchers[count] = Some(PendingEntry {
            image_id,
            client,
            img_size: 0,
            load_addr: 0,
        });
        self.watchers.set(watchers);
        self.watcher_count.set(count + 1);
        Ok(())
    }

    fn notify_init_done(&self, image_id: u32, success: bool) {
        // Consume from the head of the queue
        let head = self.pending_head.get();
        self.pending_head.set((head + 1) % MAX_PENDING);
        let count = self.pending_count.get();
        self.pending_count.set(count - 1);
        self.active_image_id.set(None);

        let result: usize = if success { 0 } else { 1 };

        if let Some(pid) = self.active_app.get() {
            let _ = self.apps.enter(pid, |_, kd| {
                kd.schedule_upcall(upcall::INIT_DONE, (image_id as usize, result, 0))
                    .ok();
            });
        }

        // Do not auto-fire the next IMAGE_NEEDED here.  The app calls IMAGE_WAITTING
        // after re-subscribing, ensuring the handler is registered before schedule_upcall.
    }
}

impl SyscallDriver for BootDevCapsule<'_> {
    fn command(&self, cmd_num: usize, r2: usize, r3: usize, pid: ProcessId) -> CommandReturn {
        match cmd_num {
            cmd::EXISTS => CommandReturn::success(),

            cmd::BOOTDEV_READY => {
                let devs = self.get_flash_devs();
                let ready = devs.iter().flatten().any(|dev| dev.ready());
                if ready {
                    CommandReturn::success()
                } else {
                    CommandReturn::failure(ErrorCode::FAIL)
                }
            }

            cmd::APP_READY => {
                if self.active_app.is_some() {
                    return CommandReturn::failure(ErrorCode::BUSY);
                }
                self.active_app.set(pid);

                // Boot from spi flash by default
                self.set_flash_dev_policy(Some(1));

                CommandReturn::success()
            }

            // App subscribes to IMAGE_NEEDED then calls this to request the next image.
            // Fires the next pending request, or a sentinel (image_id == 0) if none.
            cmd::IMAGE_WAITTING => {
                if self.active_image_id.get().is_none() {
                    if self.pending_count.get() > 0 {
                        self.fire_next_image_needed();
                    } else {
                        self.fire_image_needed_sentinel(pid);
                    }
                }
                CommandReturn::success()
            }

            cmd::IMAGE_LOADED => {
                let image_id = r2 as u32;
                let addr = r3;

                // Persist the load address into the active pending-queue entry
                // up front so it is available whether the load completes
                // synchronously or through load_complete().
                let mut queue = self.pending_queue.get();
                let head = self.pending_head.get();
                if let Some(ref mut entry) = queue[head % MAX_PENDING] {
                    entry.load_addr = addr as u64;
                }
                self.pending_queue.set(queue);

                let devs = self.get_flash_devs();
                for dev in devs.iter().flatten() {
                    let map_addr = dev.map_to_axi(addr as u64);
                    match dev.load_image(image_id, map_addr as u32) {
                        // Load finished within the call: unblock the
                        // userspace image_loaded() future right away.
                        Ok(LoadStatus::Complete(img_size)) => {
                            self.finish_load(image_id, img_size);
                            return CommandReturn::success();
                        }
                        // Split-phase load started: READ_DONE fires from
                        // load_complete() when the backend is done.
                        Ok(LoadStatus::Pending) => return CommandReturn::success(),
                        Err(_) => continue,
                    }
                }

                CommandReturn::failure(ErrorCode::FAIL)
            }

            // Read r3 bytes from flash byte address r2 into the app's
            // allowed buffer (mirrors the flash-partition READ). The app
            // parses the image format and decides offsets; the backend
            // only executes the read.
            cmd::READ => {
                let address = r2;
                let length = r3;
                if length == 0 {
                    return CommandReturn::failure(ErrorCode::SIZE);
                }
                if self.read_app.is_some() {
                    return CommandReturn::failure(ErrorCode::BUSY);
                }

                // READ is served by a flash-type backend; other backends
                // (IPC) load by image_id instead.
                let devs = self.get_flash_devs();
                let dev = match devs
                    .iter()
                    .flatten()
                    .find(|dev| dev.dev_type() == BootDevType::Flash)
                {
                    Some(dev) => *dev,
                    None => return CommandReturn::failure(ErrorCode::NOSUPPORT),
                };

                let res = self
                    .apps
                    .enter(pid, |_, kernel_data| {
                        let allow_len = kernel_data
                            .get_readwrite_processbuffer(rw_allow::READ)
                            .map_or(0, |read| read.len());
                        if allow_len == 0 {
                            return Err(ErrorCode::RESERVE);
                        }

                        let buffer = self.buffer.take().ok_or(ErrorCode::RESERVE)?;
                        let active_len = cmp::min(cmp::min(length, allow_len), buffer.len());

                        dev.read(buffer, address, active_len)
                            .map_err(|(e, buffer)| {
                                self.buffer.replace(buffer);
                                e
                            })?;
                        self.read_app.set(pid);
                        Ok(())
                    })
                    .unwrap_or_else(|err| Err(err.into()));

                match res {
                    Ok(()) => CommandReturn::success(),
                    Err(e) => CommandReturn::failure(e),
                }
            }

            cmd::GET_CHUNK_SIZE => CommandReturn::success_u32(BUF_LEN as u32),

            // Tells the boot agent which kind of backend the kernel policy
            // selected, so it can pick the matching flow (IPC: load_image /
            // Flash: raw read + userspace parsing).
            cmd::BOOTDEV_TYPE => {
                let devs = self.get_flash_devs();
                match devs.iter().flatten().find(|dev| dev.ready()) {
                    Some(dev) => CommandReturn::success_u32(dev.dev_type() as u32),
                    None => CommandReturn::failure(ErrorCode::OFF),
                }
            }

            cmd::IMAGE_INITIAL => {
                // r2 = image_id,
                // Kicks off async boot-device read → fires READ_DONE upcall when done
                // TODO: integrate with actual boot device driver
                let image_id = r2 as u32;

                let queue = self.pending_queue.get();
                let head = self.pending_head.get();
                if let Some(entry) = queue[head % MAX_PENDING] {
                    if entry.image_id == image_id {
                        entry
                            .client
                            .image_ready(entry.image_id, entry.load_addr, entry.img_size);
                    }
                }

                // Fire passive watchers registered for this image type.
                let watchers = self.watchers.get();
                for w in watchers.iter().flatten() {
                    if w.image_id == image_id {
                        w.client.image_ready(w.image_id, w.load_addr, w.img_size);
                    }
                }

                CommandReturn::success()
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, pid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(pid, |_, _| {})
    }
}
