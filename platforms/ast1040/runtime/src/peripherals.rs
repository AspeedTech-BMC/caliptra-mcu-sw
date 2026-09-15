// Licensed under the Apache-2.0 license

use crate::io::Ast1040Uart16550;
use caliptra_mcu_aspeed_soc::AhbApertureGate;
use caliptra_mcu_config_ast1040::peri;
use caliptra_mcu_flash_ctrl_ast1040::{AspeedSpiController, AspeedSpiCsConfig};
use caliptra_mcu_tock_veer::timers::InternalTimers;
use capsules_core::virtualizers::virtual_alarm::MuxAlarm;
use kernel::utilities::StaticRef;
use registers_peripherals::aspeed_hyperram::{HyperRamArbRegisters, HyperRamCsrRegisters};
use registers_peripherals::aspeed_soc::SoCRegisters;
use registers_peripherals::aspeed_spi::AspeedSpiRegisters;
use registers_peripherals::ast1040::aspeed_scu1::SCU1Registers;
use registers_peripherals::uart16550::Uart16550Registers;

// Peripheral base address definition
pub const ASPEED_UART_BASE: StaticRef<Uart16550Registers> =
    unsafe { StaticRef::new(peri::ASPEED_UART_ADDR as *const Uart16550Registers) };
pub const ASPEED_SCU_BASE: StaticRef<SCU1Registers> =
    unsafe { StaticRef::new(peri::ASPEED_SCU_ADDR as *const SCU1Registers) };
pub const ASPEED_SOC_BASE: StaticRef<SoCRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_SOC_ADDR as *const SoCRegisters) };
const ASPEED_HYPERRAM_CSR_BASE: caliptra_mcu_romtime::StaticRef<HyperRamCsrRegisters> = unsafe {
    caliptra_mcu_romtime::StaticRef::new(
        peri::ASPEED_HYPERRAM_CSR_ADDR as *const HyperRamCsrRegisters,
    )
};
const ASPEED_HYPERRAM_ARB_BASE: caliptra_mcu_romtime::StaticRef<HyperRamArbRegisters> = unsafe {
    caliptra_mcu_romtime::StaticRef::new(
        peri::ASPEED_HYPERRAM_ARB_ADDR as *const HyperRamArbRegisters,
    )
};
const ASPEED_HYPERRAM_SCU_BASE: caliptra_mcu_romtime::StaticRef<SCU1Registers> =
    unsafe { caliptra_mcu_romtime::StaticRef::new(peri::ASPEED_SCU_ADDR as *const SCU1Registers) };
const PRIMARY_FLASH_CTRL_BASE: StaticRef<AspeedSpiRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_FMC_CTRL_ADDR as *const AspeedSpiRegisters) };
const SECONDARY_FLASH_CTRL_BASE: StaticRef<AspeedSpiRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_FMC_CTRL_ADDR as *const AspeedSpiRegisters) };
const SPI0_CTRL_BASE: StaticRef<AspeedSpiRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_SPI0_CTRL_ADDR as *const AspeedSpiRegisters) };
const SPI1_CTRL_BASE: StaticRef<AspeedSpiRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_SPI1_CTRL_ADDR as *const AspeedSpiRegisters) };

// erase_page() scratch space, one sector per instance.
static mut PRIMARY_ERASE_SCRATCH: [u8; caliptra_mcu_flash_ctrl_ast1040::ERASE_SECTOR_SIZE] =
    [0; caliptra_mcu_flash_ctrl_ast1040::ERASE_SECTOR_SIZE];
static mut SECONDARY_ERASE_SCRATCH: [u8; caliptra_mcu_flash_ctrl_ast1040::ERASE_SECTOR_SIZE] =
    [0; caliptra_mcu_flash_ctrl_ast1040::ERASE_SECTOR_SIZE];
static mut SPI0_CS0_ERASE_SCRATCH: [u8; caliptra_mcu_flash_ctrl_ast1040::ERASE_SECTOR_SIZE] =
    [0; caliptra_mcu_flash_ctrl_ast1040::ERASE_SECTOR_SIZE];
static mut SPI0_CS1_ERASE_SCRATCH: [u8; caliptra_mcu_flash_ctrl_ast1040::ERASE_SECTOR_SIZE] =
    [0; caliptra_mcu_flash_ctrl_ast1040::ERASE_SECTOR_SIZE];
static mut SPI1_CS0_ERASE_SCRATCH: [u8; caliptra_mcu_flash_ctrl_ast1040::ERASE_SECTOR_SIZE] =
    [0; caliptra_mcu_flash_ctrl_ast1040::ERASE_SECTOR_SIZE];
static mut SPI1_CS1_ERASE_SCRATCH: [u8; caliptra_mcu_flash_ctrl_ast1040::ERASE_SECTOR_SIZE] =
    [0; caliptra_mcu_flash_ctrl_ast1040::ERASE_SECTOR_SIZE];

pub struct Ast1040Peripherals<'a> {
    pub uart: Ast1040Uart16550<'a>,
    pub scu: caliptra_mcu_scu_ast1040::Ast1040ScuCtrl,
    pub hyperram: caliptra_mcu_hyperram_ast1040::AspeedHyperRamCtrl,
    pub primary_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl<'a>,
    pub secondary_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl<'a>,
    pub spi0_cs0_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl<'a>,
    pub spi0_cs1_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl<'a>,
    pub spi1_cs0_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl<'a>,
    pub spi1_cs1_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl<'a>,
    pub ipc: caliptra_mcu_aspeed_ipc::AspeedIpcCtrl,
    pub dma: caliptra_mcu_dma_driver::axicdma::AxiCDMA<'a, InternalTimers<'a>>,
    #[cfg(any(feature = "doe", feature = "test-doe-transport-loopback"))]
    pub doe_transport: caliptra_mcu_doe_mbox_driver::EmulatedDoeTransport<'a, InternalTimers<'a>>,
}

