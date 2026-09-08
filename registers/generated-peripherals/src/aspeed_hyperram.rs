// Licensed under the Apache-2.0 license
use tock_registers::registers::{ReadOnly, ReadWrite};
use tock_registers::{self, register_bitfields, register_structs};

/// Reset value of `HYPERRAM_CTRL_1`.
///
/// Encryption and decryption bypass are enabled after reset; all other
/// functions controlled by this register are disabled.
pub const HYPERRAM_CTRL_1_RESET_VALUE: u32 = 0x0000_1100;

// Hyper RAM memory-mapped controller (HYPERCSR, bootmcu "MMC_REG" block).
register_structs! {
    pub HyperRamCsrRegisters {
        (0x000 => pub mmadr_range0: ReadWrite<u32>),
        (0x004 => pub mmadr_range1: ReadWrite<u32>),
        (0x008 => pub mm_reset: ReadWrite<u32>),
        (0x00C => pub mr_ctrl: ReadWrite<u32>),
        (0x010 => pub timing_pgsz: ReadWrite<u32>),
        (0x014 => pub dpd_cfg: ReadWrite<u32>),
        (0x018 => pub hs_cfg: ReadWrite<u32>),
        (0x01C => pub inten: ReadWrite<u32>),
        (0x020 => pub xfersts: ReadOnly<u32>),
        (0x024 => pub cmdctrl0: ReadWrite<u32>),
        (0x028 => pub cmdctrl1: ReadWrite<u32>),
        (0x02C => pub dqstmo: ReadWrite<u32>),
        (0x030 => pub phyctrl: [ReadWrite<u32>; 8]),
        (0x050 => pub auto_dpd_cfg: ReadWrite<u32>),
        (0x054 => pub auto_dpd_timing: ReadWrite<u32>),
        (0x058 => @END),
    }
}
// Hyper RAM arbiter controller (HYPERARB, bootmcu "MMCARB_REG" block).
register_structs! {
    pub HyperRamArbRegisters {
        (0x000 => _reserved0),
        (0x004 => pub ctrl0: ReadWrite<u32, Ctrl0::Register>),
        (0x008 => pub ctrl1: ReadWrite<u32, Ctrl1::Register>),
        (0x00C => pub ctrl2: ReadWrite<u32>),
        (0x010 => pub aes_enckey: [ReadWrite<u32>; 4]),
        (0x020 => pub aes_enciv: [ReadWrite<u32>; 3]),
        (0x02C => pub aes_encaddr: ReadWrite<u32>),
        (0x030 => pub aes_deckey: [ReadWrite<u32>; 4]),
        (0x040 => pub aes_deciv: [ReadWrite<u32>; 3]),
        (0x04C => pub aes_decaddr: ReadWrite<u32>),
        (0x050 => pub read_dead_code: ReadWrite<u32>),
        (0x054 => pub region_en: ReadWrite<u32>),
        (0x058 => pub rstcnt_thold: ReadWrite<u32>),
        (0x05C => pub mr_ctrl: ReadWrite<u32>),
        (0x060 => pub ipconfig: ReadWrite<u32>),
        (0x064 => pub phycfg: [ReadWrite<u32>; 3]),
        /// HYPERRAM_AES_ADDR_CONFIG. Writes are blocked by
        /// `rgo1_hyper_setting_lock`.
        (0x070 => pub aesaddr: ReadWrite<u32, AesAddr::Register>),
        (0x074 => pub eccaddr: ReadWrite<u32, EccAddr::Register>),
        (0x078 => pub addrdecode: ReadWrite<u32>),
        (0x07C => pub mr_tmo_thold: ReadWrite<u32>),
        (0x080 => pub arb_cfg0: ReadWrite<u32>),
        (0x084 => pub pollute_cfg: [ReadWrite<u32>; 4]),
        (0x094 => _reserved94),
        (0x100 => pub rd_prot: [ReadWrite<u32>; 16]),
        (0x140 => _reserved140),
        (0x150 => pub wr_prot: [ReadWrite<u32>; 16]),
        (0x190 => _reserved190),
        (0x200 => pub prot_addr: [ReadWrite<u32>; 16]),
        (0x240 => pub length_msb: ReadWrite<u32>),
        (0x244 => pub status0: ReadOnly<u32, Status0::Register>),
        (0x248 => pub int_ctrl: ReadWrite<u32>),
        (0x24C => pub int_sts: ReadWrite<u32>),
        (0x250 => pub lock: ReadWrite<u32>),
        (0x254 => pub mr_lock: ReadWrite<u32>),
        (0x258 => pub arb_lock: ReadWrite<u32>),
        (0x25C => pub intlog0: ReadOnly<u32>),
        (0x260 => pub intlog1: ReadOnly<u32>),
        (0x264 => pub intlog2: ReadOnly<u32>),
        (0x268 => pub intecc: ReadOnly<u32>),
        (0x26C => pub intlog_ctrl: ReadWrite<u32>),
        (0x270 => pub reset_ctrl0: ReadWrite<u32>),
        (0x274 => pub reset_ctrl1: ReadWrite<u32>),
        (0x278 => pub dummy_ctrl: ReadWrite<u32>),
        (0x27C => _reserved27c),
        (0x280 => pub dummy_out: [ReadOnly<u32>; 4]),
        (0x290 => _reserved290),
        (0xFF0 => pub hyperram_id0: ReadOnly<u32>),
        (0xFF4 => pub hyperram_id1: ReadOnly<u32>),
        (0xFF8 => pub hyperram_config0: ReadWrite<u32>),
        (0xFFC => pub hyperram_config1: ReadWrite<u32>),
        (0x1000 => @END),
    }
}
register_bitfields![u32,
pub Ctrl0 [
    IP_RST  OFFSET(0)  NUMBITS(1) [],
    SW_RST  OFFSET(4)  NUMBITS(1) [],
    LOCK_EN OFFSET(16) NUMBITS(1) [],
],
pub Ctrl1 [
    /// HyperRAM AES encryption enable (reset: 0).
    AES_ENC_EN     OFFSET(0)  NUMBITS(1) [],
    /// HyperRAM AES decryption enable (reset: 0).
    AES_DEC_EN     OFFSET(4)  NUMBITS(1) [],
    /// HyperRAM AES encryption-function bypass (reset: 1).
    AES_ENC_BYPASS OFFSET(8)  NUMBITS(1) [],
    /// HyperRAM AES decryption-function bypass (reset: 1).
    AES_DEC_BYPASS OFFSET(12) NUMBITS(1) [],
    /// HyperRAM ECC encoder enable (reset: 0).
    ECC_ENC_EN     OFFSET(16) NUMBITS(1) [],
    /// HyperRAM ECC decoder enable (reset: 0).
    ECC_DEC_EN     OFFSET(20) NUMBITS(1) [],
    /// HyperRAM write protection enable (reset: 0).
    WP_EN          OFFSET(24) NUMBITS(1) [],
    /// HyperRAM read protection enable (reset: 0).
    RP_EN          OFFSET(28) NUMBITS(1) [],
],

pub AesAddr [
    /// AES region start address, in 4 KiB units (reset: 0).
    START OFFSET(0)  NUMBITS(12) [],
    /// AES region length, in 4 KiB units (reset: 0).
    LEN   OFFSET(16) NUMBITS(12) [],
],

pub EccAddr [
    /// Bits [11:0]: ECC region start address, in 4 KiB units.
    START OFFSET(0)  NUMBITS(12) [],
    /// Bits [27:16]: ECC region length, in 4 KiB units.
    LEN   OFFSET(16) NUMBITS(12) [],
],
pub Status0 [
    RESET_DONE OFFSET(0) NUMBITS(1) [],
],
];
