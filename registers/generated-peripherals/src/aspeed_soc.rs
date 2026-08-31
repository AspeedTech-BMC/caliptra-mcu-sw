// Licensed under the Apache-2.0 license

use tock_registers::registers::ReadWrite;
use tock_registers::{self, register_bitfields, register_structs};

register_structs! {
    pub SoCRegisters {
        (0x0000 => _reserved0),
        (0x0060 => pub mcu_lsu_axi_user: ReadWrite<u32>),
        (0x0064 => pub mcu_ifu_axi_user: ReadWrite<u32>),
        (0x0068 => pub mci_sram_config_axi_user: ReadWrite<u32>),
        (0x006c => pub caliptra_dma_axi_user: ReadWrite<u32>),
        (0x0070 => pub mci_soc_config_axi_user: ReadWrite<u32>),
        (0x0074 => _reserved74),
        (0x0088 => pub jtag_sel: ReadWrite<u32>),
        (0x008C => pub ahb_msb_offset: ReadWrite<u32, AhbMsbOffset::Register>),
        (0x0090 => _reserved90),
        (0x0094 => pub mbus_msb_offset: ReadWrite<u32, MbusMsbOffset::Register>),
        (0x0098 => _reserved98),
        (0x009C => pub rst: ReadWrite<u32, Rst::Register>),
        (0x00A0 => _reserveda0),
        (0x0140 => pub cache_ctrl: ReadWrite<u32, CacheCtrl::Register>),
        (0x0144 => pub global_cache_ctrl: ReadWrite<u32, GlobalCacheCtrl::Register>),
        (0x0148 => pub lsu_mbus_map: ReadWrite<u32, LsuMbusMap::Register>),
        (0x014C => _reserved14c),
        (0x0174 => pub cptra_strap_lc_ocp_lock: ReadWrite<u32, strap_lc_ocp_lock::Register>),
        (0x0178 => _reserved178),
        (0x01B0 => pub cache_start0: ReadWrite<u32>),
        (0x01B4 => pub cache_end0: ReadWrite<u32>),
        (0x01B8 => pub cache_start1: ReadWrite<u32>),
        (0x01BC => pub cache_end1: ReadWrite<u32>),
        (0x01C0 => pub cache_start2: ReadWrite<u32>),
        (0x01C4 => pub cache_end2: ReadWrite<u32>),
        (0x01C8 => pub cache_start3: ReadWrite<u32>),
        (0x01CC => pub cache_end3: ReadWrite<u32>),
        (0x01D0 => @END),
    }
}

register_bitfields![u32,
pub Rst [
    BootMcuRst OFFSET(0) NUMBITS(1) [
        Assert = 0x1,
        Deassert = 0x0,
    ],
],
pub strap_lc_ocp_lock [
    StrapKeyReleaseKeySize OFFSET(0) NUMBITS(16) [],
    StrapLcSecVolatileUnlockEn OFFSET(16) NUMBITS(1) [
        Disabled = 0,
        Enabled = 1,
    ],
    StrapOcpLockEn OFFSET(17) NUMBITS(1) [
        Disabled = 0,
        Enabled = 1,
    ],
],
pub CacheCtrl [
    CacheEn0 OFFSET(0) NUMBITS(1) [
        Disabled = 0x0,
        Enabled = 0x1,
    ],
    CacheEn1 OFFSET(1) NUMBITS(1) [
        Disabled = 0x0,
        Enabled = 0x1,
    ],
    CacheEn2 OFFSET(2) NUMBITS(1) [
        Disabled = 0x0,
        Enabled = 0x1,
    ],
    CacheEn3 OFFSET(3) NUMBITS(1) [
        Disabled = 0x0,
        Enabled = 0x1,
    ],
],
pub GlobalCacheCtrl [
    CacheEn OFFSET(0) NUMBITS(1) [
        Disabled = 0x0,
        Enabled = 0x1,
    ],
    CacheInv OFFSET(2) NUMBITS(1) [
        NoEffect = 0x0,
        Invalidate = 0x1,
    ],
    CacheInvAddr OFFSET(15) NUMBITS(13) [],
],
pub LsuMbusMap [
    MsbLsuMbusMap0 OFFSET(0) NUMBITS(8) [],
    MsbLsuMbusMap1 OFFSET(8) NUMBITS(8) [],
    MsbLsuMbusMap2 OFFSET(16) NUMBITS(8) [],
    MsbLsuMbusMap3 OFFSET(24) NUMBITS(8) [],
],

pub AhbMsbOffset [
    MsbOffset0 OFFSET(0) NUMBITS(8) [],
],

pub MbusMsbOffset [
    MsbOffset0 OFFSET(0) NUMBITS(8) [],
    MsbOffset1 OFFSET(8) NUMBITS(8) [],
    MsbOffset2 OFFSET(16) NUMBITS(8) [],
    MsbOffset3 OFFSET(24) NUMBITS(8) [],
],

];
