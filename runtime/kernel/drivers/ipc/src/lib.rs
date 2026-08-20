// Licensed under the Apache-2.0 license
//
// File: aspeed_ipc.rs
//
// ASPEED IPC channel for AST2705/AST1040: doorbell (IPC FIFO trigger/status
// registers) plus the shared-memory buffer it guards. The hardware FIFO
// data word carries no meaning of its own; the actual request/response
// payload lives in a fixed shared-memory buffer at `shm_base`, sized
// `shm_size`, which both sides access in strict ping-pong (only one
// message in flight at a time, so there's no need to correlate doorbells
// to a particular call). `send()`/`try_recv()`/`call()` implement that
// discipline directly: `send()` writes the request and rings the
// doorbell; `try_recv()` makes one non-blocking check for a reply and
// reads it out if the doorbell has fired; `call()` is a convenience that
// does `send()` then polls `try_recv()` in a loop. The memory fences that
// pair with the doorbell's hardware synchronization live here: a release
// fence between writing the buffer and ringing the doorbell (in
// `send()`), and an acquire fence between observing the doorbell and
// reading the buffer (in `try_recv()`).
//
// Wire layout of the shared buffer (message types, offsets, sizes) is
// defined by `aspeed_ipc_protocol::net_boot`; this type only moves bytes.

#![no_std]

use core::sync::atomic::{fence, Ordering};
use kernel::utilities::StaticRef;
use registers_peripherals::aspeed_ipc::bits::{Ch_Ctrl, Ctrl};
use registers_peripherals::aspeed_ipc::rx_regs::IpcRxCtrl;
use registers_peripherals::aspeed_ipc::tx_regs::IpcTxCtrl;
use registers_peripherals::aspeed_ipc::ASPEED_IPC_DATA_SIZE_PER_CH;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

#[derive(Debug, Eq, PartialEq)]
pub enum AspeedIpcApiError {
    TXisBusy,
    RXNotReady,
    RequestTooLarge,
    ResponseTooLarge,
    Timeout,
}

pub struct AspeedIpcCtrl {
    tx_regs: StaticRef<IpcTxCtrl>,
    rx_regs: StaticRef<IpcRxCtrl>,
    ch: tock_registers::fields::Field<u32, Ch_Ctrl::Register>,
    /// Word index of this channel's slice within `tx_regs.data`/
    /// `rx_regs.data` (the raw IPC FIFO data-word registers). Not used by
    /// `write()`/`read()` — those go through the shared-memory buffer at
    /// `shm_base` instead; this is only for direct FIFO data-word access.
    #[allow(dead_code)]
    data_reg_offset: usize,
    /// Base address of the shared-memory buffer this channel's payload
    /// lives in.
    pub shm_base: u64,
    /// Size in bytes of the shared-memory buffer at `shm_base`.
    pub shm_size: u64,
}

impl AspeedIpcCtrl {
    pub const fn new(tx_addr: u32, rx_addr: u32, ch_id: u32, shm_base: u64, shm_size: u64) -> Self {
        let ch = match ch_id {
            0 => Ch_Ctrl::Ch0,
            1 => Ch_Ctrl::Ch1,
            2 => Ch_Ctrl::Ch2,
            3 => Ch_Ctrl::Ch3,
            _ => Ch_Ctrl::Ch0, // Default to Ch0 for invalid values
        };
        let data_reg_offset = (ch_id * ASPEED_IPC_DATA_SIZE_PER_CH / 4) as usize;
        AspeedIpcCtrl {
            tx_regs: unsafe { StaticRef::new(tx_addr as *const _) },
            rx_regs: unsafe { StaticRef::new(rx_addr as *const _) },
            ch,
            data_reg_offset,
            shm_base,
            shm_size,
        }
    }

    pub fn init(&self) {
        //caliptra_mcu_romtime::println!("[mcu-ipc] Initializing ASPEED IPC controller...");

        self.rx_regs.ctrl.modify(Ctrl::Enable::SET);
        /* Enable ch0 */
        self.rx_regs.enable.modify(self.ch.val(1));
    }

    pub fn is_rx_ready(&self) -> bool {
        self.tx_regs.enable.is_set(self.ch)
    }

    /// Rings the doorbell: triggers this channel's TX. Bounded busy-wait
    /// for the previous message to be consumed before triggering, so a
    /// missing peer returns `TXisBusy` instead of hanging.
    pub fn notify(&self) -> Result<(), AspeedIpcApiError> {
        if !self.is_rx_ready() {
            return Err(AspeedIpcApiError::RXNotReady);
        }

        if self.tx_regs.status.is_set(self.ch) {
            return Err(AspeedIpcApiError::TXisBusy);
        }

        self.tx_regs.trig.set(1 << self.ch.shift);

        Ok(())
    }

    /// Non-blocking check for an incoming doorbell. Clears the RX status
    /// bit and returns `Ok(())` if one is pending, `Err(RXNotReady)`
    /// otherwise.
    pub fn try_poll(&self) -> Result<(), AspeedIpcApiError> {
        if !self.rx_regs.status.is_set(self.ch) {
            return Err(AspeedIpcApiError::RXNotReady);
        }

        self.rx_regs.status.modify(self.ch.val(1));
        Ok(())
    }

