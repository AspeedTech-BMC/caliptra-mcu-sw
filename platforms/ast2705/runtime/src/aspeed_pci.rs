// Licensed under the Apache-2.0 license.
//
// File: aspeed_pci.rs
//
// PCIe initialization for AST2705 (AST2700 family). Ported from
// zephyr-mcu-runtime/aspeed-zephyr-project/apps/mcu-runtime/soc/ast2700/pci.c

use registers_peripherals::ast2705::aspeed_scu0::{
    ChipId1, ClkGate1, ModRst2, RawConfig as Scu0RawConfig, SCU0Registers, SCU0_BASE,
};
use registers_peripherals::ast2705::aspeed_scu1::{
    RawConfig as Scu1RawConfig, SCU1Registers, SCU1_BASE,
};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

// PLDA (RC bridge) base addresses — MCU address space (0x32xx/0x34xx, not CA35 0x12xx/0x14xx)
const ASPEED_PLDA1_BASE: u32 = 0x32c1_5000; // CPU-die node 0
const ASPEED_PLDA2_BASE: u32 = 0x32c1_5800; // CPU-die node 1
const ASPEED_PLDA3_BASE: u32 = 0x34c1_c000; // IO-die node

// PLDA register offsets (relative to each PLDA node base address)
const PLDA_MSI_CAP: u32 = 0x010; // MSI capability / interrupt routing register
const PLDA_ECRC_CAP: u32 = 0x048; // ECRC generation and checking enable
const PLDA_PRESET0: u32 = 0x0b0; // TX EQ preset coefficients for lane pair 0
const PLDA_PRESET1: u32 = 0x0b4; // TX EQ preset coefficients for lane pair 1
const PLDA_ECRC_DS: u32 = 0x1fc; // ECRC downstream discard policy
const PLDA_ECRC_TX: u32 = 0x258; // ECRC conditional TX-insert policy

// PLDA ECRC bits
const ECRC_GEN_SUPPORT: u32 = 1 << 2;
const ECRC_CHK_SUPPORT: u32 = 1 << 1;
const ECRC_DISCARD_IF_DS_UNSUPP: u32 = 1 << 29;
const ECRC_TX_INSERT_IF_TD: u32 = 1 << 10;

// ---------------------------------------------------------------------------
// Public configuration types
// ---------------------------------------------------------------------------

/// Capability configuration for a single PCIe root port.
///
/// Replaces the Zephyr DT-derived `PCIE_CONF` / `PCIE_INTx` macros.
#[derive(Clone, Copy)]
pub struct PciePortConfig {
    /// Which functions are present (vga / bmc_device / ehci / xhci)
    pub has_vga: bool,
    pub has_bmc_device: bool,
    pub has_ehci: bool,
    pub has_xhci: bool,
    /// true = MSI, false = INTx (per function)
    pub vga_msi: bool,
    pub bmc_device_msi: bool,
    pub ehci_msi: bool,
    pub xhci_msi: bool,
}

impl PciePortConfig {
    /// Returns a 4-bit mask of which EP functions are enabled (VGA=0, BMC=1, EHCI=2, XHCI=3).
    /// Written into SCU0 `pciN_misc[28]` bits [7:0], [15:8], [23:16] via `* 0x010101`.
    fn en_bits(self) -> u32 {
        (self.has_vga as u32)
            | ((self.has_bmc_device as u32) << 1)
            | ((self.has_ehci as u32) << 2)
            | ((self.has_xhci as u32) << 3)
    }

    /// Returns a 4-bit mask selecting MSI (1) vs INTx (0) per EP function.
    /// Written into SCU0 `pciN_misc[28]` bits [27:24].
    fn intx_bits(self) -> u32 {
        (self.vga_msi as u32)
            | ((self.bmc_device_msi as u32) << 1)
            | ((self.ehci_msi as u32) << 2)
            | ((self.xhci_msi as u32) << 3)
    }
}

