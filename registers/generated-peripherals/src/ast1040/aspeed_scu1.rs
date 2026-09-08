// Licensed under the Apache-2.0 license

use tock_registers::registers::{ReadOnly, ReadWrite};
use tock_registers::{self, register_bitfields, register_structs};

register_structs! {
    pub SCU1Registers {
        (0x0000 => _reserved0),
        (0x0010 => pub strap: ReadWrite<u32, OTPStrap::Register>),
        (0x0014 => _reserved14),
        (0x0114 => pub rst_vector: ReadWrite<u32>),
        (0x0118 => _reserved118),
        // Reset Control 1
        (0x0200 => pub rstctl1: ReadWrite<u32, Rst1::Register>),
        // Reset Clear 1
        (0x0204 => pub rstctl1_clr: ReadWrite<u32, Rst1::Register>),
        (0x0208 => _reserved208),
        // Clock Stop Clear 1
        (0x0244 => pub clk1_stop_clr: ReadWrite<u32, Clk1::Register>),
        (0x0248 => _reserved248),
        // SCU280: Clock selection register 1
        (0x0280 => pub clk_sel1: ReadWrite<u32, ClkSel1::Register>),
        (0x0284 => _reserved284),
        /// PinMux Group 8
        (0x041C => pub pinmux_grp8: ReadWrite<u32, PinMux::Register>),
        (0x0420 => _reserved420),
        /// PinMux Group 10
        (0x0424 => pub pinmux_grp10: ReadWrite<u32, PinMux::Register>),
        /// PinMux Group 11
        (0x0428 => pub pinmux_grp11: ReadWrite<u32, PinMux::Register>),
        (0x042C => _reserved42c),
        /// PinMux Group 19
        (0x0448 => pub pinmux_grp19: ReadWrite<u32, PinMux::Register>),
        /// PinMux Group 20
        (0x044C => pub pinmux_grp20: ReadWrite<u32, PinMux::Register>),
        /// PinMux Group 21
        (0x0450 => pub pinmux_grp21: ReadWrite<u32, PinMux::Register>),
        /// PinMux Group 22
        (0x0454 => pub pinmux_grp22: ReadWrite<u32, PinMux::Register>),
        (0x0458 => _reserved458),
        /// PinMux Group 24
        (0x045C => pub pinmux_grp24: ReadWrite<u32, PinMux::Register>),
        /// PinMux Group 25
        (0x0460 => pub pinmux_grp25: ReadWrite<u32, PinMux::Register>),
        (0x0464 => _reserved464),
        (0x0800 => pub otpcfg00: ReadOnly<u32, OtpCfg00::Register>),
        (0x0804 => _reserved804),
        (0x0858 => pub otpcfg22: ReadOnly<u32, OtpCfg22::Register>),
        (0x085C => _reserved85c),
        // SCU8FC: Scratch register for MCU 63
        (0x08FC => pub scratch_mcu63: ReadWrite<u32>),
        // SCU900: PSP Service Processor Control Register 1  (Init = 0x0000_00FE)
        (0x0900 => pub psp_ctrl1: ReadWrite<u32, PspCtrl1::Register>),
        // SCU904: PSP REMAP AHB BASE
        (0x0904 => pub psp_remap_ahb: ReadWrite<u32>),
        // SCU908: PSP REMAP MEM BASE
        (0x0908 => pub psp_remap_mem: ReadWrite<u32>),
        // SCU90C: PSP I CACHEABLE control
        (0x090C => pub psp_icache: ReadWrite<u32>),
        // SCU910: PSP D CACHEABLE control
        (0x0910 => pub psp_dcache: ReadWrite<u32>),
        // SCU914: PSP Cache Invalid control
        (0x0914 => pub psp_cache_inv: ReadWrite<u32, PspCacheInv::Register>),
        // SCU918: PSP Control Register 7
        (0x0918 => pub psp_ctrl7: ReadWrite<u32, PspCtrl7::Register>),
        (0x091C => _reserved91c),
        // SCU934: PSP RAM size configuration
        (0x0934 => pub psp_ram_size: ReadWrite<u32>),
        (0x0938 => @END),
    }
}

