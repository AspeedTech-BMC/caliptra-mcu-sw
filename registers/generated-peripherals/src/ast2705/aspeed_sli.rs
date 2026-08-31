// Licensed under the Apache-2.0 license

use tock_registers::registers::ReadWrite;
use tock_registers::{self, register_bitfields, register_structs};

// SLI0 = CPU-die SLI controller (BootMCU view; CA35 view: 0x12c1_7000)
pub const SLI0_BASE: u32 = 0x32c1_7000;
// SLI1 = IO-die SLI controller  (BootMCU view; CA35 view: 0x14c1_e000)
pub const SLI1_BASE: u32 = 0x34c1_e000;

/// SLIM (M-bus) sub-block offset within an SLI controller
pub const SLIM_OFFSET: u32 = 0x000;
/// SLIH (AHB) sub-block offset within an SLI controller
pub const SLIH_OFFSET: u32 = 0x200;
/// SLIV (video) sub-block offset within an SLI controller
pub const SLIV_OFFSET: u32 = 0x400;

// AHB bus controller bases (BootMCU view; CA35 view: AHBC0=0x1200_0000, AHBC1=0x140b_0000)
pub const AHBC0_BASE: u32 = 0x3200_0000;
pub const AHBC1_BASE: u32 = 0x340b_0000;

// IO-die interrupt controller (BootMCU view; CA35 view: 0x14c1_8000)
pub const IO_INTC_BASE: u32 = 0x34c1_8000;

register_structs! {
    /// SLI sub-block registers common to SLIH.
    ///
    /// CtrlIII field naming follows SLIH convention:
    ///   PadDly3/PadDly2 = TX-lane delays for SLIH.
    ///   PadDly1/PadDly0 = RX-lane 1/0 delays.
    pub SliHRegisters {
        /// SLI control register I
        (0x000 => pub ctrl1: ReadWrite<u32, CtrlI::Register>),
        /// SLI control register II
        (0x004 => pub ctrl2: ReadWrite<u32, CtrlII::Register>),
        /// SLI control register III — clock selection and pad delays
        (0x008 => pub ctrl3: ReadWrite<u32, CtrlIIISlih::Register>),
        /// SLI control register IV — PHY clock ring enable and delay stage
        (0x00c => pub ctrl4: ReadWrite<u32, CtrlIVSlih::Register>),
        /// Interrupt enable register
        (0x010 => pub intr_en: ReadWrite<u32, Intr::Register>),
        /// Interrupt status register (write-1-to-clear)
        (0x014 => pub intr_status: ReadWrite<u32, Intr::Register>),
        (0x018 => _reserved018),
        (0x01c => pub pad_ctrl: ReadWrite<u32, PadCtrl::Register>),
        (0x020 => @END),
    }
}
register_structs! {
    /// SLI sub-block registers common to SLIV.
    ///
    /// CtrlIII field naming follows SLIV convention:
    ///   PadDly3/PadDly2 = TX-lane delays for SLIV.
    ///   PadDly1/PadDly0 = RX-lane 1/0 delays.
    pub SliVRegisters {
        /// SLI control register I
        (0x000 => pub ctrl1: ReadWrite<u32, CtrlI::Register>),
        /// SLI control register II
        (0x004 => pub ctrl2: ReadWrite<u32, CtrlII::Register>),
        /// SLI control register III — clock selection and pad delays
        (0x008 => pub ctrl3: ReadWrite<u32, CtrlIIISliv::Register>),
        /// SLI control register IV — reserved
        (0x00c => _reserved0c),
        /// Interrupt enable register
        (0x010 => pub intr_en: ReadWrite<u32, Intr::Register>),
        /// Interrupt status register (write-1-to-clear)
        (0x014 => pub intr_status: ReadWrite<u32, Intr::Register>),
        (0x018 => @END),
    }
}

register_structs! {
    /// SLI M-bus (SLIM) sub-block registers — extends SliBlockRegisters with
    /// MARB function and MAC hotfix registers present only in the SLIM block.
    ///
    /// CtrlIII bits [23:0] are RX-lane delays for SLIM:
    ///   PadDly3/PadDly2 = RX lanes 3/2; PadDly1/PadDly0 = RX lanes 1/0.
    pub SliMRegisters {
        /// SLI control register I
        (0x000 => pub ctrl1: ReadWrite<u32, CtrlI::Register>),
        /// SLI control register II
        (0x004 => pub ctrl2: ReadWrite<u32, CtrlII::Register>),
        /// SLI control register III — RX pad delays (lanes 3-0)
        (0x008 => pub ctrl3: ReadWrite<u32, CtrlIIISlim::Register>),
        /// SLI control register IV — TX pad delays (lanes 3-0)
        (0x00c => pub ctrl4: ReadWrite<u32, CtrlIVSlim::Register>),
        /// Interrupt enable register
        (0x010 => pub intr_en: ReadWrite<u32, Intr::Register>),
        /// Interrupt status register (write-1-to-clear)
        (0x014 => pub intr_status: ReadWrite<u32, Intr::Register>),
        (0x018 => _reserved018),
        /// MARB function register
        (0x060 => pub marb_func_i: ReadWrite<u32, MarbFunc::Register>),
        (0x064 => _reserved064),
        /// MAC hotfix write target (raw value from marb_status[11:9])
        (0x068 => pub marb_func_ii: ReadWrite<u32>),
        (0x06c => _reserved06c),
        /// marb status register
        (0x0b8 => pub marb_status: ReadWrite<u32>),
        (0x0bc => @END),
    }
}

