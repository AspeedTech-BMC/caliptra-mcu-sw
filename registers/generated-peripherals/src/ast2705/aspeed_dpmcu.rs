// Licensed under the Apache-2.0 license

use tock_registers::registers::{ReadOnly, ReadWrite};
use tock_registers::{self, register_bitfields, register_structs};

/// DP MCU data memory base address (BootMCU view; CA35 view: 0x1100_0000)
pub const MCU_DMEM_BASE: u32 = 0x3100_0000;
/// DP MCU register base address (BootMCU view; CA35 view: 0x1101_0000)
pub const MCU_REG_BASE: u32 = 0x3101_0000;
/// DP MCU instruction memory base address (BootMCU view; CA35 view: 0x1102_0000)
pub const MCU_IMEM_BASE: u32 = 0x3102_0000;
/// DisplayPort controller base address (BootMCU view; CA35 view: 0x12c0_a000)
pub const DP_BASE: u32 = 0x32c0_a000;

register_structs! {
    pub DpMcuRegisters {
        (0x0000 => _reserved000),
        /// MCU control register
        (0x00e0 => pub ctrl: ReadWrite<u32, McuCtrl::Register>),
        (0x00e4 => _reservede4),
        /// MCU interrupt control register
        (0x00e8 => pub intr_ctrl: ReadWrite<u32, McuIntrCtrl::Register>),
        (0x00ec => @END),
    }
}

register_structs! {
    pub DpRegisters {
        (0x0000 => _reserved000),
        /// DisplayPort version register
        (0x001c => pub version: ReadOnly<u32>),
        (0x0020 => _reserved020),
        /// DisplayPort handshake register
        (0x00b8 => pub handshake: ReadWrite<u32, DpHandshake::Register>),
        (0x00bc => @END),
    }
}

register_bitfields![u32,
/// MCU control register
pub McuCtrl [
    /// Enable AHB slave access to IMEM
    AhbsImemEn   OFFSET(0)  NUMBITS(1) [],
    /// AHB slave software reset
    AhbsSwRst    OFFSET(4)  NUMBITS(1) [],
    /// AHB master software reset
    AhbmSwRst    OFFSET(8)  NUMBITS(1) [],
    /// MCU core software reset
    CoreSwRst    OFFSET(12) NUMBITS(1) [],
    /// Shut down DMEM
    DmemShutDown OFFSET(16) NUMBITS(1) [],
    /// Put DMEM into sleep mode
    DmemSleep    OFFSET(17) NUMBITS(1) [],
    /// Turn off DMEM clock
    DmemClkOff   OFFSET(18) NUMBITS(1) [],
    /// Shut down IMEM
    ImemShutDown OFFSET(20) NUMBITS(1) [],
    /// Put IMEM into sleep mode
    ImemSleep    OFFSET(21) NUMBITS(1) [],
    /// Turn off IMEM clock
    ImemClkOff   OFFSET(22) NUMBITS(1) [],
    /// Select IMEM source
    ImemSel      OFFSET(24) NUMBITS(1) [],
    /// MCU configuration bit
    Config       OFFSET(28) NUMBITS(1) [],
],
/// MCU interrupt control register
pub McuIntrCtrl [
    /// Interrupt clear bits [7:0]
    Clr  OFFSET(0)  NUMBITS(8) [],
    /// Interrupt mask bits [15:8]
    Mask OFFSET(8)  NUMBITS(8) [],
    /// Interrupt enable bits [23:16]
    En   OFFSET(16) NUMBITS(8) [],
],
/// DisplayPort handshake register
pub DpHandshake [
    /// Video format source select
    VideoFmtSrc  OFFSET(24) NUMBITS(1) [],
    /// Host read EDID flag
    HostReadEdid OFFSET(28) NUMBITS(1) [],
],
];
