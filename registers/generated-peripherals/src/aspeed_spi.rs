// Licensed under the Apache-2.0 license

use tock_registers::registers::ReadWrite;
use tock_registers::{self, register_bitfields, register_structs};

register_structs! {
    /// ASPEED FMC/SPI register map.
    pub AspeedSpiRegisters {
        (0x000 => pub conf: ReadWrite<u32, Conf::Register>), // FMC00
        (0x004 => pub ce_ctrl: ReadWrite<u32, CeCtrl::Register>), // FMC04
        (0x008 => pub intr_ctrl: ReadWrite<u32, IntrCtrl::Register>), // FMC08
        (0x00C => pub cmd_ctrl: ReadWrite<u32, CmdCtrl::Register>), // FMC0C
        (0x010 => pub ce0_ctrl: ReadWrite<u32, Ce0Ctrl::Register>), // FMC10
        (0x014 => pub ce1_ctrl: ReadWrite<u32, Ce0Ctrl::Register>), // FMC14
        (0x018 => pub ce2_ctrl: ReadWrite<u32, Ce0Ctrl::Register>), // FMC18
        (0x01C => pub ce3_ctrl: ReadWrite<u32, Ce0Ctrl::Register>), // FMC1C
        (0x020 => _reserved20),
        (0x030 => pub ce0_addr_decoding: ReadWrite<u32>), // FMC30
        (0x034 => pub ce1_addr_decoding: ReadWrite<u32>), // FMC34
        (0x038 => pub ce2_addr_decoding: ReadWrite<u32>), // FMC38
        (0x03C => pub ce3_addr_decoding: ReadWrite<u32>), // FMC3C
        (0x040 => _reserved40),
        (0x07C => pub dma_hi_addr: ReadWrite<u32>), // FMC7C: AST2700-style, always 0 on AST1040
        (0x080 => pub dma_ctrl: ReadWrite<u32, DmaCtrl::Register>), // FMC80
        (0x084 => pub dma_flash_addr: ReadWrite<u32>), // FMC84
        (0x088 => pub dma_ram_addr: ReadWrite<u32>), // FMC88
        (0x08C => pub dma_len: ReadWrite<u32>), // FMC8C: transfer length minus 1
        (0x090 => _reserved90),
        (0x0A0 => pub cs_swap_ctrl: ReadWrite<u32, CsSwapCtrl::Register>), // FMCA0
        (0x0A4 => _reserveda4),
        (0x1F4 => pub fmc_rst_wlock1: ReadWrite<u32, RstWlock1::Register>), // FMC1F4
        (0x1F8 => @END),
    }
}

register_bitfields![u32,
    /// 0x00 CE Type Setting Register
    pub Conf [
        /// Legacy (AST2400) single write-enable bit for all CS.
        LegacyWriteEnable OFFSET(0) NUMBITS(1) [],
        /// Per-CS write enable, one bit per chip select (CE0..CE4).
        CeWriteEnable OFFSET(16) NUMBITS(5) [],
    ],

pub IntrCtrl [
    DmaIrqEn  OFFSET(3)  NUMBITS(1) [],
    DmaSts    OFFSET(11) NUMBITS(1) [], // read-only
    DmaIrqSts OFFSET(19) NUMBITS(1) [], // write 1 to clear
],

pub DmaCtrl [
    Enable OFFSET(0) NUMBITS(1) [],
    Write  OFFSET(1) NUMBITS(1) [], // 0 = flash->RAM, 1 = RAM->flash
],

pub CeCtrl [
    Ce0Mode4B    OFFSET(0) NUMBITS(1) [],
    Ce1Mode4B    OFFSET(1) NUMBITS(1) [],
    Ce2Mode4B    OFFSET(2) NUMBITS(1) [],
    Ce3Mode4B    OFFSET(3) NUMBITS(1) [],
    Ce0Mode4BExt OFFSET(4) NUMBITS(1) [],
    Ce1Mode4BExt OFFSET(5) NUMBITS(1) [],
    Ce2Mode4BExt OFFSET(6) NUMBITS(1) [],
    Ce3Mode4BExt OFFSET(7) NUMBITS(1) [],
],

pub CmdCtrl [
    Force3B OFFSET(7) NUMBITS(1) [], // force 3-byte address (needed for SFDP)
],

/// CE0/CE1/CE2 control register (FMC10/FMC14/FMC18).
pub Ce0Ctrl [
    Mode OFFSET(0) NUMBITS(2) [
        AutoRead    = 0,
        NormalRead  = 1,
        NormalWrite = 2,
        UserMode    = 3,
    ],
    CsStop OFFSET(2) NUMBITS(1) [
        Active   = 1,
        Inactive = 0,
    ],
    // Dummy cycle count = {FrdWaitHi, FrdWaitLo}: 0..7 dummy bytes.
    FrdWaitLo OFFSET(6)  NUMBITS(2) [],
    ClkFreqLo OFFSET(8)  NUMBITS(4) [],
    FrdWaitHi OFFSET(14) NUMBITS(1) [],
    Cmd       OFFSET(16) NUMBITS(8) [],
    ClkFreqHi OFFSET(24) NUMBITS(4) [],
],

pub CsSwapCtrl [
    SwapByWdt OFFSET(0) NUMBITS(1) [], // enable CS0/CS1 swap triggered by WDT scratch reg
],

pub RstWlock1 [
    CsSwapCtrlRstLock OFFSET(31) NUMBITS(1) [], // lock CS-swap ctrl across SoC reset
],
];