/// Top-level PCIe init configuration, replacing `struct ast_chip` fields.
pub struct AspeedPciConfig {
    /// Root port 0 (pcie0) capability config
    pub pcie0: PciePortConfig,
    /// Root port 1 (pcie1) capability config
    pub pcie1: PciePortConfig,
    /// true when an alternate PCIe node is used for pcie1 (disables vga/bmc-dev)
    pub pcie1_alt: bool,
}

impl AspeedPciConfig {
    /// Read the HW silicon revision from SCU0 `chip_id1[23:16]`.
    ///
    /// Reserved for future use — different silicon revisions may require
    /// different PCIe initialisation sequences.
    pub fn detect(scu0_base: u32) -> u32 {
        // SAFETY: caller guarantees scu0_base is a valid SCU0 MMIO base address.
        let scu0: &SCU0Registers = unsafe { &*(scu0_base as *const SCU0Registers) };
        scu0.chip_id1.read(ChipId1::HwRev)
    }
}

// ---------------------------------------------------------------------------
// Private MMIO helpers (used only for PLDA, which has no register struct)
// ---------------------------------------------------------------------------

#[inline(always)]
unsafe fn rd(addr: u32) -> u32 {
    core::ptr::read_volatile(addr as *const u32)
}

#[inline(always)]
unsafe fn wr(addr: u32, val: u32) {
    core::ptr::write_volatile(addr as *mut u32, val);
}

#[inline(always)]
unsafe fn setbits(addr: u32, bits: u32) {
    wr(addr, rd(addr) | bits);
}

#[inline(always)]
unsafe fn clrbits(addr: u32, bits: u32) {
    wr(addr, rd(addr) & !bits);
}

#[inline(always)]
unsafe fn clrsetbits(addr: u32, clr: u32, set: u32) {
    wr(addr, (rd(addr) & !clr) | set);
}

