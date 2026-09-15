// Licensed under the Apache-2.0 license

#![no_std]

use caliptra_mcu_capsules_runtime::bootdev::hil::{KernelImageClient, KernelImageLoader};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Writeable};
use kernel::utilities::registers::Field;
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use registers_peripherals::ast2705::aspeed_scu0::SCU0Registers;
use registers_peripherals::ast2705::aspeed_scu1::{PinMux, Rst1, SCU1Registers};

/// Image type for CA35 U-Boot — the last CA35 image before CPU release.
const CA35_UBOOT: u32 = 0x100A;

pub struct AspeedScuCtrl {
    reg: StaticRef<SCU0Registers>,
    loader: OptionalCell<&'static dyn KernelImageLoader>,
}

impl AspeedScuCtrl {
    pub const fn new(base: StaticRef<SCU0Registers>) -> Self {
        Self {
            reg: base,
            loader: OptionalCell::empty(),
        }
    }

    /// Register this driver as a passive watcher for the CA35_UBOOT image.
    /// When that image is loaded, `image_ready` fires and releases the CA35 CPUs.
    pub fn init(&'static self, loader: &'static dyn KernelImageLoader) -> Result<(), ErrorCode> {
        self.loader.set(loader);
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

        // Clear SMP 1/2/3 entry point
        self.reg.cpu_smp_ep1.set(0);
        self.reg.cpu_smp_ep2.set(0);
        self.reg.cpu_smp_ep3.set(0);

        // Configure SMP 0 entry point
        self.reg.cpu_smp_ep0.set(ep);

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
            // The release is synchronous: report init completion so the
            // waiting app's INIT_DONE fires.
            self.loader.map(|loader| {
                loader.notify_init_done(image_id, true);
            });
        }
    }
}

// SCU1 (SPI pinmux/reset), separate from AspeedScuCtrl above (SCU0).

#[derive(Clone, Copy)]
pub enum ScuSignal {
    Spi0Quad,
    Spi1Quad,
    Spi2Quad,
}

#[derive(Clone, Copy)]
pub enum ScuReset {
    Spi2,
}

pub struct AspeedScu1Ctrl {
    reg: StaticRef<SCU1Registers>,
}

impl AspeedScu1Ctrl {
    pub const fn new(base: StaticRef<SCU1Registers>) -> Self {
        Self { reg: base }
    }

    /// Route the SCU1 pinmux pads needed for `sig`.
    pub fn pinmux_apply(&self, sig: ScuSignal) {
        match sig {
            ScuSignal::Spi0Quad => {
                // Group N: CLK, MOSI (DQ0), MISO (DQ1), DQ2, DQ3, CS1
                for pin in 0..6 {
                    self.reg
                        .pinmux_n
                        .modify(Field::<u32, PinMux::Register>::new(0x7, pin * 4).val(1));
                }
            }
            ScuSignal::Spi1Quad => {
                // Group O: CLK, MOSI (DQ0), MISO (DQ1), DQ2, DQ3, CS1
                for pin in 0..6 {
                    self.reg
                        .pinmux_o
                        .modify(Field::<u32, PinMux::Register>::new(0x7, pin * 4).val(1));
                }
            }
            ScuSignal::Spi2Quad => {
                // Group P: CS0, CLK, MOSI (DQ0), MISO (DQ1), DQ2, DQ3, CS1
                for pin in 0..7 {
                    self.reg
                        .pinmux_p
                        .modify(Field::<u32, PinMux::Register>::new(0x7, pin * 4).val(1));
                }
            }
        }
    }

    /// Deassert the reset line for `id`.
    pub fn reset_deassert(&self, id: ScuReset) {
        match id {
            ScuReset::Spi2 => self.reg.rstctl1_clr.set(Rst1::SPI2::SET.value),
        }
    }
}
