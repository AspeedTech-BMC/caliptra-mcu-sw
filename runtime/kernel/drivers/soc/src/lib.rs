// Licensed under the Apache-2.0 license

#![no_std]

use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::StaticRef;
use registers_peripherals::aspeed_soc::{AhbMsbOffset, LsuMbusMap, MbusMsbOffset, SoCRegisters};

pub struct AspeedSoCCtrl {
    reg: StaticRef<SoCRegisters>,
}

impl AspeedSoCCtrl {
    pub const fn new(base: StaticRef<SoCRegisters>) -> Self {
        Self { reg: base }
    }

    pub fn set_ahb_msb_offset(&self, msb: u8) {
        self.reg
            .ahb_msb_offset
            .write(AhbMsbOffset::MsbOffset0.val(msb as u32));
    }

    pub fn set_mbus_msb_offsets(&self, msb0: u8, msb1: u8, msb2: u8, msb3: u8) {
        self.reg.mbus_msb_offset.write(
            MbusMsbOffset::MsbOffset0.val(msb0 as u32)
                + MbusMsbOffset::MsbOffset1.val(msb1 as u32)
                + MbusMsbOffset::MsbOffset2.val(msb2 as u32)
                + MbusMsbOffset::MsbOffset3.val(msb3 as u32),
        );
    }

    pub fn set_lsu_mbus_map(&self, msb0: u8, msb1: u8, msb2: u8, msb3: u8) {
        self.reg.lsu_mbus_map.write(
            LsuMbusMap::MsbLsuMbusMap0.val(msb0 as u32)
                + LsuMbusMap::MsbLsuMbusMap1.val(msb1 as u32)
                + LsuMbusMap::MsbLsuMbusMap2.val(msb2 as u32)
                + LsuMbusMap::MsbLsuMbusMap3.val(msb3 as u32),
        );
    }
}

/// Borrows/restores the shared AHBC aperture for controllers with no dedicated window (e.g. SPI0/1/2).
pub trait AhbWindowGate {
    fn engage(&self) -> u8;
    fn restore(&self, prev: u8);
}

/// `AhbWindowGate` over the SoC's `ahb_msb_offset` register.
#[derive(Clone, Copy)]
pub struct AhbApertureGate {
    reg: StaticRef<SoCRegisters>,
    target: u8,
}

impl AhbApertureGate {
    pub const fn new(reg: StaticRef<SoCRegisters>, target: u8) -> Self {
        Self { reg, target }
    }
}

impl AhbWindowGate for AhbApertureGate {
    fn engage(&self) -> u8 {
        let prev = self.reg.ahb_msb_offset.read(AhbMsbOffset::MsbOffset0) as u8;
        self.reg
            .ahb_msb_offset
            .write(AhbMsbOffset::MsbOffset0.val(self.target as u32));
        prev
    }

    fn restore(&self, prev: u8) {
        self.reg
            .ahb_msb_offset
            .write(AhbMsbOffset::MsbOffset0.val(prev as u32));
    }
}
