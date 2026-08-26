// Licensed under the Apache-2.0 license

use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{self, register_bitfields, register_structs};

register_structs! {
pub SdramcRegisters {
    (0x0000 => pub prot_key: ReadWrite<u32>),		// offset 0x00
    (0x0004 => pub intr_status: ReadWrite<u32, INTR::Register>), // 0x04
    (0x0008 => pub intr_clear:  WriteOnly<u32, INTR::Register>), // 0x08
    (0x000C => pub intr_mask:   ReadWrite<u32, INTR::Register>), // 0x0C

    (0x0010 => pub mcfg: ReadWrite<u32, MCFG::Register>),     // 0x10
    (0x0014 => pub mctl: ReadWrite<u32, MCTL::Register>),     // 0x14
    (0x0018 => pub msts: ReadOnly<u32>),
    (0x001C => pub error_status: ReadOnly<u32>),

    (0x0020 => pub actime1: ReadWrite<u32, ACTIME1::Register>),
    (0x0024 => pub actime2: ReadWrite<u32, ACTIME2::Register>),
    (0x0028 => pub actime3: ReadWrite<u32, ACTIME3::Register>),
    (0x002C => pub actime4: ReadWrite<u32, ACTIME4::Register>),

    (0x0030 => pub actime5: ReadWrite<u32, ACTIME5::Register>),
    (0x0034 => pub actime6: ReadWrite<u32, ACTIME6::Register>),
    (0x0038 => pub actime7: ReadWrite<u32, ACTIME7::Register>),
    (0x003C => pub dfi_timing: ReadWrite<u32, DFI_TIMING::Register>),

    (0x0040 => pub dcfg: ReadWrite<u32>),
    (0x0044 => pub dctl: ReadWrite<u32>),

    (0x0048 => pub mrctl: ReadWrite<u32, MRCTL::Register>),
    (0x004C => pub mrwr:  ReadWrite<u32>),
    (0x0050 => pub mrrd:  ReadOnly<u32>),

    (0x0054 => pub mr01: ReadWrite<u32>),
    (0x0058 => pub mr23: ReadWrite<u32>),
    (0x005C => pub mr45: ReadWrite<u32>),
    (0x0060 => pub mr67: ReadWrite<u32>),

    (0x0064 => pub refctl: ReadWrite<u32, REFCTL::Register>),
    (0x0068 => pub refmng_ctl: ReadWrite<u32>),
    (0x006C => pub refsts: ReadOnly<u32>),

    (0x0070 => pub zqctl: ReadWrite<u32, ZQCTL::Register>),  // 0x70
    (0x0074 => pub ecc_addr_range: ReadWrite<u32>),     // 0x74
    (0x0078 => pub ecc_failure_status: ReadOnly<u32>),  // 0x78
    (0x007C => pub ecc_failure_addr: ReadOnly<u32>),    // 0x7C

    (0x0080 => pub ecc_test_control: ReadWrite<u32>),   // 0x80
    (0x0084 => pub ecc_test_status:  ReadOnly<u32>),    // 0x84
    (0x0088 => pub arbctl: ReadWrite<u32, ARBCTL::Register>),  // 0x88
    (0x008C => pub enccfg: ReadWrite<u32>),	       // 0x8C

    (0x0090 => pub protect_lock_set: ReadWrite<u32>),   // ... 省略保護/加密欄位
    (0x0094 => pub reserved0: ReadWrite<u32>),
    (0x0098 => pub reserved1: ReadWrite<u32>),
    (0x009C => pub enc_min_addr: ReadWrite<u32>),
    (0x00A0 => pub enc_max_addr: ReadWrite<u32>),
    (0x00A4 => pub _reserved_a4_to_bf: [u8; 0x1C]),

    (0x00C0 => pub bistcfg: ReadWrite<u32, BISTCFG::Register>),	 // 0xC0
    (0x00C4 => pub bist_addr: ReadWrite<u32>),
    (0x00C8 => pub bist_size: ReadWrite<u32>),
    (0x00CC => pub bist_patt: ReadWrite<u32>),
    (0x00D0 => pub bist_res:  ReadOnly<u32, BISTRES::Register>),
    (0x00D4 => pub _reserved_d4_to_ff: [u8; 0x2C]),

    (0x0100 => pub gfmcfg: ReadOnly<u32>),
    (0x0104 => pub gfm0ctl: ReadWrite<u32>),
    (0x0108 => pub gfm1ctl: ReadWrite<u32>),
    /// Per-node graphics-memory base address, packed as two 16-bit halves:
    /// bits [15:0] = node 0 GM base, bits [31:16] = node 1 GM base.
    (0x010C => pub gm_base: ReadWrite<u32>),
    (0x0110 => _reserved_110_to_1ff: [u8; 0xF0]),

    // 省略: port[6], region[16] 等（若要啟 MPU/QoS 再展開）
    (0x0200 => @END),
    }
}