register_bitfields![u32,
pub OTPStrap [
    /// Boot source strap enable.
    BootSrcEn OFFSET(2) NUMBITS(1) [],
    LowSecure OFFSET(5) NUMBITS(1) [],
    /// Boot source select (0 = I3C).
    BootSrcSel OFFSET(16) NUMBITS(2) [],
],

/// Reset Control/Clear 1
pub Rst1 [
    /// Reset bit for MDC/MDIO Controller
    PMI   OFFSET(2) NUMBITS(1) [],
    /// Reset bit for MAC
    MAC   OFFSET(5) NUMBITS(1) [],
],

/// Clock Stop 1
pub Clk1 [
    /// Clock gate for MAC
    MAC   OFFSET(8) NUMBITS(1) [],
],

/// Clock selection 1
pub ClkSel1 [
    /// Memory (hyper RAM) clock divider selection.
    MEMCLK_DIV OFFSET(14) NUMBITS(3) [],
    /// RGMIICLK
    RGMIICLK OFFSET(25) NUMBITS(3) [],
    /// MHCLK
    MHCLK    OFFSET(29) NUMBITS(3) [],
],

/// PinMux Group
pub PinMux [
    MUX0  OFFSET(0) NUMBITS(3) [],
    MUX1  OFFSET(4) NUMBITS(3) [],
    MUX2  OFFSET(8) NUMBITS(3) [],
    MUX3  OFFSET(12) NUMBITS(3) [],
    MUX4  OFFSET(16) NUMBITS(3) [],
    MUX5  OFFSET(20) NUMBITS(3) [],
    MUX6  OFFSET(24) NUMBITS(3) [],
    MUX7  OFFSET(28) NUMBITS(3) [],
],

pub OtpCfg00 [
    /// Disable recvoery boot
    DisRecoveryBoot OFFSET(17) NUMBITS(1) [],
],

pub OtpCfg22 [
    /// Hyper RAM ECC enable strap from OTP.
    HYPERECC_ENABLE OFFSET(0) NUMBITS(1) [],
],

pub PspCtrl1 [
    /// Write 1 to enable PSP Service Processor (write-trigger)
    ENABLE      OFFSET(0)  NUMBITS(1) [],
    /// PSP Service Processor Reset (1 = held in reset, init = 1)
    RESET       OFFSET(1)  NUMBITS(1) [],
    DBG_ENABLE  OFFSET(2)  NUMBITS(1) [],
    DBGEN       OFFSET(3)  NUMBITS(1) [],
    NIDEN       OFFSET(4)  NUMBITS(1) [],
    SRAM_SLP    OFFSET(5)  NUMBITS(1) [],
    SRAM_DSLP   OFFSET(6)  NUMBITS(1) [],
    SRAM_SD     OFFSET(7)  NUMBITS(1) [],
    /// RO: PSP reset readback
    ENRESET     OFFSET(8)  NUMBITS(1) [],
    /// RO: PSP reset hold readback
    CRESET      OFFSET(9)  NUMBITS(1) [],
    /// RO: PSP WDT reset source
    WDT_RESET   OFFSET(10) NUMBITS(1) [],
],

pub PspCacheInv [
    I_CACHE_INVALID_ADDR OFFSET(0)  NUMBITS(13) [],
    I_CACHE_INVALID_CMD  OFFSET(15) NUMBITS(1)  [],
    D_CACHE_INVALID_ADDR OFFSET(16) NUMBITS(13) [],
    D_CACHE_INVALID_CMD  OFFSET(31) NUMBITS(1)  [],
],

pub PspCtrl7 [
    ICACHE_EN       OFFSET(0) NUMBITS(1) [],
    DCACHE_EN       OFFSET(1) NUMBITS(1) [],
    SRAM_MASTER_SEL OFFSET(7) NUMBITS(1) [],
],
];
