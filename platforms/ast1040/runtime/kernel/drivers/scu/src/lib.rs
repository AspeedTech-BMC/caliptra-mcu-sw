// Licensed under the Apache-2.0 license

#![no_std]

use caliptra_mcu_capsules_runtime::bootdev::hil::{KernelImageClient, KernelImageLoader};
use kernel::utilities::registers::interfaces::{ReadWriteable, Writeable};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use registers_peripherals::ast1040::aspeed_scu1::{PspCtrl1, PspCtrl7, SCU1Registers};

const SSP_ZEPHYR: u32 = 0x100E;

pub struct Ast1040ScuCtrl {
    scu: StaticRef<SCU1Registers>,
}

impl Ast1040ScuCtrl {
    pub const fn new(base: StaticRef<SCU1Registers>) -> Self {
        Self { scu: base }
    }

    /// Register as a watcher for SSP_ZEPHYR: when userspace loads that image,
    /// `image_ready` fires and initializes PSP cache, RAM, and releases reset.
    pub fn init(&'static self, loader: &dyn KernelImageLoader) -> Result<(), ErrorCode> {
        loader.watch_image(SSP_ZEPHYR, self)
    }

    fn init_psp(&self) {
        let scu = &self.scu;

        // I-cache block enable: all 32 regions cacheable
        scu.psp_icache.set(0xffff_ffff);

        // D-cache block enable: all 32 regions cacheable
        scu.psp_dcache.set(0xffff_ffff);

        // Enable I-cache and D-cache
        scu.psp_ctrl7
            .modify(PspCtrl7::ICACHE_EN::SET + PspCtrl7::DCACHE_EN::SET);

        // Remap to hyper RAM base 0
        scu.psp_remap_mem.set(0x0);

        // Configure PSP RAM size: 14 MB
        scu.psp_ram_size.set(0x00e0_0000);

        // Disable uart interrupt
        unsafe {
            core::ptr::write_volatile(0x3413_0210 as *mut u32, 0);
        }

        // Release reset and trigger PSP enable
        scu.psp_ctrl1
            .modify(PspCtrl1::RESET::SET + PspCtrl1::ENABLE::SET);
    }
}

impl KernelImageClient for Ast1040ScuCtrl {
    fn image_ready(&self, image_id: u32, _addr: u64, _size: u32) {
        if image_id == SSP_ZEPHYR {
            self.init_psp();
        }
    }
}