// 依 sdram_ast2700.h 巨集建立 bitfield
register_bitfields![u32,
pub INTR [
    PHY_INIT_DONE  OFFSET(0) NUMBITS(1) [],   // DRAMC_IRQSTA_PHY_INIT_DONE
    MR_DONE        OFFSET(1) NUMBITS(1) [],   // DRAMC_IRQSTA_MR_DONE
    BIST_DONE      OFFSET(8) NUMBITS(1) [],   // DRAMC_IRQSTA_BIST_DONE
    REF_DONE       OFFSET(10) NUMBITS(1) [],  // DRAMC_IRQSTA_REF_DONE
],

pub ACTIME1 [
    TMRD_TMOD  OFFSET(0)  NUMBITS(6) [],  // DDR4: tMOD, DDR5: tMRD  (reg = N, timing = 2N nCK)
    TRRD       OFFSET(8)  NUMBITS(6) [],  // tRRD   (reg = N, timing = 2N nCK)
    TRRD_L     OFFSET(16) NUMBITS(6) [],  // tRRD_L (reg = N, timing = 2N nCK)
    TCCD_L     OFFSET(24) NUMBITS(6) [],  // tCCD_L (reg = N, timing = N nCK)
],

pub ACTIME2 [
    TRCD       OFFSET(0)  NUMBITS(6) [],  // tRCD (reg = N, timing = 2N nCK)
    TRAS       OFFSET(8)  NUMBITS(6) [],  // tRAS (reg = N, timing = 2N nCK)
    TRP        OFFSET(16) NUMBITS(6) [],  // tRP  (reg = N, timing = 2N nCK)
    TFAW       OFFSET(24) NUMBITS(6) [],  // tFAW (reg = N, timing = 2N nCK)
],

pub ACTIME3 [
    TRTP       OFFSET(0)  NUMBITS(6) [],  // tRTP        (reg = N, timing = 2N nCK)
    TWTP       OFFSET(8)  NUMBITS(7) [],  // WL+BL/2+tWR (reg = N, timing = 2N nCK)
    TRTW       OFFSET(16) NUMBITS(6) [],  // CL-WL+BL/2+2 (reg = N, timing = 2N nCK)
    TWTR       OFFSET(24) NUMBITS(6) [],  // WL+BL/2+tWTR (reg = N, timing = 2N nCK)
],

pub ACTIME4 [
    TWTR_L     OFFSET(0)  NUMBITS(6) [],  // CWL+BL/2+tWTRL  (reg = N, timing = 2N nCK)
    TWTR_A     OFFSET(8)  NUMBITS(7) [],  // CWL+BL/2+tWTRA, DDR5 only (reg = N, timing = 2N nCK)
],

pub ACTIME5 [
    TRFC            OFFSET(0) NUMBITS(10) [],   // DRAMC_ACTIME5_TRFC
    TRFCSB          OFFSET(10) NUMBITS(10) [],   // DRAMC_ACTIME5_TRFCSB
    TREFSBRD        OFFSET(20) NUMBITS(8) [],  // DRAMC_ACTIME5_TREFSBRD
],

pub ACTIME6 [
    TCKSRE          OFFSET(0)  NUMBITS(6) [],  // tCKSRE/tCKSRX/tCKLCS (reg = N, timing = 2N nCK)
    TXP             OFFSET(8)  NUMBITS(6) [],  // tXP (exit power-down, reg = N, timing = 2N nCK)
    TPD             OFFSET(16) NUMBITS(6) [],  // tPD (power-down entry to exit, reg = N, timing = 2N nCK)
    TCSHSR          OFFSET(24) NUMBITS(6) [],  // tCSH SRexit, DDR5 only (reg = N, timing = 2N nCK)
],

pub ACTIME7 [
    TDLLK           OFFSET(0)  NUMBITS(10) [], // tDLLK/tXSDLL/tXS (DLL locking, reg = N, timing = 2N nCK)
    TZQCS           OFFSET(10) NUMBITS(10) [], // tZQCS (DDR4) / tZQLat (DDR5) (reg = N, timing = 2N nCK)
],

pub MCTL [
    PHY_INIT_START	 OFFSET(0) NUMBITS(1) [], // DRAMC_MCTL_PHY_INIT_START
    SELF_REF_START	 OFFSET(1) NUMBITS(1) [], // DRAMC_MCTL_SELF_REF_START
    PHY_POWER_ON	 OFFSET(16) NUMBITS(1) [],// DRAMC_MCTL_PHY_POWER_ON
    PHY_RESET	 OFFSET(17) NUMBITS(1) [],// DRAMC_MCTL_PHY_RESET
],

pub MRCTL [
    CMD_START      OFFSET(0) NUMBITS(1) [],   // DRAMC_MRCTL_CMD_START
    CMD_WR_CTL     OFFSET(1) NUMBITS(1) [],   // DRAMC_MRCTL_CMD_WR_CTL
    CMD_TYPE       OFFSET(2) NUMBITS(1) [],   // DRAMC_MRCTL_CMD_TYPE / MR_RW
    DQ_SEL	       OFFSET(6) NUMBITS(1) [],
    DLL_RST        OFFSET(7) NUMBITS(1) [],
    ADDR	       OFFSET(8) NUMBITS(8) [],   // MR_ADDR(n)
],

pub MCFG [
    PGM_EN	       OFFSET(5) NUMBITS(1) [],   // DRAMC_MCFG_PGM_EN
    ECC_EN	       OFFSET(6) NUMBITS(1) [],   // DRAMC_MCFG_ECC_EN
    SIZE	       OFFSET(2) NUMBITS(3) [],   // RAM size field
    DDR5           OFFSET(0) NUMBITS(1) [],   // DRAMC_MCFG_DDR5
],

pub BISTCFG [
    START	       OFFSET(0) NUMBITS(1) [],   // DRAMC_BISTCFG_START
    ENABLE	       OFFSET(1) NUMBITS(1) [],   // DRAMC_BISTCFG_ENABLE
    BMODE	       OFFSET(2) NUMBITS(2) [],   // DRAMC_BISTCFG_BMODE
    PMODE	       OFFSET(4) NUMBITS(3) [],   // DRAMC_BISTCFG_PMODE
    INIT_MODE      OFFSET(7) NUMBITS(1) [],
],

pub BISTRES [
    DONE	       OFFSET(8) NUMBITS(1) [],   // DRAMC_BISTRES_DONE
    FAIL	       OFFSET(9) NUMBITS(1) [],   // DRAMC_BISTRES_FAIL
],

pub DFI_TIMING [
    // tphy_wrlat: WR command to write data enable.  Timing = N+4 (DDR4) / N+3 (DDR5)
    TWRLAT          OFFSET(0)  NUMBITS(6) [],
    // tphy_wrdata: write data enable to write data.  Timing = N nCK
    TWRDATA         OFFSET(6)  NUMBITS(3) [],
    // tphy_rddata_en: RD command to read data enable.  Timing = N+4 (DDR4) / N+3 (DDR5)
    TRDDATA_EN      OFFSET(10) NUMBITS(6) [],
    // tphy_odtlat: WR command to ODT active.  Timing = N+4 nCK.  DDR4 only.
    TODTLAT         OFFSET(16) NUMBITS(4) [],
    // ODT high-pulse extension.  0-3 = extend 0-3 nCK.  DDR4 only.
    ODT_EXT         OFFSET(20) NUMBITS(2) [],
    // ODT signal enable.  DDR4 only.
    ODT_EN          OFFSET(22) NUMBITS(1) [],
    // ODT always-on (when ODT_EN=1).
    ODT_AON         OFFSET(23) NUMBITS(1) [],
],

pub REFCTL [
    // W1T: trigger refresh mode change
    REF_UPDATE      OFFSET(0)  NUMBITS(1) [],
    // 0=normal (1X), 1=FGR mode (2X/4X)
    FGR_MODE        OFFSET(1)  NUMBITS(1) [],
    // FGR rate select: 0=2X, 1=4X.  DDR4 only.
    FGR_RATE        OFFSET(2)  NUMBITS(1) [],
    // Same-bank refresh (REFsb).  DDR5+FGR only.
    SAME_BANK_REF   OFFSET(3)  NUMBITS(1) [],
    // Auto MRS/MRW during mode change
    AUTO_MR         OFFSET(4)  NUMBITS(1) [],
    // Refresh flexibility: allow postponing REF when busy
    FLEX_EN         OFFSET(5)  NUMBITS(1) [],
    // Max number of postponeable REF commands
    FLEX_THRESHOLD  OFFSET(6)  NUMBITS(4) [],
    // 1 = disable periodic refresh
    REF_DIS         OFFSET(15) NUMBITS(1) [],
    // tREFI in units of 24 MHz clock cycles
    TREFI           OFFSET(16) NUMBITS(8) [],
    // Refresh flexibility time period
    FLEX_PERIOD     OFFSET(24) NUMBITS(8) [],
],

pub ZQCTL [
    // DDR4: 0=ZQCS, 1=ZQCL
    ZQCL_EN         OFFSET(2)  NUMBITS(1) [],
    // DDR5: block DRAM access during tZQCAL
    ZQ_LOCK         OFFSET(3)  NUMBITS(1) [],
    // R: 0=idle, 1=ZQ calibration in progress
    ZQ_STATUS       OFFSET(7)  NUMBITS(1) [],
    // Time between ZQCal start and latch MPC commands, in units of 128 nCK
    ZQ_LAT_PERIOD   OFFSET(8)  NUMBITS(8) [],
    // Period to issue ZQ command automatically, in units of tREFI count
    ZQ_PERIOD       OFFSET(16) NUMBITS(16) [],
],

pub ARBCTL [
    // Enable write-request-count limit to force switch to read mode
    WR_LIMIT_EN     OFFSET(0)  NUMBITS(1) [],
    // Write limit count select: 0=4,1=8,2=16,3=32,4=48,5=64,6=96,7=128
    WR_LIMIT_SEL    OFFSET(1)  NUMBITS(3) [],
    // Enable read-request-count limit to force switch to write mode
    RD_LIMIT_EN     OFFSET(4)  NUMBITS(1) [],
    // Read limit count select: same encoding as WR_LIMIT_SEL
    RD_LIMIT_SEL    OFFSET(5)  NUMBITS(3) [],
    // Enable page-hit request count limit
    PAGE_HIT_LIMIT_EN  OFFSET(11) NUMBITS(1) [],
    // Page-hit limit select: 0=8,1=16,2=32,3=64
    PAGE_HIT_LIMIT_SEL OFFSET(12) NUMBITS(2) [],
    // Disable write-to-read force condition
    DIS_WR2RD_FORCE OFFSET(14) NUMBITS(1) [],
    // Disable read-to-write force condition
    DIS_RD2WR_FORCE OFFSET(15) NUMBITS(1) [],
],
];
