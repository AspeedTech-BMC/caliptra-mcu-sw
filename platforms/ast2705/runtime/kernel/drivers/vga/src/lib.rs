// Licensed under the Apache-2.0 license
//
// Rust port of vga_init() from
// zephyr-mcu-runtime/aspeed-zephyr-project/apps/mcu-runtime/soc/ast2700/vga.c

#![no_std]

use core::cell::Cell;

use caliptra_mcu_capsules_runtime::bootdev::hil::{KernelImageClient, KernelImageLoader};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use registers_peripherals::aspeed_dramc::SdramcRegisters;
use registers_peripherals::ast2705::aspeed_e2m::{E2m0Registers, E2m1Registers};
use registers_peripherals::ast2705::aspeed_scu0::{
    ChipId1, ClkGate1, HwStrap1, ModRst2, SCU0Registers, VgaFuncCtrl, VgaScratch, AST2700_REV_A2,
};
use registers_peripherals::ast2705::aspeed_vga::{
    VgaLinkRegisters, VGA_PACKER_CPU_BASE, VGA_PACKER_IO_BASE, VGA_RETIMER_CPU_BASE,
    VGA_RETIMER_IO_BASE,
};

/// Caliptra UEFI/VBIOS firmware image identifier (mirrors CPTRA_UEFI_FW_ID).
pub const VGA_FW: u32 = 0x1007;
pub const DDR4_2D_DMEM_PHY: u32 = 0x1003;
pub const DDR5_DMEM_PHY: u32 = 0x1005;

/// Reserved DRAM region backing PCIe node 0's VBIOS image (`vbios_base0` in the DT).
const VBIOS0_RESERVED_MEM_BASE: u32 = 0xb1bb_0000;
/// Reserved DRAM region backing PCIe node 1's VBIOS image (`vbios_base1` in the DT).
const VBIOS1_RESERVED_MEM_BASE: u32 = 0xb1bc_0000;
const VBIOS_RESERVED_MEM_SIZE: usize = 0x1_0000;

const ASPEED_DRAM_BASE: u32 = 0x8000_0000;
const ARM_DRAM_BASE: u32 = ASPEED_DRAM_BASE >> 1;

pub struct AspeedVgaCtrl {
    scu0: StaticRef<SCU0Registers>,
    sdramc: StaticRef<SdramcRegisters>,
    e2m0: StaticRef<E2m0Registers>,
    e2m1: StaticRef<E2m1Registers>,
    loader: OptionalCell<&'static dyn KernelImageLoader>,
    self_ref: OptionalCell<&'static dyn KernelImageClient>,
    nodes: Cell<[bool; 2]>,
    is_64vram: Cell<bool>,
}

impl AspeedVgaCtrl {
    pub const fn new(
        scu0: StaticRef<SCU0Registers>,
        sdramc: StaticRef<SdramcRegisters>,
        e2m0: StaticRef<E2m0Registers>,
        e2m1: StaticRef<E2m1Registers>,
    ) -> Self {
        Self {
            scu0,
            sdramc,
            e2m0,
            e2m1,
            loader: OptionalCell::empty(),
            self_ref: OptionalCell::empty(),
            nodes: Cell::new([false, false]),
            is_64vram: Cell::new(false),
        }
    }

    /// Per-node graphics-memory base -> E2M aperture address (`_ast_get_e2m_addr`).
    fn e2m_gm_addr(&self, node: u8) -> u32 {
        let val = (self.sdramc.gm_base.get() >> (node as u32 * 16)) & 0xffff;
        (val << 20) | 0x4000_0000
    }

    /// Programs the VGA framebuffer aperture for each enabled node
    /// (`_ast_update_e2m`).
    ///
    /// Reads `gm_base`, which DRAM training fills in; `init()` runs before
    /// training has necessarily finished, so this is deferred to
    /// `image_ready`, once training is known to be done.
    fn update_e2m_vga_ram(&self, nodes: [bool; 2], is_64vram: bool) {
        let vram_size_cfg: u32 = if is_64vram { 0xf } else { 0xe };

        if nodes[0] {
            let val = self.e2m_gm_addr(0) | vram_size_cfg;
            self.e2m0.vga_ram.set(val);
            self.scu0.pci0_misc[3].set(val);
        }
        if nodes[1] {
            let val = self.e2m_gm_addr(1) | vram_size_cfg;
            self.e2m1.vga_ram.set(val);
            self.scu0.pci1_misc[3].set(val);
        }
    }

