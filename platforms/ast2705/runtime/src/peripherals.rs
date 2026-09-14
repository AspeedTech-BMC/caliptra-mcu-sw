// Licensed under the Apache-2.0 license

use crate::aspeed_pci::{pci_init, AspeedPciConfig, PciePortConfig};
use crate::io::Ast2705Uart16550;
use caliptra_mcu_aspeed_soc::AhbApertureGate;
use caliptra_mcu_config_ast2705::peri;
use caliptra_mcu_flash_ctrl_ast2705::{AspeedSpiController, AspeedSpiCsConfig};
use caliptra_mcu_tock_veer::timers::InternalTimers;
use caliptra_mcu_usb_ast2705::{AspeedUsbConfig, UsbFunction};
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
use registers_peripherals::ast2705::aspeed_usb::{
    BEhciRegisters, UsbPhy2Registers, UsbPhy3Registers, USB0_EHCI_BASE, USB0_PHY2_BASE,
    USB0_PHY3_BASE, USB1_EHCI_BASE, USB1_PHY2_BASE, USB1_PHY3_BASE,
};
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
const SPI0_CTRL_BASE: StaticRef<AspeedSpiRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_SPI0_CTRL_ADDR as *const AspeedSpiRegisters) };
const SPI1_CTRL_BASE: StaticRef<AspeedSpiRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_SPI1_CTRL_ADDR as *const AspeedSpiRegisters) };
const SPI2_CTRL_BASE: StaticRef<AspeedSpiRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_SPI2_CTRL_ADDR as *const AspeedSpiRegisters) };
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
pub const ASPEED_USB0_PHY3_BASE: StaticRef<UsbPhy3Registers> =
    unsafe { StaticRef::new(USB0_PHY3_BASE as *const UsbPhy3Registers) };
pub const ASPEED_USB0_PHY2_BASE: StaticRef<UsbPhy2Registers> =
    unsafe { StaticRef::new(USB0_PHY2_BASE as *const UsbPhy2Registers) };
pub const ASPEED_USB0_EHCI_BASE: StaticRef<BEhciRegisters> =
    unsafe { StaticRef::new(USB0_EHCI_BASE as *const BEhciRegisters) };
pub const ASPEED_USB1_PHY3_BASE: StaticRef<UsbPhy3Registers> =
    unsafe { StaticRef::new(USB1_PHY3_BASE as *const UsbPhy3Registers) };
pub const ASPEED_USB1_PHY2_BASE: StaticRef<UsbPhy2Registers> =
    unsafe { StaticRef::new(USB1_PHY2_BASE as *const UsbPhy2Registers) };
pub const ASPEED_USB1_EHCI_BASE: StaticRef<BEhciRegisters> =
    unsafe { StaticRef::new(USB1_EHCI_BASE as *const BEhciRegisters) };

// erase_page() scratch space, one sector per instance.
static mut PRIMARY_ERASE_SCRATCH: [u8; caliptra_mcu_flash_ctrl_ast2705::ERASE_SECTOR_SIZE] =
    [0; caliptra_mcu_flash_ctrl_ast2705::ERASE_SECTOR_SIZE];
static mut SECONDARY_ERASE_SCRATCH: [u8; caliptra_mcu_flash_ctrl_ast2705::ERASE_SECTOR_SIZE] =
    [0; caliptra_mcu_flash_ctrl_ast2705::ERASE_SECTOR_SIZE];
static mut SPI0_ERASE_SCRATCH: [u8; caliptra_mcu_flash_ctrl_ast2705::ERASE_SECTOR_SIZE] =
    [0; caliptra_mcu_flash_ctrl_ast2705::ERASE_SECTOR_SIZE];
static mut SPI1_ERASE_SCRATCH: [u8; caliptra_mcu_flash_ctrl_ast2705::ERASE_SECTOR_SIZE] =
    [0; caliptra_mcu_flash_ctrl_ast2705::ERASE_SECTOR_SIZE];
static mut SPI2_CS0_ERASE_SCRATCH: [u8; caliptra_mcu_flash_ctrl_ast2705::ERASE_SECTOR_SIZE] =
    [0; caliptra_mcu_flash_ctrl_ast2705::ERASE_SECTOR_SIZE];
static mut SPI2_CS1_ERASE_SCRATCH: [u8; caliptra_mcu_flash_ctrl_ast2705::ERASE_SECTOR_SIZE] =
    [0; caliptra_mcu_flash_ctrl_ast2705::ERASE_SECTOR_SIZE];

