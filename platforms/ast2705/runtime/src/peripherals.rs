// Licensed under the Apache-2.0 license

use crate::aspeed_pci::{pci_init, AspeedPciConfig, PciePortConfig};
use crate::io::Ast2705Uart16550;
use caliptra_mcu_config_ast2705::peri;
use caliptra_mcu_tock_veer::timers::InternalTimers;
use capsules_core::virtualizers::virtual_alarm::MuxAlarm;
use kernel::utilities::StaticRef;
use registers_peripherals::aspeed_dramc::SdramcRegisters;
use registers_peripherals::aspeed_soc::SoCRegisters;
use registers_peripherals::aspeed_spi::AspeedSpiRegisters;
use registers_peripherals::ast2705::aspeed_dpmcu::{
    DpMcuRegisters, DpRegisters, DP_BASE, MCU_REG_BASE,
};
use registers_peripherals::ast2705::aspeed_e2m::{
    E2m0Registers, E2m1Registers, E2M0_BASE, E2M1_BASE,
};
use registers_peripherals::ast2705::aspeed_scu0::SCU0Registers;
use registers_peripherals::ast2705::aspeed_scu1::SCU1Registers;
use registers_peripherals::uart16550::Uart16550Registers;

// Peripheral base address definition
pub const ASPEED_UART_BASE: StaticRef<Uart16550Registers> =
    unsafe { StaticRef::new(peri::ASPEED_UART_ADDR as *const Uart16550Registers) };
pub const ASPEED_SDRAMC_BASE: StaticRef<SdramcRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_SDRAMC_ADDR as *const SdramcRegisters) };
pub const ASPEED_SOC_BASE: StaticRef<SoCRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_SOC_ADDR as *const SoCRegisters) };
pub const ASPEED_SCU0_BASE: StaticRef<SCU0Registers> =
    unsafe { StaticRef::new(peri::ASPEED_SCU0_ADDR as *const SCU0Registers) };
const PRIMARY_FLASH_CTRL_BASE: StaticRef<AspeedSpiRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_FMC_CTRL_ADDR as *const AspeedSpiRegisters) };
const SECONDARY_FLASH_CTRL_BASE: StaticRef<AspeedSpiRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_FMC_CTRL_ADDR as *const AspeedSpiRegisters) };
pub const ASPEED_SCU1_BASE: StaticRef<SCU1Registers> =
    unsafe { StaticRef::new(peri::ASPEED_SCU1_ADDR as *const SCU1Registers) };
pub const ASPEED_DPMCU_REG_BASE: StaticRef<DpMcuRegisters> =
    unsafe { StaticRef::new(MCU_REG_BASE as *const DpMcuRegisters) };
pub const ASPEED_DP_BASE: StaticRef<DpRegisters> =
    unsafe { StaticRef::new(DP_BASE as *const DpRegisters) };
pub const ASPEED_E2M0_BASE: StaticRef<E2m0Registers> =
    unsafe { StaticRef::new(E2M0_BASE as *const E2m0Registers) };
pub const ASPEED_E2M1_BASE: StaticRef<E2m1Registers> =
    unsafe { StaticRef::new(E2M1_BASE as *const E2m1Registers) };

const BOOTMCU_DRAM_BASE: u64 = 0x8000_0000;

pub struct Ast2705Peripherals<'a> {
    pub soc: caliptra_mcu_aspeed_soc::AspeedSoCCtrl,
    pub scu0: caliptra_mcu_scu_ast2705::AspeedScuCtrl,
    pub sli: caliptra_mcu_sli_ast2705::AspeedSliCtrl,
    pub uart: Ast2705Uart16550<'a>,
    pub dramc: caliptra_mcu_dramc_ast2705::DramCtrl,
    pub dp: caliptra_mcu_dp_ast2705::AspeedDpCtrl,
    pub vga: caliptra_mcu_vga_ast2705::AspeedVgaCtrl,
    pub ipc: caliptra_mcu_aspeed_ipc::AspeedIpcCtrl,
    pub primary_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl<'a>,
    pub secondary_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl<'a>,
    pub dma: caliptra_mcu_dma_driver::axicdma::AxiCDMA<'a, InternalTimers<'a>>,
    #[cfg(any(feature = "doe", feature = "test-doe-transport-loopback"))]
    pub doe_transport: caliptra_mcu_doe_mbox_driver::EmulatedDoeTransport<'a, InternalTimers<'a>>,
}

