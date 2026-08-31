// Licensed under the Apache-2.0 license

use tock_registers::registers::ReadWrite;
use tock_registers::{self, register_structs};

/// CPU-die PCIe E2M node 0 base address (BootMCU view; CA35 view: 0x12c2_1000)
pub const E2M0_BASE: u32 = 0x32c2_1000;
/// CPU-die PCIe E2M node 1 base address (BootMCU view; CA35 view: 0x12c2_2000)
pub const E2M1_BASE: u32 = 0x32c2_2000;

register_structs! {
    pub E2m0Registers {
        (0x0000 => _reserved000),
        /// VGA framebuffer aperture mapping for E2M node 0
        (0x0100 => pub vga_ram: ReadWrite<u32>),
        /// VBIOS aperture mapping for E2M node 0
        (0x0104 => pub vbios_ram: ReadWrite<u32>),
        (0x0108 => @END),
    }
}

register_structs! {
    pub E2m1Registers {
        (0x0000 => _reserved000),
        /// VGA framebuffer aperture mapping for E2M node 1
        (0x0120 => pub vga_ram: ReadWrite<u32>),
        /// VBIOS aperture mapping for E2M node 1
        (0x0124 => pub vbios_ram: ReadWrite<u32>),
        (0x0128 => @END),
    }
}