pub struct Ast2705Peripherals<'a> {
    pub soc: caliptra_mcu_aspeed_soc::AspeedSoCCtrl,
    pub scu0: caliptra_mcu_scu_ast2705::AspeedScuCtrl,
    pub scu1: caliptra_mcu_scu_ast2705::AspeedScu1Ctrl,
    pub sli: caliptra_mcu_sli_ast2705::AspeedSliCtrl,
    pub uart: Ast2705Uart16550<'a>,
    pub dramc: caliptra_mcu_dramc_ast2705::DramCtrl,
    pub dp: caliptra_mcu_dp_ast2705::AspeedDpCtrl,
    pub vga: caliptra_mcu_vga_ast2705::AspeedVgaCtrl,
    pub usb: caliptra_mcu_usb_ast2705::AspeedUsbCtrl,
    pub ipc: caliptra_mcu_aspeed_ipc::AspeedIpcCtrl,
    pub primary_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl<'a>,
    pub secondary_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl<'a>,
    pub spi0_cs0_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl<'a>,
    pub spi1_cs0_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl<'a>,
    pub spi2_cs0_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl<'a>,
    pub spi2_cs1_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl<'a>,
    pub dma: caliptra_mcu_dma_driver::axicdma::AxiCDMA<'a, InternalTimers<'a>>,
    #[cfg(any(feature = "doe", feature = "test-doe-transport-loopback"))]
    pub doe_transport: caliptra_mcu_doe_mbox_driver::EmulatedDoeTransport<'a, InternalTimers<'a>>,
}