    /// Programs the VBIOS aperture for `node` (`vbios_init`).
    ///
    /// DRAM isn't guaranteed to be trained yet when this runs (called from
    /// `init()`, right after DRAM training has merely been kicked off), so
    /// the reserved region itself can't be touched here. It's zeroed and
    /// filled once the (single, shared) VBIOS image actually arrives, in
    /// `image_ready`.
    fn init_vbios(&self, node: u8) {
        let vbios_base = if node == 0 {
            VBIOS0_RESERVED_MEM_BASE
        } else {
            VBIOS1_RESERVED_MEM_BASE
        };

        // The E2M mapping only depends on the (fixed) reserved-memory
        // address, so it can be programmed immediately.
        let mem = vbios_base & !ASPEED_DRAM_BASE;
        let e2m_value = (mem >> 4) | 0x05 | ARM_DRAM_BASE;
        if node == 0 {
            self.e2m0.vbios_ram.set(e2m_value);
            self.scu0.pci0_misc[11].set(e2m_value);
        } else {
            self.e2m1.vbios_ram.set(e2m_value);
            self.scu0.pci1_misc[11].set(e2m_value);
        }
    }

    fn setup_vga_link(&self) {
        let hwstrap1 = self.scu0.hwstrap1.get();
        // NOTE: mirrors vga.c, which narrows these single strap bits into a
        // `uint8_t` — BIT(28)/BIT(29) both lose their only set bit in the
        // truncation, so `dac_src`/`dp_src` always end up 0.
        let dac_src = (hwstrap1 & (1 << 28)) as u8 as u32;
        let dp_src = (hwstrap1 & (1 << 29)) as u8 as u32;

        // Open the vlink codec on AST2700 A2 silicon.
        let open_codec = self.scu0.chip_id1.read(ChipId1::HwRev) == AST2700_REV_A2;

        self.scu0.clock_stop_clr.modify(ClkGate1::Dac::SET);
        self.scu0.modrst2_clr.modify(ModRst2::Vga::SET);

        self.scu0.vga_func_ctrl.modify(
            VgaFuncCtrl::DacSrc.val(dac_src)
                + VgaFuncCtrl::DpSrc.val(dp_src)
                + VgaFuncCtrl::DacDisable::CLEAR,
        );

        // SAFETY: these are fixed MMIO ranges for the CPU-die and IO-die VGA
        // packer/retimer link blocks.
        let packer_cpu: &VgaLinkRegisters =
            unsafe { &*(VGA_PACKER_CPU_BASE as *const VgaLinkRegisters) };
        let retimer_cpu: &VgaLinkRegisters =
            unsafe { &*(VGA_RETIMER_CPU_BASE as *const VgaLinkRegisters) };
        let packer_io: &VgaLinkRegisters =
            unsafe { &*(VGA_PACKER_IO_BASE as *const VgaLinkRegisters) };
        let retimer_io: &VgaLinkRegisters =
            unsafe { &*(VGA_RETIMER_IO_BASE as *const VgaLinkRegisters) };

        if open_codec {
            packer_cpu.reg10.set(0x0002_0009);
            packer_cpu.reg1c.set(0x80f0_f002);
        } else {
            packer_cpu.reg10.set(0x0003_0009);
        }
        packer_cpu.reg50.set(0x1000_0000 | dac_src);
        packer_cpu.reg44.set(0x0010_0010);
        retimer_cpu.reg10.set(0x0003_0009);
        packer_io.reg10.set(0x0003_0009);
        packer_io.reg44.set(0x0001_0002);
        retimer_io.reg10.set(0x0023_0009);
        retimer_io.reg44.set(0x0010_0010);
    }