impl<'a> Ast1040Peripherals<'a> {
    #[allow(static_mut_refs)]
    pub fn new(alarm: &'a MuxAlarm<'a, InternalTimers<'a>>) -> Self {
        Self {
            uart: Ast1040Uart16550::new(ASPEED_UART_BASE),
            scu: caliptra_mcu_scu_ast1040::Ast1040ScuCtrl::new(ASPEED_SCU_BASE),
            hyperram: caliptra_mcu_hyperram_ast1040::AspeedHyperRamCtrl::new(
                ASPEED_HYPERRAM_CSR_BASE,
                ASPEED_HYPERRAM_ARB_BASE,
                ASPEED_HYPERRAM_SCU_BASE,
            ),
            // CS0: boot-critical XIP, never call .init(); secondary_flash_ctrl below aliases the same hardware.
            primary_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl::new(
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
            secondary_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl::new(
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
            spi0_cs0_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl::new(
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
                unsafe { &mut SPI0_CS0_ERASE_SCRATCH },
            ),
            // Same SPI0 controller/aperture as CS0; cs=1, own 16 MB slice after CS0's.
            spi0_cs1_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl::new(
                AspeedSpiController {
                    regs: SPI0_CTRL_BASE,
                    mem_base: peri::ASPEED_SHARED_AHBC_WINDOW_BASE,
                    window_gate: Some(AhbApertureGate::new(
                        ASPEED_SOC_BASE,
                        peri::ASPEED_SPI0_AHB_MSB_OFFSET,
                    )),
                },
                AspeedSpiCsConfig {
                    cs: 1,
                    decode_start: peri::ASPEED_SPI0_FLASH_SIZE_PER_CS,
                    decode_size: peri::ASPEED_SPI0_FLASH_SIZE_PER_CS,
                    clk_div_bits: 0, // TODO: placeholder (HCLK/16); needs a real strap-derived value
                    use_4byte_addr: false, // decode_size (16 MB) fits the 3-byte-address range
                    read_opcode: 0x0B, // FAST_READ; adjust per the actual chip on this CS
                    read_dummy_clks: 8,
                },
                unsafe { &mut SPI0_CS1_ERASE_SCRATCH },
            ),
            spi1_cs0_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl::new(
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
                unsafe { &mut SPI1_CS0_ERASE_SCRATCH },
            ),
            // Same SPI1 controller/aperture as CS0; cs=1, own 16 MB slice after CS0's.
            spi1_cs1_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl::new(
                AspeedSpiController {
                    regs: SPI1_CTRL_BASE,
                    mem_base: peri::ASPEED_SHARED_AHBC_WINDOW_BASE,
                    window_gate: Some(AhbApertureGate::new(
                        ASPEED_SOC_BASE,
                        peri::ASPEED_SPI1_AHB_MSB_OFFSET,
                    )),
                },
                AspeedSpiCsConfig {
                    cs: 1,
                    decode_start: peri::ASPEED_SPI1_FLASH_SIZE_PER_CS,
                    decode_size: peri::ASPEED_SPI1_FLASH_SIZE_PER_CS,
                    clk_div_bits: 0, // TODO: placeholder (HCLK/16); needs a real strap-derived value
                    use_4byte_addr: false, // decode_size (16 MB) fits the 3-byte-address range
                    read_opcode: 0x0B, // FAST_READ; adjust per the actual chip on this CS
                    read_dummy_clks: 8,
                },
                unsafe { &mut SPI1_CS1_ERASE_SCRATCH },
            ),
            ipc: caliptra_mcu_aspeed_ipc::AspeedIpcCtrl::new(
                peri::ASPEED_IPC_TX_ADDR,
                peri::ASPEED_IPC_RX_ADDR,
                0,
                peri::ASPEED_IPC_SHM_ADDR as u64,
                peri::ASPEED_IPC_SHM_SIZE as u64,
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

    pub fn init(
        &'static self,
    ) -> Result<
        caliptra_mcu_hyperram_ast1040::HyperRamInitStatus,
        caliptra_mcu_hyperram_ast1040::HyperRamError,
    > {
        let hyperram_status = self.hyperram.init()?;

        self.ipc.init();

        // primary_flash_ctrl and secondary_flash_ctrl alias the same boot-critical FMC CE0 hardware; neither calls .init().
        self.scu
            .pinmux_apply(caliptra_mcu_scu_ast1040::ScuSignal::Spi0Quad);
        self.spi0_cs0_flash_ctrl.init();
        self.spi0_cs1_flash_ctrl.init();
        self.scu
            .pinmux_apply(caliptra_mcu_scu_ast1040::ScuSignal::Spi1Quad);
        self.spi1_cs0_flash_ctrl.init();
        self.spi1_cs1_flash_ctrl.init();

        #[cfg(any(feature = "doe", feature = "test-doe-transport-loopback"))]
        self.doe_transport.init();

        Ok(hyperram_status)
    }
}