impl<'a> Ast2705Peripherals<'a> {
    #[allow(static_mut_refs)]
    pub fn new(alarm: &'a MuxAlarm<'a, InternalTimers<'a>>) -> Self {
        Self {
            soc: caliptra_mcu_aspeed_soc::AspeedSoCCtrl::new(ASPEED_SOC_BASE),
            scu0: caliptra_mcu_scu_ast2705::AspeedScuCtrl::new(ASPEED_SCU0_BASE),
            scu1: caliptra_mcu_scu_ast2705::AspeedScu1Ctrl::new(ASPEED_SCU1_BASE),
            sli: caliptra_mcu_sli_ast2705::AspeedSliCtrl::new(
                ASPEED_SCU0_BASE,
                ASPEED_SCU1_BASE,
                caliptra_mcu_sli_ast2705::TxClk::Ghz1,
            ),
            uart: Ast2705Uart16550::new(ASPEED_UART_BASE),
            dramc: caliptra_mcu_dramc_ast2705::DramCtrl::new(
                ASPEED_SDRAMC_BASE,
                ASPEED_SOC_BASE,
                ASPEED_SCU0_BASE,
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
            usb: caliptra_mcu_usb_ast2705::AspeedUsbCtrl::new(
                ASPEED_SCU0_BASE,
                ASPEED_SCU1_BASE,
                ASPEED_USB0_PHY3_BASE,
                ASPEED_USB0_PHY2_BASE,
                ASPEED_USB0_EHCI_BASE,
                ASPEED_USB1_PHY3_BASE,
                ASPEED_USB1_PHY2_BASE,
                ASPEED_USB1_EHCI_BASE,
                AspeedUsbConfig {
                    port_a_func: UsbFunction::EhciVhubAndXhciPhy,
                    port_b_func: UsbFunction::EhciVhubAndXhciPhy,
                    port_a_large_burst: false,
                    port_b_large_burst: false,
                },
            ),
            ipc: caliptra_mcu_aspeed_ipc::AspeedIpcCtrl::new(
                peri::ASPEED_IPC_TX_ADDR,
                peri::ASPEED_IPC_RX_ADDR,
                0,
                peri::ASPEED_IPC_SHM_ADDR as u64,
                peri::ASPEED_IPC_SHM_SIZE as u64,
            ),
            // CS0: boot-critical XIP, never call .init(); secondary_flash_ctrl below aliases the same hardware.
            primary_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl::new(
                AspeedSpiController {
                    regs: PRIMARY_FLASH_CTRL_BASE,
                    mem_base: peri::ASPEED_FMC_MEM_ADDR,
                    window_gate: None,
                },
                AspeedSpiCsConfig {
                    cs: 0,
                    decode_start: 0,
                    decode_size: peri::ASPEED_FMC_FLASH_SIZE_PER_CS,
                    clk_div_bits: 0, // TODO: placeholder (HCLK/16); needs a real strap-derived value
                    use_4byte_addr: true, // decode_size exceeds the 16 MB 3-byte-address range
                    read_opcode: 0x0B, // FAST_READ
                    read_dummy_clks: 8,
                },
                unsafe { &mut PRIMARY_ERASE_SCRATCH },
            ),
            secondary_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl::new(
                AspeedSpiController {
                    regs: SECONDARY_FLASH_CTRL_BASE,
                    mem_base: peri::ASPEED_FMC_MEM_ADDR,
                    window_gate: None,
                },
                AspeedSpiCsConfig {
                    cs: 0,
                    decode_start: 0,
                    decode_size: peri::ASPEED_FMC_FLASH_SIZE_PER_CS,
                    clk_div_bits: 0, // TODO: placeholder (HCLK/16); needs a real strap-derived value
                    use_4byte_addr: true, // decode_size exceeds the 16 MB 3-byte-address range
                    read_opcode: 0x0B, // FAST_READ
                    read_dummy_clks: 8,
                },
                unsafe { &mut SECONDARY_ERASE_SCRATCH },
            ),
            spi0_cs0_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl::new(
                AspeedSpiController {
                    regs: SPI0_CTRL_BASE,
                    mem_base: peri::ASPEED_SHARED_AHBC_WINDOW_BASE,
                    window_gate: Some(AhbApertureGate::new(
                        ASPEED_SOC_BASE,
                        peri::ASPEED_SPI0_AHB_MSB_OFFSET,
                    )),
                },
                AspeedSpiCsConfig {
                    cs: 0,
                    decode_start: 0,
                    decode_size: peri::ASPEED_SPI0_FLASH_SIZE_PER_CS,
                    clk_div_bits: 0, // TODO: placeholder (HCLK/16); needs a real strap-derived value
                    use_4byte_addr: false, // decode_size (16 MB) fits the 3-byte-address range
                    read_opcode: 0x0B, // FAST_READ; adjust per the actual chip on this CS
                    read_dummy_clks: 8,
                },
                unsafe { &mut SPI0_ERASE_SCRATCH },
            ),
            spi1_cs0_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl::new(
                AspeedSpiController {
                    regs: SPI1_CTRL_BASE,
                    mem_base: peri::ASPEED_SHARED_AHBC_WINDOW_BASE,
                    window_gate: Some(AhbApertureGate::new(
                        ASPEED_SOC_BASE,
                        peri::ASPEED_SPI1_AHB_MSB_OFFSET,
                    )),
                },
                AspeedSpiCsConfig {
                    cs: 0,
                    decode_start: 0,
                    decode_size: peri::ASPEED_SPI1_FLASH_SIZE_PER_CS,
                    clk_div_bits: 0, // TODO: placeholder (HCLK/16); needs a real strap-derived value
                    use_4byte_addr: false, // decode_size (16 MB) fits the 3-byte-address range
                    read_opcode: 0x0B, // FAST_READ; adjust per the actual chip on this CS
                    read_dummy_clks: 8,
                },
                unsafe { &mut SPI1_ERASE_SCRATCH },
            ),
            spi2_cs0_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl::new(
                AspeedSpiController {
                    regs: SPI2_CTRL_BASE,
                    mem_base: peri::ASPEED_SHARED_AHBC_WINDOW_BASE,
                    window_gate: Some(AhbApertureGate::new(
                        ASPEED_SOC_BASE,
                        peri::ASPEED_SPI2_AHB_MSB_OFFSET,
                    )),
                },
                AspeedSpiCsConfig {
                    cs: 0,
                    decode_start: 0,
                    decode_size: peri::ASPEED_SPI2_FLASH_SIZE_PER_CS,
                    clk_div_bits: 0, // TODO: placeholder (HCLK/16); needs a real strap-derived value
                    use_4byte_addr: false, // decode_size (8 MB) fits the 3-byte-address range
                    read_opcode: 0x0B, // FAST_READ; adjust per the actual chip on this CS
                    read_dummy_clks: 8,
                },
                unsafe { &mut SPI2_CS0_ERASE_SCRATCH },
            ),
            // Same SPI2 controller/aperture as CS0; cs=1, 8 MB slice after CS0's.
            spi2_cs1_flash_ctrl: caliptra_mcu_flash_ctrl_ast2705::Ast2705FlashCtrl::new(
                AspeedSpiController {
                    regs: SPI2_CTRL_BASE,
                    mem_base: peri::ASPEED_SHARED_AHBC_WINDOW_BASE,
                    window_gate: Some(AhbApertureGate::new(
                        ASPEED_SOC_BASE,
                        peri::ASPEED_SPI2_AHB_MSB_OFFSET,
                    )),
                },
                AspeedSpiCsConfig {
                    cs: 1,
                    decode_start: peri::ASPEED_SPI2_FLASH_SIZE_PER_CS,
                    decode_size: peri::ASPEED_SPI2_FLASH_SIZE_PER_CS,
                    clk_div_bits: 0, // TODO: placeholder (HCLK/16); needs a real strap-derived value
                    use_4byte_addr: false, // decode_size (8 MB) fits the 3-byte-address range
                    read_opcode: 0x0B, // FAST_READ; adjust per the actual chip on this CS
                    read_dummy_clks: 8,
                },
                unsafe { &mut SPI2_CS1_ERASE_SCRATCH },
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

        // primary_flash_ctrl and secondary_flash_ctrl alias the same boot-critical CE0 hardware; neither calls .init().
        self.scu1
            .pinmux_apply(caliptra_mcu_scu_ast2705::ScuSignal::Spi0Quad);
        self.spi0_cs0_flash_ctrl.init();
        self.scu1
            .pinmux_apply(caliptra_mcu_scu_ast2705::ScuSignal::Spi1Quad);
        self.spi1_cs0_flash_ctrl.init();
        self.scu1
            .pinmux_apply(caliptra_mcu_scu_ast2705::ScuSignal::Spi2Quad);
        self.scu1
            .reset_deassert(caliptra_mcu_scu_ast2705::ScuReset::Spi2);
        self.spi2_cs0_flash_ctrl.init();
        self.spi2_cs1_flash_ctrl.init();

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
