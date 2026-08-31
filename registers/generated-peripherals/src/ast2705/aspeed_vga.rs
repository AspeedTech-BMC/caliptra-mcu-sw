// Licensed under the Apache-2.0 license

use tock_registers::registers::ReadWrite;
use tock_registers::{self, register_structs};

/// VGA packer (CPU-die) base address (BootMCU view; CA35 view: 0x12c1_d000)
pub const VGA_PACKER_CPU_BASE: u32 = 0x32c1_d000;
/// VGA retimer (CPU-die) base address (BootMCU view; CA35 view: 0x12c1_d100)
pub const VGA_RETIMER_CPU_BASE: u32 = 0x32c1_d100;
/// VGA packer (IO-die) base address (BootMCU view; CA35 view: 0x14c3_a000)
pub const VGA_PACKER_IO_BASE: u32 = 0x34c3_a000;
/// VGA retimer (IO-die) base address (BootMCU view; CA35 view: 0x14c3_a100)
pub const VGA_RETIMER_IO_BASE: u32 = 0x34c3_a100;

// Packer and retimer instances share the same register layout.
register_structs! {
    pub VgaLinkRegisters {
        (0x0000 => _reserved000),
        /// Control register I: packer/retimer enable, polarity, bypass bits
        (0x0010 => pub reg10: ReadWrite<u32>),
        (0x0014 => _reserved014),
        /// ENC counterpart config (packer only)
        (0x001c => pub reg1c: ReadWrite<u32>),
        (0x0020 => _reserved020),
        /// WAIT_FRAME config/hold
        (0x0044 => pub reg44: ReadWrite<u32>),
        (0x0048 => _reserved048),
        /// SRC_OVERWRITE select/enable (packer only)
        (0x0050 => pub reg50: ReadWrite<u32>),
        (0x0054 => @END),
    }
}
