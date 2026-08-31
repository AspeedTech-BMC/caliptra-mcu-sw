// Licensed under the Apache-2.0 license

use tock_registers::registers::{ReadOnly, ReadWrite};
use tock_registers::{self, register_bitfields, register_structs};

/// SCU0 base address as seen by the BootMCU (MCU: 0x32xx, CA35: 0x12xx)
pub const SCU0_BASE: u32 = 0x32c0_2000;
/// AST2700 silicon revision A2
pub const AST2700_REV_A2: u32 = 0x2;

register_structs! {
    pub SCU0Registers {
        /// Chip ID / HW silicon revision register
        (0x0000 => pub chip_id1: ReadOnly<u32, ChipId1::Register>),
        (0x0004 => _reserved004),
        /// HW strap override register (write 1 to force a strap bit high)
        (0x0010 => pub hwstrap1: ReadWrite<u32, HwStrap1::Register>),
        /// HW strap override clear register (write 1 to force a strap bit low)
        (0x0014 => pub hwstrap1_clr: ReadWrite<u32, HwStrap1::Register>),
        (0x0018 => _reserved018),
        /// CA35 reset-release register (write 1 to release)
        (0x010C => pub ca35_release: ReadWrite<u32>),
        /// CA35 reset vector base address register 0
        (0x0110 => pub psp_rvbar0: ReadWrite<u32>),
        /// CA35 reset vector base address register 1
        (0x0114 => pub psp_rvbar1: ReadWrite<u32>),
        /// CA35 reset vector base address register 2
        (0x0118 => pub psp_rvbar2: ReadWrite<u32>),
        /// CA35 reset vector base address register 3
        (0x011C => pub psp_rvbar3: ReadWrite<u32>),
        (0x0120 => _reserved120),
        /// Module reset 1 control: write 1 to assert reset for a module
        (0x0200 => pub modrst1_ctrl: ReadWrite<u32, ModRst1::Register>),
        /// Module reset 1 clear: write 1 to deassert (release) reset for a module
        (0x0204 => pub modrst1_clr: ReadWrite<u32, ModRst1::Register>),
        (0x0208 => _reserved208),
        /// Module reset 2 control: write 1 to assert reset for a module
        (0x0220 => pub modrst2_ctrl: ReadWrite<u32, ModRst2::Register>),
        /// Module reset 2 clear: write 1 to deassert (release) reset for a module
        (0x0224 => pub modrst2_clr: ReadWrite<u32, ModRst2::Register>),
        (0x0228 => _reserved228),
        /// Clock-gate 1 clear: write 1 to enable clock (clear clock-stop bit)
        (0x0244 => pub clock_stop_clr: ReadWrite<u32, ClkGate1::Register>),
        (0x0248 => _reserved248),
        /// USB function control register — PortA/PortB EHCI/vHub/xHCI mux
        (0x0410 => pub usb_func_ctrl: ReadWrite<u32, UsbFuncCtrl::Register>),
        /// VGA function control register — bits [9:8] select active VGA port
        (0x0414 => pub vga_func_ctrl: ReadWrite<u32, VgaFuncCtrl::Register>),
        (0x0418 => _reserved418),
        /// CPU SMP entry point lower 32 bits
        (0x0780 => pub cpu_smp_ep_lo: ReadWrite<u32>),
        /// CPU SMP entry point upper 32 bits
        (0x0784 => pub cpu_smp_ep_hi: ReadWrite<u32>),
        (0x0788 => _reserved788),
        /// SLI SLIM US delay window for lanes 0/1 (cpu_scratch[28], offset 0x7F0)
        /// bits [7:0]=lane-0 first, [15:8]=lane-0 last, [23:16]=lane-1 first, [31:24]=lane-1 last
        (0x07f0 => pub sli_slim_us_win_low: ReadWrite<u32>),
        /// SLI SLIM US delay window for lanes 2/3 (cpu_scratch[29], offset 0x7F4)
        /// bits [7:0]=lane-2 first, [15:8]=lane-2 last, [23:16]=lane-3 first, [31:24]=lane-3 last
        (0x07f4 => pub sli_slim_us_win_high: ReadWrite<u32>),
        /// SLI SLIV DS delay window (cpu_scratch[30], offset 0x7F8)
        /// bits [23:16]=first, [31:24]=last
        (0x07f8 => pub sli_sliv_ds_win: ReadWrite<u32>),
        /// SLI calibration status (cpu_scratch[31], offset 0x7FC)
        (0x07fc => pub sli_status: ReadWrite<u32, Scu0SliStatus::Register>),
        (0x0800 => _reserved800),
        /// VGA port 0 scratch register 0 (0x900) — bitfields defined in VgaScratch
        (0x0900 => pub vga0_scratch1: ReadWrite<u32, VgaScratch::Register>),
        /// VGA port 0 scratch registers 1..3 (0x904–0x90C)
        (0x0904 => pub vga0_scratch1_ext: [ReadWrite<u32>; 3]),
        /// VGA port 1 scratch register 0 (0x910) — bitfields defined in VgaScratch
        (0x0910 => pub vga1_scratch1: ReadWrite<u32, VgaScratch::Register>),
        /// VGA port 1 scratch registers 1..3 (0x914–0x91C)
        (0x0914 => pub vga1_scratch1_ext: [ReadWrite<u32>; 3]),
        (0x0920 => _reserved920),
        /// RAW (retry-after-wait) suppression configuration for CPU-die E2M
        (0x0960 => pub raw_config: ReadWrite<u32, RawConfig::Register>),
        (0x0964 => _reserved964),
        /// PCIe 0 miscellaneous registers — 32 entries (pci0_misc[0..31])
        (0x0a00 => pub pci0_misc: [ReadWrite<u32>; 32]),
        /// PCIe 1 miscellaneous registers — 32 entries (pci1_misc[0..31])
        (0x0a80 => pub pci1_misc: [ReadWrite<u32>; 32]),
        (0x0b00 => @END),
    }
}

