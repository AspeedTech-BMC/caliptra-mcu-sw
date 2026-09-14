// Licensed under the Apache-2.0 license

use crate::peripherals::Ast2705Peripherals;
use kernel::platform::chip::InterruptService;

pub const IPC0_IRQ: u8 = 0x21;
pub const IPC1_IRQ: u8 = 0x22;

#[cfg(any(feature = "doe", feature = "test-doe-transport-loopback"))]
pub const DOE_MBOX_EVENT_IRQ: u8 = 0x19;

/// Interrupt lines used by these peripherals; every other PIC line stays
/// masked. Must stay in sync with `service_interrupt()` below.
pub const USED_IRQS: &[u32] = &[IPC0_IRQ as u32, IPC1_IRQ as u32];

impl<'a> InterruptService for Ast2705Peripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        if interrupt == IPC0_IRQ as u32 || interrupt == IPC1_IRQ as u32 {
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
