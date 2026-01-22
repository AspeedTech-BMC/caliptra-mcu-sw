// Licensed under the Apache-2.0 license

use crate::peripherals::Ast2705Peripherals;
use kernel::platform::chip::InterruptService;

pub const UART_IRQ: u8 = 0x95;
pub const MAIN_FLASH_CTRL_ERROR_IRQ: u8 = 0x13;
pub const MAIN_FLASH_CTRL_EVENT_IRQ: u8 = 0x14;
pub const RECOVERY_FLASH_CTRL_EVENT_IRQ: u8 = 0x15;
pub const RECOVERY_FLASH_CTRL_ERROR_IRQ: u8 = 0x16;
pub const DMA_EVENT_IRQ: u8 = 0x17;
pub const DMA_ERROR_IRQ: u8 = 0x18;
pub const IPC0_IRQ: u8 = 0x21;
pub const IPC1_IRQ: u8 = 0x22;

#[cfg(any(feature = "doe", feature = "test-doe-transport-loopback"))]
pub const DOE_MBOX_EVENT_IRQ: u8 = 0x19;

impl<'a> InterruptService for Ast2705Peripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        if interrupt == UART_IRQ as u32 {
            self.uart.handle_interrupt();
            return true;
        } else if interrupt == IPC0_IRQ as u32 || interrupt == IPC1_IRQ as u32 {
            return true;
        } else if interrupt == MAIN_FLASH_CTRL_ERROR_IRQ as u32
            || interrupt == MAIN_FLASH_CTRL_EVENT_IRQ as u32
        {
            self.primary_flash_ctrl.handle_interrupt();
            return true;
        } else if interrupt == RECOVERY_FLASH_CTRL_ERROR_IRQ as u32
            || interrupt == RECOVERY_FLASH_CTRL_EVENT_IRQ as u32
        {
            self.secondary_flash_ctrl.handle_interrupt();
            return true;
        } else if interrupt == DMA_ERROR_IRQ as u32 || interrupt == DMA_EVENT_IRQ as u32 {
            self.dma.handle_interrupt();
            return true;
        }
        #[cfg(any(feature = "doe", feature = "test-doe-transport-loopback"))]
        if interrupt == DOE_MBOX_EVENT_IRQ as u32 {
            self.doe_transport.handle_interrupt();
            return true;
        }
        false
    }
}