register_bitfields![u32,
/// chip_id1 register
pub ChipId1 [
    /// Hardware silicon revision, bits [23:16]; 0=A0, 1=A1, 2=A2
    HwRev OFFSET(16) NUMBITS(8) [],
    /// Efuse-selected chip variant, bits [15:8]: 0=AST2750 (full), 1=AST2700
    /// (single VGA), 2=AST2720 (no VGA)
    Efuse OFFSET(8) NUMBITS(8) [],
],
/// HW strap register (shared layout for hwstrap1 and hwstrap1_clr)
pub HwStrap1 [
    /// VGA CRAA[1:0] VRAM size select, lower bit of GENMASK(11,10):
    /// 10b = 32MB, 11b = 64MB (the upper bit is always forced high)
    VramSizeLo OFFSET(10) NUMBITS(1) [],
    /// VGA CRAA[1:0] VRAM size select, upper bit (always forced high)
    VramSizeHi OFFSET(11) NUMBITS(1) [],
    /// DAC source select strap (SCU0_HWSTRAP_DAC_SRC)
    DacSrc OFFSET(28) NUMBITS(1) [],
    /// DisplayPort source select strap (SCU0_HWSTRAP_DP_SRC)
    DpSrc OFFSET(29) NUMBITS(1) [],
],
/// VGA function control register
pub VgaFuncCtrl [
    /// DAC source select [11:10]: 0 = VGA0, 1 = VGA1, 2 = GFX
    DacSrc OFFSET(10) NUMBITS(2) [
        Vga0 = 0,
        Vga1 = 1,
        Gfx  = 2,
    ],
    /// DisplayPort source select [9:8]: 0 = VGA0, 1 = VGA1, 2 = GFX
    DpSrc OFFSET(8) NUMBITS(2) [
        Vga0 = 0,
        Vga1 = 1,
        Gfx  = 2,
    ],
    /// Disable the DAC output entirely (SCU0_VGA_FUNC_DAC_DISABLE)
    DacDisable OFFSET(7) NUMBITS(1) [],
],
/// VGA scratch register 0 — bits [15:0]
pub VgaScratch [
    /// iKVM wide screen resolution support (obsolete)
    IkvmWideScreen   OFFSET(0)  NUMBITS(1) [],
    /// BMC firmware protection: forbid SOCFlash from updating flash
    BmcFwProtection  OFFSET(1)  NUMBITS(1) [],
    /// V-PLL reference clock: 0 = 24 MHz, 1 = 25 MHz
    VPllRefClk       OFFSET(2)  NUMBITS(1) [],
    /// SOC firmware DRAM init status: 0 = not ready, 1 = ready
    DramInitStatus   OFFSET(6)  NUMBITS(1) [],
    /// DRAM initial selection: 0 = VBIOS, 1 = SOC firmware
    DramInitSel      OFFSET(7)  NUMBITS(1) [],
    /// Disable 1920x1200 support (obsolete)
    Dis1920x1200     OFFSET(8)  NUMBITS(1) [],
    /// TX type [11:9]: selects the active DP transmitter
    TxType           OFFSET(9)  NUMBITS(3) [
        NoTx         = 0,
        Ite66121     = 1,
        Si164        = 2,
        Ch7003       = 3,
        Dp501        = 4,
        Anx9807      = 5,
        TxFwEmbedded = 6,
        DpmcuDp      = 7,
    ],
    /// Disable P2A bridge
    DisP2aBridge     OFFSET(12) NUMBITS(1) [],
    /// DP MCU firmware is executing
    DpMcuFwExecuting OFFSET(13) NUMBITS(1) [],
    /// Monitor connected
    MonitorConnected OFFSET(14) NUMBITS(1) [],
    /// Support monitor connect status in SOC scratch
    MonitorConnectStatus OFFSET(15) NUMBITS(1) [],
],
/// Module reset 1 register (shared layout for modrst1_ctrl and modrst1_clr)
pub ModRst1 [
    /// Reset bit for DisplayPort TX controller
    Dp    OFFSET(28) NUMBITS(1) [],
    /// Reset bit for DisplayPort MCU
    DpMcu OFFSET(29) NUMBITS(1) [],
],
/// Module reset 2 register (shared layout for modrst2_ctrl and modrst2_clr)
pub ModRst2 [
    /// Reset bit for USB PortA vHub1 (SCU0_RST2_USBA_VHUB1)
    UsbaVhub1 OFFSET(0) NUMBITS(1) [],
    /// Reset bit for USB PortA PHY3 (SCU0_RST2_USBA_PHY3)
    UsbaPhy3  OFFSET(1) NUMBITS(1) [],
    /// Reset bit for USB PortA xHCI (SCU0_RST2_USBA_XHCI)
    UsbaXhci  OFFSET(2) NUMBITS(1) [],
    /// Reset bit for USB PortB vHub1 (SCU0_RST2_USBB_VHUB1)
    UsbbVhub1 OFFSET(3) NUMBITS(1) [],
    /// Reset bit for USB PortB PHY3 (SCU0_RST2_USBB_PHY3)
    UsbbPhy3  OFFSET(4) NUMBITS(1) [],
    /// Reset bit for USB PortB xHCI (SCU0_RST2_USBB_XHCI)
    UsbbXhci  OFFSET(5) NUMBITS(1) [],
    /// Reset bit for USB PortA EHCI/vHub0 (SCU0_RST2_USBA_EHCI_VHUB0)
    UsbaEhciVhub0 OFFSET(6) NUMBITS(1) [],
    /// Reset bit for USB PortB EHCI/vHub0 (SCU0_RST2_USBB_EHCI_VHUB0)
    UsbbEhciVhub0 OFFSET(7) NUMBITS(1) [],
    /// Reset bit for CPU-die PCIe E2M0 (PLDA1)
    E2M0  OFFSET(10) NUMBITS(1) [],
    /// Reset bit for CPU-die PCIe E2M1 (PLDA2)
    E2M1  OFFSET(11) NUMBITS(1) [],
    /// Reset bit for IO-die PCIe E2M (SCU1_RSTCTL2_E2M, bit 24 of SCU0 modrst2)
    IoE2M OFFSET(24) NUMBITS(1) [],
    /// Reset bit for VGA (SCU0_RST2_VGA)
    Vga OFFSET(12) NUMBITS(1) [],
],
/// Clock-gate 1 register (for clock_stop_clr at offset 0x244)
pub ClkGate1 [
    /// Clock-gate for VGA port 0 (SCU0_CLKGATE1_VGA0)
    Vga0 OFFSET(5) NUMBITS(1) [],
    /// Clock-gate for USB PortB PHY3/xHCI/vHub/EHCI (SCU0_CLKGATE1_USBB)
    UsbB OFFSET(7) NUMBITS(1) [],
    /// Clock-gate for VGA port 1 (SCU0_CLKGATE1_VGA1)
    Vga1 OFFSET(10) NUMBITS(1) [],
    /// Clock-gate for CPU-die PCIe E2M0
    E2M0 OFFSET(12) NUMBITS(1) [],
    /// Clock-gate for USB PortA PHY3/xHCI/vHub/EHCI (SCU0_CLKGATE1_USBA)
    UsbA OFFSET(14) NUMBITS(1) [],
    /// Clock-gate for the shared VGA DAC (SCU0_CLKGATE1_DAC)
    Dac  OFFSET(17) NUMBITS(1) [],
    /// Clock-gate for DisplayPort controller
    Dp   OFFSET(18) NUMBITS(1) [],
    /// Clock-gate for CPU-die PCIe E2M1
    E2M1 OFFSET(19) NUMBITS(1) [],
],
/// USB function control register — selects EHCI/vHub/xHCI routing per port
/// (mirrors `usb_func_ctrl` / `enum scu0_usb_function` in the Zephyr reference).
pub UsbFuncCtrl [
    /// PortA xHCI-to-PHY3 mux (2 = xHCI routed to PHY3)
    U3XhciA    OFFSET(0)  NUMBITS(2) [],
    /// PortA xHCI/vHub1-to-PHY2 mux (0 = xHCI, 1 = vHub1, 2 = xHCI (PCIe))
    U2XhciA    OFFSET(2)  NUMBITS(2) [],
    /// PortB xHCI-to-PHY3 mux (2 = xHCI routed to PHY3)
    U3XhciB    OFFSET(4)  NUMBITS(2) [],
    /// PortB xHCI/vHub1-to-PHY2 mux (0 = xHCI, 1 = vHub1, 2 = xHCI (PCIe))
    U2XhciB    OFFSET(6)  NUMBITS(2) [],
    /// PortA EHCI/vHub0 mux (0 = EHCI to vHub, 3 = PCIe EHCI to PHY)
    VhubEhciA  OFFSET(24) NUMBITS(2) [],
    /// PortB EHCI/vHub0 mux (0 = EHCI to vHub, 3 = PCIe EHCI to PHY)
    VhubEhciB  OFFSET(28) NUMBITS(2) [],
],
/// SLI calibration status in cpu_scratch[31]
pub Scu0SliStatus [
    /// IO-die SLI1 is ready (set by BootMCU after sli_init_r completes)
    Sli1Ready OFFSET(0) NUMBITS(1) [],
],
/// raw_config register
pub RawConfig [
    /// Disable CPU-die E2M0 RAW under A2 silicon (DISCPUE2M0RAW)
    DisCpuE2M0Raw OFFSET(2)  NUMBITS(1) [],
    /// Disable CPU-die E2M1 RAW under A2 silicon (DISCPUE2M1RAW)
    DisCpuE2M1Raw OFFSET(13) NUMBITS(1) [],
],
];