impl<'a> Ast2705Peripherals<'a> {
    pub fn new(alarm: &'a MuxAlarm<'a, InternalTimers<'a>>) -> Self {
        Self {
            soc: caliptra_mcu_aspeed_soc::AspeedSoCCtrl::new(ASPEED_SOC_BASE),
            scu0: caliptra_mcu_scu_ast2705::AspeedScuCtrl::new(ASPEED_SCU0_BASE),
            sli: caliptra_mcu_sli_ast2705::AspeedSliCtrl::new(
                ASPEED_SCU0_BASE,
                ASPEED_SCU1_BASE,
                caliptra_mcu_sli_ast2705::TxClk::Ghz1,
            ),
            uart: Ast2705Uart16550::new(ASPEED_UART_BASE),
            dramc: caliptra_mcu_dramc_ast2705::DramCtrl::new(
                ASPEED_SDRAMC_BASE,
                peri::ASPEED_DRAM_PHY_ADDR,
            ),
            dp: caliptra_mcu_dp_ast2705::AspeedDpCtrl::new(
                ASPEED_DPMCU_REG_BASE,
                ASPEED_DP_BASE,
                ASPEED_SCU0_BASE,
            ),
            vga: caliptra_mcu_vga_ast2705::AspeedVgaCtrl::new(
                ASPEED_SCU0_BASE,
                ASPEED_SDRAMC_BASE,
                ASPEED_E2M0_BASE,
                ASPEED_E2M1_BASE,
            ),
            ipc: caliptra_mcu_aspeed_ipc::AspeedIpcCtrl::new(
                peri::ASPEED_IPC_TX_ADDR,
                peri::ASPEED_IPC_RX_ADDR,
                0,
                BOOTMCU_DRAM_BASE,
            ),
            primary_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl::new(
                PRIMARY_FLASH_CTRL_BASE,
            ),
            secondary_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl::new(
                SECONDARY_FLASH_CTRL_BASE,
            ),
            dma: caliptra_mcu_dma_driver::axicdma::AxiCDMA::new(
                caliptra_mcu_dma_driver::axicdma::DMA_CTRL_BASE,
                false,
                Some(alarm),
            ),
            #[cfg(any(feature = "doe", feature = "test-doe-transport-loopback"))]
            doe_transport: caliptra_mcu_doe_mbox_driver::EmulatedDoeTransport::new(
                caliptra_mcu_doe_mbox_driver::DOE_MBOX_BASE,
                alarm,
            ),
        }
    }

    pub fn init(&'static self) {
        self.ipc.init();
        self.soc.set_ahb_msb_offset(0x1);
        self.soc.set_lsu_mbus_map(0x0, 0x1, 0x2, 0x3);
        self.primary_flash_ctrl.init();
        self.secondary_flash_ctrl.init();
        #[cfg(any(feature = "doe", feature = "test-doe-transport-loopback"))]
        self.doe_transport.init();

        let port_cfg = PciePortConfig {
            has_vga: true,
            has_bmc_device: true,
            has_ehci: true,
            has_xhci: true,
            vga_msi: true,
            bmc_device_msi: true,
            ehci_msi: false,
            xhci_msi: false,
        };
        let pci_cfg = AspeedPciConfig {
            pcie0: port_cfg,
            pcie1: port_cfg,
            pcie1_alt: false,
        };
        pci_init(&pci_cfg);
    }
}
