// Licensed under the Apache-2.0 license

pub mod hil;

use self::hil::{ImageFlash, KernelImageClient, KernelImageLoader};
use core::cell::Cell;
use core::marker::PhantomData;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{ErrorCode, ProcessId};

pub const DRIVER_NUM: usize = 0x9000_1000;

mod upcall {
    /// Kernel → app: driver needs image. args: (image_id, img_size, 0).
    /// image_id == 0 is a sentinel meaning no more pending requests.
    pub const IMAGE_NEEDED: usize = 0;
    /// Kernel → app: hardware init complete. args: (image_id, result, all_done).
    pub const INIT_DONE: usize = 1;
    /// Kernel → app: READ_IMAGE transfer done. args: (0, 0, 0).
    pub const READ_DONE: usize = 2;
    pub const COUNT: u8 = 3;
}

mod cmd {
    pub const EXISTS: usize = 0;
    pub const APP_READY: usize = 1;
    pub const IMAGE_LOADED: usize = 2;
    pub const IMAGE_WAITTING: usize = 3;
    pub const IMAGE_INITIAL: usize = 4;
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
    // Userspace app
    apps: Grant<App, UpcallCount<{ upcall::COUNT }>, AllowRoCount<0>, AllowRwCount<0>>,
    active_app: OptionalCell<ProcessId>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> BootDevCapsule<'a> {
    pub fn new(
        devs: &[&'static dyn ImageFlash],
        grant: Grant<App, UpcallCount<{ upcall::COUNT }>, AllowRoCount<0>, AllowRwCount<0>>,
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

            cmd::APP_READY => {
                if self.active_app.is_some() {
                    return CommandReturn::failure(ErrorCode::BUSY);
                }
                self.active_app.set(pid);
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

                let devs = self.get_flash_devs();
                let mut img_size = None;
                for dev in devs.iter().flatten() {
                    let map_addr = dev.map_to_axi(addr as u64);
                    if let Ok(sz) = dev.load_image(image_id, map_addr as u32) {
                        img_size = Some(sz);
                        break;
                    }
                }
                let img_size = match img_size {
                    None => return CommandReturn::failure(ErrorCode::FAIL),
                    Some(sz) => sz,
                };

                // Persist IPC-confirmed size and load address into the active pending queue entry.
                let mut queue = self.pending_queue.get();
                let head = self.pending_head.get();
                if let Some(ref mut entry) = queue[head % MAX_PENDING] {
                    entry.img_size = img_size;
                    entry.load_addr = addr as u64;
                }
                self.pending_queue.set(queue);

                // Unblock the userspace image_loaded() async call
                if let Some(pid) = self.active_app.get() {
                    let _ = self.apps.enter(pid, |_, kd| {
                        kd.schedule_upcall(
                            upcall::READ_DONE,
                            (image_id as usize, img_size as usize, 0),
                        )
                        .ok();
                    });
                }

                CommandReturn::success()
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
