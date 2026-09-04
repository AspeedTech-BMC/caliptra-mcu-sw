// Licensed under the Apache-2.0 license

use tock_registers::registers::{ReadOnly, ReadWrite};
use tock_registers::{self, register_bitfields, register_structs};

/// SCU1 base address as seen by the BootMCU (MCU: 0x34xx, CA35: 0x14xx)
pub const SCU1_BASE: u32 = 0x34c0_2000;

register_structs! {
    pub SCU1Registers {
        /// Chip ID / HW silicon revision register
        (0x0000 => pub chip_id1: ReadOnly<u32, ChipId1::Register>),
        (0x0004 => _reserved004),
        /// OTP strap register
        (0x0010 => pub strap: ReadWrite<u32, OTPStrap::Register>),
        (0x0014 => _reserved14),
        /// Crystal oscillator control register
        (0x00F8 => pub xtal_ctrl: ReadWrite<u32, XtalCtrl::Register>),
        (0x00FC => _reservedfc),
        /// MCU0 control register
        (0x0110 => pub mcu0_ctrl: ReadWrite<u32, MCU0_CTRL::Register>),
        /// MCU0 reset vector register
        (0x0114 => pub rst_vector: ReadWrite<u32>),
        (0x0118 => _reserved118),
        /// RAW (retry-after-wait) suppression configuration for IO-die E2M
        (0x01EC => pub raw_config: ReadWrite<u32, RawConfig::Register>),
        /// SLI SLIM DS delay window for lanes 0/1 (scratch[28], offset 0x1F0)
        /// bits [7:0]=lane-0 first, [15:8]=lane-0 last, [23:16]=lane-1 first, [31:24]=lane-1 last
        (0x01F0 => pub sli_slim_ds_win_low: ReadWrite<u32>),
        /// SLI SLIM DS delay window for lanes 2/3 (scratch[29], offset 0x1F4)
        /// bits [7:0]=lane-2 first, [15:8]=lane-2 last, [23:16]=lane-3 first, [31:24]=lane-3 last
        (0x01F4 => pub sli_slim_ds_win_high: ReadWrite<u32>),
        /// SLI SLIH AHB + SLIV US delay windows (scratch[30], offset 0x1F8)
        /// bits [15:0] = SLIH AHB window (first=[7:0], last=[15:8])
        /// bits [31:16] = SLIV US window  (first=[23:16], last=[31:24])
        (0x01F8 => pub sli_ahb_sliv_us_win: ReadWrite<u32>),
        /// SLI calibration status (scratch[31], offset 0x1FC)
        (0x01fc => pub sli_status: ReadWrite<u32, Scu1SliStatus::Register>),
        /// Reset Control 1
        (0x0200 => pub rstctl1: ReadWrite<u32, Rst1::Register>),
        /// Reset Clear 1
        (0x0204 => pub rstctl1_clr: ReadWrite<u32, Rst1::Register>),
        (0x0208 => _reserved208),
        /// Reset Control 2
        (0x0218 => pub rstctl2: ReadWrite<u32, Rst2::Register>),
        /// Reset Clear 2
        (0x021C => pub rstctl2_clr: ReadWrite<u32, Rst2::Register>),
        (0x0220 => _reserved220),
        /// Clock Stop Clear 1
        (0x024C => pub clkstop1_clr: ReadWrite<u32, Clk1::Register>),
        (0x0250 => _reserved250),
        /// Clock Selection 1
        (0x0280 => pub clk_sel1: ReadWrite<u32, ClkSel1::Register>),
        (0x0284 => _reserved284),
        (0x03F0 => pub cptra_mci_generic_input_wire0: ReadWrite<u32, MCIInputWire0::Register>),
        (0x03F4 => pub cptra_mci_generic_input_wire1: ReadWrite<u32>),
        (0x03F8 => _reserved3f8),
        /// PinMux Group N
        (0x0434 => pub pinmux_n: ReadWrite<u32, PinMux::Register>),
        /// PinMux Group O
        (0x0438 => pub pinmux_o: ReadWrite<u32, PinMux::Register>),
        /// PinMux Group P
        (0x043C => pub pinmux_p: ReadWrite<u32, PinMux::Register>),
        /// PinMux Group Q
        (0x0440 => pub pinmux_q: ReadWrite<u32, PinMux::Register>),
        /// PinMux Group R
        (0x0444 => pub pinmux_r: ReadWrite<u32, PinMux::Register>),
        /// PinMux Group S
        (0x0448 => pub pinmux_s: ReadWrite<u32, PinMux::Register>),
        /// PinMux Group T
        (0x044C => pub pinmux_t: ReadWrite<u32, PinMux::Register>),
        /// PinMux Group U
        (0x0450 => pub pinmux_u: ReadWrite<u32, PinMux::Register>),
        (0x0454 => _reserved454),
        (0x0800 => pub otpcfg: ReadWrite<u32>),
        (0x0804 => _reserved804),
        (0x0900 => pub vga0_scratch: ReadWrite<u32>),
        (0x0904 => _reserved904),
        (0x0910 => pub vga1_scratch: ReadWrite<u32>),
        (0x0914 => _reserved914),
        /// Host Configuration 1 register.
        (0x0a00 => pub host_conf1: ReadWrite<u32, HostConf::Register>),
        (0x0a04 => _reserveda04),
        /// Host Configuration 2 register.
        (0x0a20 => pub host_conf2: ReadWrite<u32, HostConf::Register>),
        (0x0a24 => @END),
    }
}

