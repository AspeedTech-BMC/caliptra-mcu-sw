// Licensed under the Apache-2.0 license

use tock_registers::registers::ReadWrite;
use tock_registers::{self, register_bitfields, register_structs};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EdafBridgeRoute {
    Espi0,
    Espi1,
}

register_structs! {
    pub EdafBridgeRegisters {
        /// EDAF bridge configuration register.
        (0x0000 => pub cfg: ReadWrite<u32, EdafBridgeCfg::Register>),
        (0x0004 => _reserved004),
        /// Controller base address low 32 bits.
        (0x0020 => pub cbase: ReadWrite<u32>),
        /// Memory base address low 32 bits.
        (0x0024 => pub mbase: ReadWrite<u32>),
        (0x0028 => _reserved028),
        /// SPI read command value.
        (0x0040 => pub cmd_read: ReadWrite<u32>),
        /// SPI write-enable command value.
        (0x0044 => pub cmd_write_enable: ReadWrite<u32>),
        /// SPI write/program command value.
        (0x0048 => pub cmd_write: ReadWrite<u32>),
        /// SPI read-status command value.
        (0x004c => pub cmd_read_status: ReadWrite<u32>),
        /// SPI 4 KiB erase command value.
        (0x0050 => pub cmd_erase_4k: ReadWrite<u32>),
        /// SPI 32 KiB erase command value.
        (0x0054 => pub cmd_erase_32k: ReadWrite<u32>),
        /// SPI 64 KiB erase command value.
        (0x0058 => pub cmd_erase_64k: ReadWrite<u32>),
        (0x005c => _reserved05c),
        /// High address bytes for cbase and mbase.
        (0x0070 => pub misc: ReadWrite<u32, EdafBridgeMisc::Register>),
        (0x0074 => @END),
    }
}

register_structs! {
    pub EdafBridgeGlobalRegisters {
        /// EDAF bridge global register.
        (0x0000 => pub global: ReadWrite<u32, EdafBridgeGlobal::Register>),
        (0x0004 => @END),
    }
}

register_structs! {
    pub EspiChannel3Registers {
        (0x0000 => _reserved000),
        /// eSPI channel 3 control register.
        (0x0400 => pub ch3_ctrl: ReadWrite<u32, EspiCh3Ctrl::Register>),
        (0x0404 => @END),
    }
}

register_bitfields![u32,
pub EdafBridgeCfg [
    /// Enable command mode. Clear this bit for DDR-backed EDAF mode.
    CmdEn OFFSET(0) NUMBITS(1) [
        Disabled = 0,
        Enabled = 1,
    ],
],

pub EdafBridgeMisc [
    /// Memory base address high byte, usually bits [39:32] of mbase.
    MbaseHigh OFFSET(8) NUMBITS(8) [],
    /// Controller base address high byte, usually bits [39:32] of cbase.
    CbaseHigh OFFSET(0) NUMBITS(8) [],
],

pub EdafBridgeGlobal [
    /// Use erased value 1 for EDAF bridge instance 0.
    Bridge0EraseWithOne OFFSET(1) NUMBITS(1) [
        Disabled = 0,
        Enabled = 1,
    ],
    /// Use erased value 1 for EDAF bridge instance 1.
    Bridge1EraseWithOne OFFSET(9) NUMBITS(1) [
        Disabled = 0,
        Enabled = 1,
    ],
],

pub Scu1HostConf [
    /// LPC host interface enable. EDAF is invalid when this bit is set.
    LpcEn OFFSET(9) NUMBITS(1) [],
    /// EDAF host interface enable.
    EdafEn OFFSET(10) NUMBITS(1) [
        Disabled = 0,
        Enabled = 1,
    ],
],

pub EspiCh3Ctrl [
    /// Software-ready indication for eSPI channel 3.
    SwReady OFFSET(5) NUMBITS(1) [
        NotReady = 0,
        Ready = 1,
    ],
],
];
