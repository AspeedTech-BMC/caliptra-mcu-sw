// Licensed under the Apache-2.0 license

use crate::io::Ast1040Uart16550;
use caliptra_mcu_config_ast1040::peri;
use caliptra_mcu_tock_veer::timers::InternalTimers;
use capsules_core::virtualizers::virtual_alarm::MuxAlarm;
use kernel::utilities::StaticRef;
use registers_peripherals::aspeed_spi::AspeedSpiRegisters;
use registers_peripherals::uart16550::Uart16550Registers;

// Peripheral base address definition
pub const ASPEED_UART_BASE: StaticRef<Uart16550Registers> =
    unsafe { StaticRef::new(peri::ASPEED_UART_ADDR as *const Uart16550Registers) };
const PRIMARY_FLASH_CTRL_BASE: StaticRef<AspeedSpiRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_FMC_CTRL_ADDR as *const AspeedSpiRegisters) };
const SECONDARY_FLASH_CTRL_BASE: StaticRef<AspeedSpiRegisters> =
    unsafe { StaticRef::new(peri::ASPEED_FMC_CTRL_ADDR as *const AspeedSpiRegisters) };

pub struct Ast1040Peripherals<'a> {
    pub uart: Ast1040Uart16550<'a>,
    pub primary_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl<'a>,
    pub secondary_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl<'a>,
    pub ipc: caliptra_mcu_aspeed_ipc::AspeedIpcCtrl,
    pub dma: caliptra_mcu_dma_driver::axicdma::AxiCDMA<'a, InternalTimers<'a>>,
    #[cfg(any(feature = "doe", feature = "test-doe-transport-loopback"))]
    pub doe_transport: caliptra_mcu_doe_mbox_driver::EmulatedDoeTransport<'a, InternalTimers<'a>>,
}

impl<'a> Ast1040Peripherals<'a> {
    pub fn new(alarm: &'a MuxAlarm<'a, InternalTimers<'a>>) -> Self {
        Self {
            uart: Ast1040Uart16550::new(ASPEED_UART_BASE),
            primary_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl::new(
                PRIMARY_FLASH_CTRL_BASE,
            ),
            secondary_flash_ctrl: caliptra_mcu_flash_ctrl_ast1040::Ast1040FlashCtrl::new(
                SECONDARY_FLASH_CTRL_BASE,
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

    pub fn init(&'static self) {
        self.ipc.init();
        self.primary_flash_ctrl.init();
        self.secondary_flash_ctrl.init();
        #[cfg(any(feature = "doe", feature = "test-doe-transport-loopback"))]
        self.doe_transport.init();
    }
}
