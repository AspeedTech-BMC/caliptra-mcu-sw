// Licensed under the Apache-2.0 license

//! Software (no-op hardware) DMA engine, CPU-copy based.
//!
//! Implements the `caliptra_mcu_capsules_runtime::dma::hil::Dma` HAL by copying with the
//! CPU, for platforms without a usable DMA controller. Modeled on the
//! emulator's `nodma` placeholder, but completes via a deferred call
//! instead of an alarm, so it needs no timer wiring and finishes on the
//! next kernel-loop iteration. Swap in a real engine (e.g. AxiCDMA)
//! without touching any client code.
//! Boards must call `register()` on the driver during init.

use caliptra_mcu_capsules_runtime::dma::hil::{Dma, DmaClient, DmaError, DmaRoute, DmaStatus};
use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

pub struct NoDMA {
    client: OptionalCell<&'static dyn DmaClient>,
    src_addr: Cell<u64>,
    dest_addr: Cell<u64>,
    byte_count: Cell<usize>,
    busy: Cell<bool>,
    deferred_call: DeferredCall,
}

impl Default for NoDMA {
    fn default() -> Self {
        Self::new()
    }
}

impl NoDMA {
    pub fn new() -> Self {
        Self {
            client: OptionalCell::empty(),
            src_addr: Cell::new(0),
            dest_addr: Cell::new(0),
            byte_count: Cell::new(0),
            busy: Cell::new(false),
            deferred_call: DeferredCall::new(),
        }
    }
}

impl Dma for NoDMA {
    fn configure_transfer(
        &self,
        byte_count: usize,
        block_size: usize,
        src_addr: Option<u64>,
        dest_addr: Option<u64>,
    ) -> Result<(), ErrorCode> {
        if byte_count == 0 || block_size == 0 || block_size > byte_count {
            return Err(ErrorCode::INVAL);
        }
        let (src, dest) = match (src_addr, dest_addr) {
            (Some(src), Some(dest)) => (src, dest),
            _ => return Err(ErrorCode::INVAL),
        };
        if self.busy.get() {
            return Err(ErrorCode::BUSY);
        }
        self.src_addr.set(src);
        self.dest_addr.set(dest);
        self.byte_count.set(byte_count);
        Ok(())
    }

    fn start_transfer(
        &self,
        read_route: DmaRoute,
        write_route: DmaRoute,
        _fixed_addr: bool,
    ) -> Result<(), ErrorCode> {
        if read_route != DmaRoute::AxiToAxi || write_route != DmaRoute::AxiToAxi {
            // Only AxiToAxi route is supported, same as the nodma placeholder.
            return Err(ErrorCode::INVAL);
        }
        if self.busy.get() {
            return Err(ErrorCode::BUSY);
        }
        self.busy.set(true);
        self.deferred_call.set();
        Ok(())
    }

    fn poll_status(&self) -> Result<DmaStatus, DmaError> {
        // Not supported
        Err(DmaError::CommandError)
    }

    fn write_fifo(&self, _data: &[u8]) -> Result<(), DmaError> {
        Err(DmaError::CommandError)
    }

    fn read_fifo(&self, _buffer: &mut [u8]) -> Result<usize, DmaError> {
        Err(DmaError::CommandError)
    }

    fn set_client(&self, client: &'static dyn DmaClient) {
        self.client.set(client);
    }
}

impl DeferredCallClient for NoDMA {
    fn handle_deferred_call(&self) {
        if !self.busy.get() {
            return;
        }

        // Byte-wise volatile copy, same as the emulator nodma placeholder
        // (src or dest may not be word aligned).
        for offset in 0..self.byte_count.get() as u64 {
            let src_ptr = self.src_addr.get().wrapping_add(offset) as usize as *const u8;
            let dst_ptr = self.dest_addr.get().wrapping_add(offset) as usize as *mut u8;
            // SAFETY: the addresses were supplied by the DMA client under
            // the DMA syscall contract: src is a live source buffer, dest
            // is the transfer destination (plain MCU-writable memory with
            // no other referents during the transfer).
            unsafe {
                let value = core::ptr::read_volatile(src_ptr);
                core::ptr::write_volatile(dst_ptr, value);
            }
        }

        self.busy.set(false);
        self.client.map(|client| {
            client.transfer_complete(DmaStatus::TxnDone);
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