register_bitfields![u32,
/// chip_id1 register
pub ChipId1 [
    /// Hardware silicon revision, bits [23:16]; 0=A0, 1=A1, 2=A2
    HwRev OFFSET(16) NUMBITS(8) [],
],

pub OTPStrap [
    DDR_TYPE OFFSET(0) NUMBITS(3) [],
    BootSrcEn OFFSET(4) NUMBITS(1) [],
    LowSecure OFFSET(8) NUMBITS(1) [],
    /// Recovery boot interface select (SCU1_HWSTRAP1_RECOVERY_INTERFACE):
    /// 0b00 = I3C, 0b01 = Mac, 0b10 = Usb, 0b11 = Uart
    BootSrcSel OFFSET(16) NUMBITS(2) [
        I3c = 0b00,
        Mac = 0b01,
        Usb = 0b10,
        Uart = 0b11,
    ],
    RecoveryInterface OFFSET(26) NUMBITS(2) [
        Spi = 0b00,
        Usb = 0b01,
        I2c = 0b10,
        I3c = 0b11,
    ],
    /// Recovery USB port select when RecoveryInterface == Usb
    /// (SCU1_HWSTRAP1_RECOVERY_USB_PORT): 0 = PortA, 1 = PortB
    RecoveryUsbPort OFFSET(28) NUMBITS(2) [],
],

/// Crystal oscillator control register (offset 0xF8)
pub XtalCtrl [
    /// Crystal oscillator select/enable, bit [0]
    Xtal OFFSET(0) NUMBITS(1) [
        Disabled = 0,
        Enabled = 1,
    ],
],

pub MCU0_CTRL [
    SPI_SEL     OFFSET(0) NUMBITS(1) [], // SCU_IO_MCU0_CTRL_SPI_SEL
    DIS_EM_MODE OFFSET(1) NUMBITS(1) [], // SCU_IO_MCU0_CTRL_DIS_EM_MODE
    MAP0        OFFSET(9) NUMBITS(7) [], // SCU_IO_MCU0_CTRL_MCU0_MAP0
    MAP1        OFFSET(16) NUMBITS(7) [], // SCU_IO_MCU0_CTRL_MCU0_MAP1
    DISDBG      OFFSET(23) NUMBITS(1) [], // SCU_IO_MCU0_CTRL_DISDBG
    LOCK        OFFSET(24) NUMBITS(1) [], // SCU_IO_MCU0_CTRL_LOCK
    SLEEP       OFFSET(28) NUMBITS(1) [], // SCU_IO_MCU0_CTRL_SLEEP
    MAJOR_ALERT OFFSET(29) NUMBITS(2) [], // SCU_IO_MCU0_CTRL_MAJOR_ALERT
],

/// raw_config register
pub RawConfig [
    /// Disable IO-die E2M RAW under A2 silicon (DISIOE2MRAW)
    DisIoE2MRaw OFFSET(6) NUMBITS(1) [],
],

/// Reset Control/Clear 1
pub Rst1 [
    /// Reset bit for MDC/MDIO Controller
    PMI   OFFSET(2) NUMBITS(1) [],
    /// Reset bit for MAC0
    MAC0  OFFSET(5) NUMBITS(1) [],
    /// Reset bit for MAC1
    MAC1  OFFSET(6) NUMBITS(1) [],
    /// Reset bit for XGMAC
    MAC2  OFFSET(7) NUMBITS(1) [],
    /// Reset bit for SPI2
    SPI2  OFFSET(15) NUMBITS(1) [],
],

/// Reset Control/Clear 2
pub Rst2 [
    /// Reset bit for XPCS
    E16PHY  OFFSET(11) NUMBITS(1) [],
],

/// Clock Stop 1
pub Clk1 [
    /// Clock gate for MAC0
    MAC0  OFFSET(8) NUMBITS(1) [],
    /// Clock gate for MAC1
    MAC1  OFFSET(9) NUMBITS(1) [],
    /// Clock gate for XGMAC
    MAC2  OFFSET(10) NUMBITS(1) [],
],

/// Clock selection 1
pub ClkSel1 [
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

pub MCIInputWire0 [
    BurnLCTokens OFFSET(0) NUMBITS(1) [],
    TransitionManufacturing OFFSET(1) NUMBITS(1) [],
    TransitionProduction OFFSET(2) NUMBITS(1) [],
    ProgramFieldEntropy OFFSET(3) NUMBITS(1) [],
],

/// SLI calibration status in scratch[31]
pub Scu1SliStatus [
    /// CPU-die SLI0 is ready (set by IO-die MCU after sli_init_f completes)
    Sli0Ready    OFFSET(0) NUMBITS(1) [],
    /// Skip SLI calibration (e.g. warm reboot)
    SliSkipCali  OFFSET(1) NUMBITS(1) [],
],

/// Host Configuration register.
pub HostConf [
    /// LPC host interface enable. EDAF is invalid when this bit is set.
    LpcEn OFFSET(9) NUMBITS(1) [],
    /// EDAF host interface enable.
    EdafEn OFFSET(10) NUMBITS(1) [
        Disabled = 0,
        Enabled = 1,
    ],
],
];