fn delay_ms(ms: u32) {
    let start = caliptra_mcu_romtime::mcycle();
    let ticks = crate::TIMER_FREQUENCY_HZ as u64 / 1000 * ms as u64;
    while caliptra_mcu_romtime::mcycle().wrapping_sub(start) < ticks {}
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Initialize one CPU-die PLDA node: TX EQ presets, bridge MSI/INTA, ECRC,
/// then enable the clock and release reset via typed SCU0 register access.
///
/// `set_clk_gate` writes the per-port bit of `clock_stop_clr` (un-gates clock).
/// `set_rst_clr` writes the per-port bit of `modrst2_clr` (deasserts reset).
unsafe fn pcie_init_node<FCG, FMR>(
    scu0: &SCU0Registers,
    node_base: u32,
    set_clk_gate: FCG,
    set_rst_clr: FMR,
) where
    FCG: FnOnce(&SCU0Registers),
    FMR: FnOnce(&SCU0Registers),
{
    // TX EQ preset P6 (coefficients Cpre=0, Cmain=18, Cpost=6) for both lane pairs.
    wr(node_base + PLDA_PRESET0, 0x12600000);
    wr(node_base + PLDA_PRESET1, 0x00012600);

    // Enable bridge MSI, set INTA routing
    clrbits(node_base + PLDA_MSI_CAP, 1 << 3);
    clrsetbits(node_base + PLDA_MSI_CAP, 0x7, 0x1);

    // Enable ECRC generation, checking, and TD-conditional insertion
    setbits(
        node_base + PLDA_ECRC_CAP,
        ECRC_GEN_SUPPORT | ECRC_CHK_SUPPORT,
    );
    setbits(node_base + PLDA_ECRC_DS, ECRC_DISCARD_IF_DS_UNSUPP);
    setbits(node_base + PLDA_ECRC_TX, ECRC_TX_INSERT_IF_TD);

    // Enable clock then release reset via typed SCU0 registers
    set_clk_gate(scu0);
    delay_ms(10);
    set_rst_clr(scu0);
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Initialize AST2705 PCIe: configure EP capabilities, set up CPU-die PLDA
/// nodes (PLDA1/PLDA2 via E2M0/E2M1), the IO-die PLDA node (PLDA3 via IoE2M),
/// then disable E2M RAW mode on both dies.
///
/// Returns 0. Skips initialisation silently if E2M resets are already clear
/// (i.e. a prior boot stage already ran PCIe init).
pub fn pci_init(cfg: &AspeedPciConfig) -> i32 {
    // SAFETY: SCU0_BASE and SCU1_BASE are valid MMIO base addresses.
    let scu0: &SCU0Registers = unsafe { &*(SCU0_BASE as *const SCU0Registers) };
    let scu1: &SCU1Registers = unsafe { &*(SCU1_BASE as *const SCU1Registers) };

    let pcie0_en = cfg.pcie0.en_bits();
    let pcie1_en = cfg.pcie1.en_bits();
    let pcie0_intx = cfg.pcie0.intx_bits();
    let pcie1_intx = cfg.pcie1.intx_bits();

    // If both E2M reset bits are already clear, PCIe was already initialised
    // (e.g. by a previous boot stage).
    if !scu0.modrst2_ctrl.is_set(ModRst2::E2M0) && !scu0.modrst2_ctrl.is_set(ModRst2::E2M1) {
        return 0;
    }

    // Program PCIe capability / interrupt-type registers in SCU0.
    // `en * 0x010101` replicates the 4-bit enable mask into bytes [23:16], [15:8], [7:0]
    // of pciN_misc[28]; the MSI/INTx select mask goes into bits [27:24].
    scu0.pci0_misc[28].set(pcie0_en * 0x010101 | (pcie0_intx << 24));
    scu0.pci1_misc[28].set(pcie1_en * 0x010101 | (pcie1_intx << 24));
    if cfg.pcie1_alt {
        // Disable vga & bmc-dev when an alternate pcie1 node is selected
        scu0.pci1_misc[28].set(scu0.pci1_misc[28].get() & !0x0303_0303);
        scu0.pci1_misc[30].set(scu0.pci1_misc[30].get() & !(1 << 31));
    }

    // CPU-die node 1 (PLDA2, E2M1) then node 0 (PLDA1, E2M0)
    unsafe {
        pcie_init_node(
            scu0,
            ASPEED_PLDA2_BASE,
            |s| s.clock_stop_clr.modify(ClkGate1::E2M1::SET),
            |s| s.modrst2_clr.modify(ModRst2::E2M1::SET),
        );
        pcie_init_node(
            scu0,
            ASPEED_PLDA1_BASE,
            |s| s.clock_stop_clr.modify(ClkGate1::E2M0::SET),
            |s| s.modrst2_clr.modify(ModRst2::E2M0::SET),
        );

        // IO-die PLDA3: only MSI/INTA setup here — no TX EQ presets or ECRC
        // registers (IO-die node does not expose those fields).  Clock and
        // reset are handled separately below via SCU0 modrst2.
        clrbits(ASPEED_PLDA3_BASE + PLDA_MSI_CAP, 1 << 3);
        clrsetbits(ASPEED_PLDA3_BASE + PLDA_MSI_CAP, 0x7, 0x1);
    }
    scu0.modrst2_ctrl.modify(ModRst2::IoE2M::SET); // assert IO-die E2M reset
    delay_ms(10);
    scu0.modrst2_clr.modify(ModRst2::IoE2M::SET); // deassert IO-die E2M reset
    delay_ms(10);

    // Disable E2M RAW on CPU-die and IO-die
    scu0.raw_config
        .modify(Scu0RawConfig::DisCpuE2M0Raw::SET + Scu0RawConfig::DisCpuE2M1Raw::SET);
    scu1.raw_config.modify(Scu1RawConfig::DisIoE2MRaw::SET);

    0
}
