// Licensed under the Apache-2.0 license.

// Copyright Tock Contributors 2022.
// Copyright (c) 2024 Antmicro <www.antmicro.com>

#![allow(static_mut_refs)]

#[cfg(not(feature = "release"))]
use crate::CHIP;
#[cfg(not(feature = "release"))]
use crate::PROCESSES;
#[cfg(not(feature = "release"))]
use crate::PROCESS_PRINTER;
use core::cell::Cell;
use core::fmt::Write;
use core::panic::PanicInfo;
#[cfg(not(feature = "release"))]
use core::ptr::addr_of;
use core::ptr::addr_of_mut;
#[cfg(not(feature = "release"))]
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use registers_peripherals::uart16550::{Uart16550Registers, IER, LCR, LSR, THR};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

pub(crate) static mut RUNTIME_WRITER: Writer = Writer {};

/// Panic handler.
///
/// # Safety
/// Accesses memory-mapped registers.
#[cfg(all(not(test), not(feature = "release")))]
#[no_mangle]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut *addr_of_mut!(RUNTIME_WRITER);
    debug::panic_print(
        writer,
        pi,
        &rv32i::support::nop,
        &*addr_of!(PROCESSES),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    );
    exit_ast1040(1);
}

/// Panic handler (release build).
///
/// # Safety
/// Accesses memory-mapped registers.
#[cfg(all(not(test), feature = "release"))]
#[no_mangle]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    use core::fmt::Write as _;
    let writer = &mut *addr_of_mut!(RUNTIME_WRITER);
    let _ = writer.write_str("PANIC");
    if let Some(loc) = pi.location() {
        let _ = write!(writer, " at {}:{}", loc.file(), loc.line());
    }
    let _ = write!(writer, ": {}\n", pi.message());
    exit_ast1040(1);
}

/// Exit the ast1040
pub fn exit_ast1040(_exit_code: u32) -> ! {
    loop {}
}

pub struct Writer {}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }
        let regs = crate::peripherals::ASPEED_UART_BASE;
        let last_char_idx = buf.len() - 1;

        // Save/disable/restore the interrupt-enable register once for the
        // whole write, rather than once per `uart_transmit_bytes` call.
        let prev_ier = regs.ier.extract();
        if prev_ier.is_set(IER::ETBEI) {
            regs.ier.modify_no_read(prev_ier, IER::ETBEI::CLEAR);
        }

        uart_transmit_bytes(regs, &buf[..last_char_idx]);
        if buf[last_char_idx] == b'\n' {
            uart_transmit_bytes(regs, b"\r");
        }
        uart_transmit_bytes(regs, &buf[last_char_idx..]);

        regs.ier.set(prev_ier.get());

        buf.len()
    }
}

// Synchronous, busy-wait UART transmit that only touches the raw registers.
// Used by the panic/debug `Writer`, which must not build a full
// `Ast1040Uart16550` (its `DeferredCall` field would register a new
// deferred call on every print, eventually overflowing the kernel's fixed
// deferred-call table).
fn uart_transmit_bytes(regs: StaticRef<Uart16550Registers>, bytes: &[u8]) {
    for byte in bytes.iter() {
        while !regs.lsr.is_set(LSR::TxterEmpty) {}
        regs.rbr_thr.write(THR::Data.val((*byte).into()));
    }
}

pub struct Ast1040Uart16550<'a> {
    regs: StaticRef<Uart16550Registers>,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    deferred_call: DeferredCall,
}

impl<'a> Ast1040Uart16550<'a> {
    pub fn new(regs: StaticRef<Uart16550Registers>) -> Ast1040Uart16550<'a> {
        Ast1040Uart16550 {
            regs,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            deferred_call: DeferredCall::new(),
        }
    }
}

impl hil::uart::Configure for Ast1040Uart16550<'_> {
    fn configure(&self, params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        use hil::uart::{Parity, StopBits, Width};

        // 16550 operates at a default frequency of 115200. Dividing
        // this by the target frequency gives the divisor register
        // contents.
        self.regs.lcr.modify(LCR::DivLatch::SET);
        self.regs.rbr_thr.set(1);
        self.regs.lcr.modify(LCR::DivLatch::CLEAR);

        let mut lcr = self.regs.lcr.extract();

        match params.width {
            Width::Six => lcr.modify(LCR::DataWordLen::Bits6),
            Width::Seven => lcr.modify(LCR::DataWordLen::Bits7),
            Width::Eight => lcr.modify(LCR::DataWordLen::Bits8),
        };

        match params.stop_bits {
            StopBits::One => LCR::STOP::One,
            // 1.5 stop bits for 5bit works, 2 stop bits for 6-8 bit
            // words. We only support 6-8 bit words, so this
            // configures 2 stop bits
            StopBits::Two => LCR::STOP::OneHalfTwo,
        };

        match params.parity {
            Parity::None => lcr.modify(LCR::PEN.val(0b000)),
            Parity::Odd => lcr.modify(LCR::PEN.val(0b001)),
            Parity::Even => lcr.modify(LCR::PEN.val(0b011)),
        };

        match params.hw_flow_control {
            true => lcr.modify(LCR::BreakCtrlBit::SET),
            false => lcr.modify(LCR::BreakCtrlBit::CLEAR),
        };

        self.regs.lcr.set(lcr.get());

        Ok(())
    }
}

impl<'a> hil::uart::Transmit<'a> for Ast1040Uart16550<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if tx_len > tx_data.len() {
            return Err((ErrorCode::INVAL, tx_data));
        }

        if self.tx_buffer.is_some() {
            return Err((ErrorCode::BUSY, tx_data));
        }

        // Transmit the whole buffer synchronously; the completion callback
        // is delivered via a deferred call rather than the THR-empty
        // interrupt, so this path keeps working even when the UART
        // interrupt line is masked at the PIC.
        let mut index = 0;
        while index < tx_len {
            while !self.regs.lsr.is_set(LSR::THRE) {}
            self.regs
                .rbr_thr
                .write(THR::Data.val(tx_data[index].into()));
            index += 1;
        }

        self.tx_buffer.replace(tx_data);
        self.tx_len.set(tx_len);
        self.deferred_call.set();

        Ok(())
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl<'a> hil::uart::Receive<'a> for Ast1040Uart16550<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        _rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // UART receive is not serviced: the UART interrupt line is masked
        // at the PIC, so there is no interrupt to drive completion.
        Err((ErrorCode::FAIL, rx_buffer))
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        // Currently unsupported as we'd like to avoid using deferred
        // calls. Needs to be migrated to the new UART HIL anyways.
        Err(ErrorCode::FAIL)
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        // Currently unsupported.
        Err(ErrorCode::FAIL)
    }
}

impl<'a> DeferredCallClient for Ast1040Uart16550<'a> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        if let Some(tx_buf) = self.tx_buffer.take() {
            let len = self.tx_len.get();
            self.tx_client
                .map(move |client| client.transmitted_buffer(tx_buf, len, Ok(())));
        }
    }
}
