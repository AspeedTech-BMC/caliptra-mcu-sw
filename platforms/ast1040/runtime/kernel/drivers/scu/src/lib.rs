// Licensed under the Apache-2.0 license

#![no_std]

use caliptra_mcu_capsules_runtime::bootdev::hil::{KernelImageClient, KernelImageLoader};
use kernel::utilities::registers::interfaces::{ReadWriteable, Writeable};
use kernel::utilities::registers::Field;
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use registers_peripherals::ast1040::aspeed_scu1::{PinMux, PspCtrl1, PspCtrl7, SCU1Registers};

const SSP_ZEPHYR: u32 = 0x100E;

#[derive(Clone, Copy)]
pub enum ScuSignal {
    Spi0Quad,
    Spi1Quad,
}

pub struct Ast1040ScuCtrl {
    scu: StaticRef<SCU1Registers>,
}

impl Ast1040ScuCtrl {
    pub const fn new(base: StaticRef<SCU1Registers>) -> Self {
        Self { scu: base }
    }

    /// Route the SCU pinmux pads needed for `sig`. SPI0/SPI1 pins are
    /// scattered across several pinmux groups (unlike AST2705's SPI0/1/2,
    /// which each fit a contiguous range within one group register), so
    /// each pad is set individually rather than via a range loop.
    pub fn pinmux_apply(&self, sig: ScuSignal) {
        match sig {
            ScuSignal::Spi0Quad => {
                // Group 21: SPIM0CK (pin 6), SPIM0DQ0 (pin 7)
                self.scu
                    .pinmux_grp21
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 24).val(1));
                self.scu
                    .pinmux_grp21
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 28).val(1));
                // Group 22: SPIM0DQ1 (pin 0), SPIM0DQ2 (pin 1), SPIM0DQ3 (pin 2)
                self.scu
                    .pinmux_grp22
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 0).val(1));
                self.scu
                    .pinmux_grp22
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 4).val(1));
                self.scu
                    .pinmux_grp22
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 8).val(1));
                // Group 24: SPIM0CS0 (pin 7); Group 25: SPIM0CS1 (pin 0)
                self.scu
                    .pinmux_grp24
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 28).val(1));
                self.scu
                    .pinmux_grp25
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 0).val(1));
            }
            ScuSignal::Spi1Quad => {
                // Group 19: SPIM1CK (pin 2), SPIM1DQ0 (pin 3), SPIM1DQ1 (pin 4),
                // SPIM1DQ2 (pin 5), SPIM1DQ3 (pin 6)
                self.scu
                    .pinmux_grp19
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 8).val(1));
                self.scu
                    .pinmux_grp19
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 12).val(1));
                self.scu
                    .pinmux_grp19
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 16).val(1));
                self.scu
                    .pinmux_grp19
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 20).val(1));
                self.scu
                    .pinmux_grp19
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 24).val(1));
                // Group 24: SPIM1CS0 (pin 5), SPIM1CS1 (pin 6)
                self.scu
                    .pinmux_grp24
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 20).val(1));
                self.scu
                    .pinmux_grp24
                    .modify(Field::<u32, PinMux::Register>::new(0x7, 24).val(1));
            }
        }
    }

    /// Register as a watcher for SSP_ZEPHYR: when userspace loads that image,
    /// `image_ready` fires and initializes PSP cache, RAM, and releases reset.
    pub fn init(&'static self, loader: &dyn KernelImageLoader) -> Result<(), ErrorCode> {
        loader.watch_image(SSP_ZEPHYR, self)
    }

    fn init_psp(&self) {
        let scu = &self.scu;

        scu.psp_ctrl1.modify(
            PspCtrl1::SRAM_SLP::CLEAR + PspCtrl1::SRAM_DSLP::CLEAR + PspCtrl1::SRAM_SD::CLEAR,
        );

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