// AHB bus controller — only the per-channel timeout registers are exposed.
// Each of the 8 channels occupies 0x40 bytes; the timeout register sits at
// channel_base + 0x034 (i.e. 0x034, 0x074, 0x0b4, …, 0x1f4).
register_structs! {
    pub AhbcRegisters {
        (0x000 => _reserved000),
        (0x034 => pub ch0_timeout: ReadWrite<u32>),
        (0x038 => _reserved038),
        (0x074 => pub ch1_timeout: ReadWrite<u32>),
        (0x078 => _reserved078),
        (0x0b4 => pub ch2_timeout: ReadWrite<u32>),
        (0x0b8 => _reserved0b8),
        (0x0f4 => pub ch3_timeout: ReadWrite<u32>),
        (0x0f8 => _reserved0f8),
        (0x134 => pub ch4_timeout: ReadWrite<u32>),
        (0x138 => _reserved138),
        (0x174 => pub ch5_timeout: ReadWrite<u32>),
        (0x178 => _reserved178),
        (0x1b4 => pub ch6_timeout: ReadWrite<u32>),
        (0x1b8 => _reserved1b8),
        (0x1f4 => pub ch7_timeout: ReadWrite<u32>),
        (0x1f8 => @END),
    }
}

register_structs! {
    pub IoIntcRegisters {
        /// Interrupt controller protection / source-enable register
        (0x00 => pub prot: ReadWrite<u32, IntcProt::Register>),
        (0x04 => _reserved04),
        /// Interrupt status register (write-1-to-clear)
        (0x14 => pub status: ReadWrite<u32>),
        (0x18 => @END),
    }
}