    /// Initializes the AST2705 VGA subsystem: E2M apertures, VBIOS loading,
    /// clocks/reset, and the packer/retimer VGA link. Mirrors `vga_init()`.
    pub fn init(&'static self, loader: &'static dyn KernelImageLoader) -> Result<(), ErrorCode> {
        self.loader.set(loader);
        self.self_ref.set(self);

        // Decide feature by efuse: 0 = AST2750 (full function), 1 = AST2700
        // (single VGA), 2 = AST2720 (no VGA).
        let efuse = self.scu0.chip_id1.read(ChipId1::Efuse);
        if efuse == 2 {
            caliptra_mcu_romtime::println!("[vga] AST2720 has no VGA");
            return Ok(());
        }

        // Whether each PCIe node's VGA function is present: bit0 of
        // pciN_misc[28], sampled here before init_vbios() below overwrites
        // that same register with the VBIOS E2M aperture value.
        let node0_vga = self.scu0.pci0_misc[28].get() & 0x1 != 0;
        let mut node1_vga = self.scu0.pci1_misc[28].get() & 0x1 != 0;
        if efuse == 1 {
            node1_vga = false;
        }
        let nodes = [node0_vga, node1_vga];

        let is_64vram = (self.sdramc.gfmcfg.get() & 1) != 0;

        caliptra_mcu_romtime::println!("[vga] enable 0({}) 1({})", nodes[0] as u8, nodes[1] as u8);

        self.nodes.set(nodes);
        self.is_64vram.set(is_64vram);

        // scratch for VGA CRAA[1:0]: 10b = 32MB, 11b = 64MB
        self.scu0.hwstrap1.modify(HwStrap1::VramSizeHi::SET);
        if is_64vram {
            self.scu0.hwstrap1.modify(HwStrap1::VramSizeLo::SET);
        } else {
            self.scu0.hwstrap1_clr.modify(HwStrap1::VramSizeLo::SET);
        }

        for (node, &enabled) in nodes.iter().enumerate() {
            if !enabled {
                continue;
            }
            let node = node as u8;

            if node == 0 {
                self.scu0.clock_stop_clr.modify(ClkGate1::Vga0::SET);
            } else {
                self.scu0.clock_stop_clr.modify(ClkGate1::Vga1::SET);
            }

            self.init_vbios(node);

            // scratch for VGA CRD0[12]: disable P2A, DRAM init done by SOC firmware
            if node == 0 {
                self.scu0
                    .vga0_scratch1
                    .modify(VgaScratch::DramInitSel::SET + VgaScratch::DisP2aBridge::SET);
            } else {
                self.scu0
                    .vga1_scratch1
                    .modify(VgaScratch::DramInitSel::SET + VgaScratch::DisP2aBridge::SET);
            }
        }

        let _ = loader.request_image(VGA_FW, self);

        if nodes[0] || nodes[1] {
            self.setup_vga_link();
        }

        Ok(())
    }
}

impl KernelImageClient for AspeedVgaCtrl {
    fn image_ready(&self, image_id: u32, addr: u64, size: u32) {
        if image_id != VGA_FW {
            return;
        }

        // DRAM training must have completed by now, so gm_base is valid.
        self.update_e2m_vga_ram(self.nodes.get(), self.is_64vram.get());

        // A single VGA_FW image is requested for both nodes; copy it into
        // both nodes' reserved regions. DRAM is safe to touch here: this
        // fires once its image has already been loaded, so training must
        // have completed by now.
        let len = (size as usize).min(VBIOS_RESERVED_MEM_SIZE);
        for vbios_base in [VBIOS0_RESERVED_MEM_BASE, VBIOS1_RESERVED_MEM_BASE] {
            let vbios_base = vbios_base + 0x4000_0000;
            unsafe {
                core::ptr::write_bytes(vbios_base as *mut u8, 0, VBIOS_RESERVED_MEM_SIZE);
                core::ptr::copy_nonoverlapping(addr as *const u8, vbios_base as *mut u8, len);
            }
        }

        if let Some(loader) = self.loader.get() {
            loader.notify_init_done(VGA_FW, true);
        }
    }
}
