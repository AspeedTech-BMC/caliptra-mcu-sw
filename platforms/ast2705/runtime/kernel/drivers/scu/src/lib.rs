// Licensed under the Apache-2.0 license

#![no_std]

use caliptra_mcu_capsules_runtime::bootdev::hil::{KernelImageClient, KernelImageLoader};
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use registers_peripherals::ast2705::aspeed_scu0::SCU0Registers;

/// Image type for CA35 U-Boot — the last CA35 image before CPU release.
const CA35_UBOOT: u32 = 0x100A;

pub struct AspeedScuCtrl {
    reg: StaticRef<SCU0Registers>,
}

impl AspeedScuCtrl {
    pub const fn new(base: StaticRef<SCU0Registers>) -> Self {
        Self { reg: base }
    }

    /// Register this driver as a passive watcher for the CA35_UBOOT image.
    /// When that image is loaded, `image_ready` fires and releases the CA35 CPUs.
    pub fn init(&'static self, loader: &dyn KernelImageLoader) -> Result<(), ErrorCode> {
        loader.watch_image(CA35_UBOOT, self)
    }

    /// Release all four CA35 cores from reset.
    ///
    /// `boot_addr` is written to RVBAR0–RVBAR3 as the reset vector.
    pub fn release_ca35(&self, boot_addr: u32, ep: u64) {
        // CA35 reset vector configure
        self.reg.psp_rvbar0.set(boot_addr);
        self.reg.psp_rvbar1.set(boot_addr);
        self.reg.psp_rvbar2.set(boot_addr);
        self.reg.psp_rvbar3.set(boot_addr);

        // SMP 0 entry point configure
        self.reg.cpu_smp_ep_lo.set(ep as u32);
        self.reg.cpu_smp_ep_hi.set((ep >> 32) as u32);

        self.reg.ca35_release.set(1);
    }

    /// Clear clock-stop bits. `mask` selects which clocks to un-gate.
    pub fn clock_stop_clr(&self, mask: u32) {
        self.reg.clock_stop_clr.set(mask);
    }
}

impl KernelImageClient for AspeedScuCtrl {
    fn image_ready(&self, image_id: u32, _addr: u64, _size: u32) {
        if image_id == CA35_UBOOT {
            self.release_ca35(0x43000000, 0x4_0000_0000);
        }
    }

    fn image_error(&self, _err: ErrorCode) {}
}