register_bitfields![u32,
/// SLI control register I — common across SLIM / SLIH / SLIV
pub CtrlI [
    /// All channels in suspend state
    AllInSuspend     OFFSET(28) NUMBITS(1) [],
    /// Disable auto-clear when changing data pad delay
    AutoClrOffDat    OFFSET(23) NUMBITS(1) [],
    /// Disable auto-clear when changing clock pad delay
    AutoClrOffClk    OFFSET(22) NUMBITS(1) [],
    /// SP down period [21:20]
    SpDownPeriod     OFFSET(20) NUMBITS(2) [],
    /// No reset when changing TX clock
    NoRstTxClkChg    OFFSET(17) NUMBITS(1) [],
    /// SLIV raw mode (SLIV block only)
    SlivRawMode      OFFSET(15) NUMBITS(1) [],
    /// TX mode select (SLIV block only)
    TxMode           OFFSET(14) NUMBITS(1) [],
    /// RX PHY LAH select: reverse
    RxPhyLahSelRev   OFFSET(13) NUMBITS(1) [],
    /// RX PHY LAH select: negative edge
    RxPhyLahSelNeg   OFFSET(12) NUMBITS(1) [],
    /// Disable auto-send training packet
    AutoSendTrnOff   OFFSET(8)  NUMBITS(1) [],
    /// Clear bus state machine
    ClearBus         OFFSET(6)  NUMBITS(1) [],
    /// Enable SLI transceiver
    TransEn          OFFSET(5)  NUMBITS(1) [],
    /// Clear RX state machine
    ClearRx          OFFSET(2)  NUMBITS(1) [],
    /// Clear TX state machine
    ClearTx          OFFSET(1)  NUMBITS(1) [],
    /// Trigger SLI reset
    ResetTrigger     OFFSET(0)  NUMBITS(1) [],
],
/// SLI control register II
pub CtrlII [
    /// SLIV TX enter-suspend count [15:14]
    SlivTxEntSuspend OFFSET(14) NUMBITS(2) [],
],
/// SLI control register III — clock selection and pad delays (/ SLIM)
pub CtrlIIISlim [
    /// Pad delay lane 3 / TX-lane 1 [23:18]
    PadDly3   OFFSET(18) NUMBITS(6) [],
    /// Pad delay lane 2 / TX-lane 0 [17:12]
    PadDly2   OFFSET(12) NUMBITS(6) [],
    /// Pad delay RX lane 1 [11:6]
    PadDly1   OFFSET(6)  NUMBITS(6) [],
    /// Pad delay RX lane 0 [5:0]
    PadDly0   OFFSET(0)  NUMBITS(6) [],
],
/// SLI control register III — clock selection and pad delays (SLIH).
pub CtrlIIISlih [
    /// Engine clock select [31:28]
    ClkSel    OFFSET(28) NUMBITS(4) [
        Clk25M  = 0,
        Clk800M = 1,
        Clk400M = 2,
        Clk200M = 3,
        Clk500M = 6,
        Clk250M = 7,
    ],
    /// TX clock select [27:24]
    TxClkSel  OFFSET(24) NUMBITS(4) [
        TxClk25M  = 0,
        TxClk800M = 1,
        TxClk400M = 2,
        TxClk200M = 3,
        TxClk1G   = 5,
        TxClk500M = 6,
        TxClk250M = 7,
    ],
    /// SLIH TX pad delay lane 1 [22:18]
    PadDlyTx1 OFFSET(18) NUMBITS(6) [],
    /// SLIH TX pad delay lane 0 [16:12]
    PadDlyTx0 OFFSET(12) NUMBITS(6) [],
    /// SLIH RX pad delay lane 1 [10:6]
    PadDlyRx1 OFFSET(6)  NUMBITS(6) [],
    /// SLIH RX pad delay lane 0 [4:0]
    PadDlyRx0 OFFSET(0)  NUMBITS(6) [],
],
/// SLI control register III — clock selection and pad delays (SLIV).
pub CtrlIIISliv [
    /// SLIV TX pad delay lane 1 [22:18]
    PadDlyTx1 OFFSET(18) NUMBITS(6) [],
    /// SLIV TX pad delay lane 0 [16:12]
    PadDlyTx0 OFFSET(12) NUMBITS(6) [],
    /// SLIV RX pad delay lane 1 [10:6]
    PadDlyRx1 OFFSET(6)  NUMBITS(6) [],
    /// SLIV RX pad delay lane 0 [4:0]
    PadDlyRx0 OFFSET(0)  NUMBITS(6) [],
],
/// SLI control register IV — SLIM TX pad delays
pub CtrlIVSlim [
    /// SLIM TX pad delay lane 3 [23:18]
    PadDlyTx3 OFFSET(18) NUMBITS(6) [],
    /// SLIM TX pad delay lane 2 [17:12]
    PadDlyTx2 OFFSET(12) NUMBITS(6) [],
    /// SLIM TX pad delay lane 1 [11:6]
    PadDlyTx1 OFFSET(6)  NUMBITS(6) [],
    /// SLIM TX pad delay lane 0 [5:0]
    PadDlyTx0 OFFSET(0)  NUMBITS(6) [],
],
/// SLI control register IV — SLIH/SLIV PHY clock ring enable and delay stage
pub CtrlIVSlih [
    /// RX PHY clock ring enable [25]
    RxClkRingEn OFFSET(25) NUMBITS(1) [],
    /// TX PHY clock ring enable [24]
    TxClkRingEn OFFSET(24) NUMBITS(1) [],
    /// TX PHY clock inverse (bit 11 of TX delay stage)
    TxClkInv    OFFSET(11) NUMBITS(1) [],
    /// TX PHY clock delay programming [10:6]
    TxClkDly    OFFSET(6)  NUMBITS(5) [],
    /// RX PHY clock inverse (bit 5 of RX delay stage)
    RxClkInv    OFFSET(5)  NUMBITS(1) [],
    /// RX PHY clock delay programming [4:0]
    RxClkDly    OFFSET(0)  NUMBITS(5) [],
],
/// Interrupt enable / status register (shared layout for intr_en and intr_status)
pub Intr [
    /// RX sync detected
    RxSync     OFFSET(15) NUMBITS(1) [],
    /// RX error
    RxErr      OFFSET(13) NUMBITS(1) [],
    /// RX NACK received
    RxNack     OFFSET(12) NUMBITS(1) [],
    /// RX training packet received
    RxTrainPkt OFFSET(10) NUMBITS(1) [],
    /// RX disconnected
    RxDisconn  OFFSET(6)  NUMBITS(1) [],
    /// TX suspend state entered
    TxSuspend  OFFSET(4)  NUMBITS(1) [],
    /// TX training state entered
    TxTrain    OFFSET(3)  NUMBITS(1) [],
    /// TX idle state entered
    TxIdle     OFFSET(2)  NUMBITS(1) [],
    /// RX suspend state entered
    RxSuspend  OFFSET(1)  NUMBITS(1) [],
    /// RX idle state entered
    RxIdle     OFFSET(0)  NUMBITS(1) [],
],
/// SLIM MARB function register (SLIM block only, offset 0x60)
pub MarbFunc [
    /// Clear MARB arbitration state
    MarbClr OFFSET(4) NUMBITS(1) [],
    /// Enable round-robin MARB arbitration
    MarbRr  OFFSET(0) NUMBITS(1) [],
],
/// IO interrupt controller protection register
pub IntcProt [
    /// Enable CPU-die reset as interrupt source
    EnableCpudResetSrc OFFSET(11) NUMBITS(1) [],
],
pub PadCtrl [
    tx_drv OFFSET(27) NUMBITS(3) [],
]
];