    /// Copies `bytes` into the shared buffer at `shm_base`, starting at
    /// offset 0. Bytes beyond `bytes.len()` are left untouched (may hold a
    /// previous message's tail); that's fine because a reader only ever
    /// reads as many bytes as its expected message type requires.
    ///
    /// Uses only volatile writes: the region is written by another core,
    /// so forming a `&mut` reference over it would be undefined behavior.
    pub fn write(&self, bytes: &[u8]) -> Result<(), AspeedIpcApiError> {
        if bytes.len() as u64 > self.shm_size {
            return Err(AspeedIpcApiError::RequestTooLarge);
        }
        let base = self.shm_base as *mut u8;
        unsafe {
            for (i, b) in bytes.iter().enumerate() {
                base.add(i).write_volatile(*b);
            }
        }
        Ok(())
    }

    /// Fills `buf` with `buf.len()` bytes read from the shared buffer at
    /// `shm_base`, starting at offset 0. Uses only volatile reads, for
    /// the same reason as `write()`.
    pub fn read(&self, buf: &mut [u8]) -> Result<(), AspeedIpcApiError> {
        self.read_at(0, buf)
    }

    /// Fills `buf` with `buf.len()` bytes read from the shared buffer at
    /// `shm_base + offset`. Same volatile-read discipline as `read()`; use
    /// this to pull data out of the buffer that follows a fixed-size
    /// header already read via `read()`/`call()`, without a copy of the
    /// whole buffer.
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<(), AspeedIpcApiError> {
        if offset as u64 + buf.len() as u64 > self.shm_size {
            return Err(AspeedIpcApiError::ResponseTooLarge);
        }
        let base = self.shm_base as *const u8;
        unsafe {
            for (i, b) in buf.iter_mut().enumerate() {
                *b = base.add(offset + i).read_volatile();
            }
        }
        Ok(())
    }

    /// Reads `len` bytes from the shared buffer at `shm_base + offset` and
    /// writes them directly to `dst_addr`, without needing a caller-owned
    /// `&mut [u8]` for the destination. Used to stage image chunks straight
    /// into their final location -- callers (e.g. the `capsules-runtime`
    /// crate, which forbids `unsafe`) can't build a slice over an arbitrary
    /// address themselves, so the raw write is done here instead.
    ///
    /// Same volatile-read discipline as `read_at()` on the source side; the
    /// destination is assumed to be exclusively owned by the caller for the
    /// duration of this call (not concurrently accessed elsewhere), so
    /// plain volatile writes suffice there too -- there's no peer core
    /// racing the destination the way there is with `shm_base`.
    pub fn read_at_to_addr(
        &self,
        offset: usize,
        dst_addr: usize,
        len: usize,
    ) -> Result<(), AspeedIpcApiError> {
        if offset as u64 + len as u64 > self.shm_size {
            return Err(AspeedIpcApiError::ResponseTooLarge);
        }
        let src = self.shm_base as *const u8;
        let dst = dst_addr as *mut u8;
        unsafe {
            for i in 0..len {
                dst.add(i)
                    .write_volatile(src.add(offset + i).read_volatile());
            }
        }
        Ok(())
    }

    /// Serializes `req` into the shared buffer and rings the doorbell.
    /// Returns as soon as the doorbell has been rung — it does not wait
    /// for a reply; call `try_recv()` (or `call()`) to get one.
    ///
    /// Don't call `send()` again until a `try_recv()`/`call()` has
    /// returned the reply to this one — correctness depends on strict
    /// ping-pong discipline (see the module doc comment): nothing may
    /// call `send()` again until a reply has been read, which guarantees
    /// there is never more than one message in flight.
    pub fn send(&self, req: &[u8]) -> Result<(), AspeedIpcApiError> {
        self.write(req)?;
        // Publish the write above before the doorbell trigger that tells
        // the peer to look at it.
        fence(Ordering::Release);
        self.notify()
    }

    /// Makes one non-blocking check for a reply. Returns `Ok(true)` and
    /// fills `resp` if the doorbell has fired since the last `send()`;
    /// `Ok(false)` if it hasn't yet (call again later, or fall through to
    /// your own timeout).
    pub fn try_recv(&self, resp: &mut [u8]) -> Result<bool, AspeedIpcApiError> {
        if self.try_poll().is_err() {
            return Ok(false);
        }

        // Pairs with the peer's release fence before it rang the doorbell
        // back: the buffer write it made is now visible.
        fence(Ordering::Acquire);
        self.read(resp)?;
        Ok(true)
    }

    pub fn recv(&self, resp: &mut [u8], max_polls: usize) -> Result<(), AspeedIpcApiError> {
        for _ in 0..max_polls {
            if self.try_recv(resp)? {
                return Ok(());
            }
        }
        Err(AspeedIpcApiError::Timeout)
    }

    /// Issues one request/response exchange: `send()`s `req`, then polls
    /// `try_recv()` up to `max_polls` times for the reply, which is copied
    /// into `resp`.
    ///
    /// This call blocks (via polling) until it gets a reply or times out —
    /// that's what guarantees only one message is ever in flight (see the
    /// module doc comment). Don't issue another `call()`/`send()`
    /// concurrently with this one.
    ///
    /// `max_polls` bounds the wait as an iteration count rather than a
    /// wall-clock timeout, since ROM has no timer to measure one against.
    pub fn call(
        &self,
        req: &[u8],
        resp: &mut [u8],
        max_polls: usize,
    ) -> Result<(), AspeedIpcApiError> {
        self.send(req)?;
        self.recv(resp, max_polls)
    }
}
