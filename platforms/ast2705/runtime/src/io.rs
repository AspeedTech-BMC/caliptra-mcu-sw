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
use core::ptr::{read_volatile, write_volatile};
#[cfg(not(feature = "release"))]
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use registers_peripherals::uart16550::{Uart16550Registers, IER, IIR, LCR, LSR, THR};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

pub(crate) static mut WRITER: Writer = Writer {};

/// Panic handler.
///
/// # Safety
/// Accesses memory-mapped registers.
#[cfg(all(not(test), not(feature = "release")))]
#[no_mangle]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut *addr_of_mut!(WRITER);
    debug::panic_print(
        writer,
        pi,
        &rv32i::support::nop,
        &*addr_of!(PROCESSES),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    );
    exit_ast2705(1);
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
    let writer = &mut *addr_of_mut!(WRITER);
    let _ = writer.write_str("PANIC");
    if let Some(loc) = pi.location() {
        let _ = write!(writer, " at {}:{}", loc.file(), loc.line());
    }
    let _ = write!(writer, ": {}\n", pi.message());
    exit_ast2705(1);
}

/// Exit the ast2705
pub fn exit_ast2705(exit_code: u32) -> ! {
    // Safety: This is a safe memory address to write to for exiting the ast2705.
    unsafe {
        // By writing to this address we can exit the ast2705.
        write_volatile(0x1000_2000 as *mut u32, exit_code);
    }
    unreachable!()
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
        for b in buf {
            // Print to this address for emulator output
            unsafe {
                write_volatile(0x1000_1041 as *mut u8, *b);
            }
        }
        buf.len()
    }
}

fn read_byte() -> u8 {
    unsafe { read_volatile(0x1000_1041 as *mut u8) }
}

pub struct Ast2705Uart16550<'a> {
    regs: StaticRef<Uart16550Registers>,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_index: Cell<usize>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_index: Cell<usize>,
}

impl<'a> Ast2705Uart16550<'a> {
    pub fn new(regs: StaticRef<Uart16550Registers>) -> Ast2705Uart16550<'a> {
        Ast2705Uart16550 {
            regs,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_index: Cell::new(0),
        }
    }

    pub fn handle_interrupt(&self) {
        let iir = self.regs.iir_fcr.extract();

        if !iir.matches_all(IIR::Pending::Pending) {
            return;
        }

        if iir.matches_all(IIR::Identification::ThrEmpty) {
            self.transmit_continue();
        } else if iir.matches_all(IIR::Identification::RxDataAvailable) {
            self.receive();
        } else if iir.matches_all(IIR::Identification::ModemStatusChange) {
            self.regs.msr.get();
        } else if iir.matches_all(IIR::Identification::RxLineStatus) {
            self.regs.lsr.get();
        } else if iir.matches_all(IIR::Identification::Timeout) {
            self.regs.rbr_thr.get();
        } else {
            panic!("UART 16550: unknown interrupt");
        }
    }

    fn transmit_sync(&self, bytes: &[u8]) {
        let prev_ier = self.regs.ier.extract();
        if prev_ier.is_set(IER::ETBEI) {
            self.regs.ier.modify_no_read(prev_ier, IER::ETBEI::CLEAR);
        }

        for byte in bytes.iter() {
            while !self.regs.lsr.is_set(LSR::TxterEmpty) {}
            self.regs.rbr_thr.write(THR::Data.val((*byte).into()));
        }

        // Restore the IER register to its original state
        self.regs.ier.set(prev_ier.get());
    }

    fn transmit_continue(&self) {
        // This should only be called as a result of a transmit
        // interrupt

        // Get the current transmission information
        let mut index = self.tx_index.get();
        let tx_data = self.tx_buffer.take().expect("UART 16550: no tx buffer");

        if index < self.tx_len.get() {
            // Still data to send
            while index < self.tx_len.get() && self.regs.lsr.is_set(LSR::THRE) {
                // self.regs.rbr_thr.write(THR::Data.val(tx_data[index]));
                self.regs
                    .rbr_thr
                    .write(THR::Data.val(tx_data[index].into()));
                index += 1;
            }

            // Put the updated index and buffer back, wait for an
            // interrupt
            self.tx_index.set(index);
            self.tx_buffer.replace(tx_data);
        } else {
            // We are finished, disable tx interrupts
            self.regs.ier.modify(IER::ETBEI::CLEAR);

            // Callback to the client
            self.tx_client
                .map(move |client| client.transmitted_buffer(tx_data, self.tx_len.get(), Ok(())));
        }
    }

    fn receive(&self) {
        // Receive interrupts must only be enabled when we're currently holding
        // a buffer to receive data into:
        let rx_buffer = self.rx_buffer.take().expect("UART 16550: no rx buffer");
        let len = self.rx_len.get();
        let mut index = self.rx_index.get();

        // Read in a while loop, until no more data in the FIFO
        while self.regs.lsr.is_set(LSR::DR) && index < len {
            // rx_buffer[index] = self.regs.rbr_thr.get();
            rx_buffer[index] = self.regs.rbr_thr.get().try_into().unwrap();
            index += 1;
        }

        // Check whether we've read sufficient data:
        if index == len {
            // We're done, disable interrupts and return to the client:
            self.regs.ier.modify(IER::ERBFI::CLEAR);

            self.rx_client.map(move |client| {
                client.received_buffer(rx_buffer, len, Ok(()), hil::uart::Error::None)
            });
        } else {
            // Store the new index and place the buffer back:
            self.rx_index.set(index);
            self.rx_buffer.replace(rx_buffer);
        }
    }
}

impl hil::uart::Configure for Ast2705Uart16550<'_> {
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

impl<'a> hil::uart::Transmit<'a> for Ast2705Uart16550<'a> {
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

        // Enable interrupts for the transmitter holding register
        // being empty such that we can callback to the client
        self.regs.ier.modify(IER::ETBEI::SET);

        // Start transmitting the first data word(s) already
        let mut index = 0;
        // while index < tx_len && self.regs.lsr.is_set(LSR::THREmpty) {
        while index < tx_len {
            while !self.regs.lsr.is_set(LSR::THRE) {}
            self.regs
                .rbr_thr
                .write(THR::Data.val(tx_data[index].into()));
            index += 1;
        }

        // Store the required buffer and information for the interrupt
        // handler
        self.tx_buffer.replace(tx_data);
        self.tx_len.set(tx_len);
        self.tx_index.set(index);

        Ok(())
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl<'a> hil::uart::Receive<'a> for Ast2705Uart16550<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // Ensure the provided buffer holds at least `rx_len` bytes, and
        // `rx_len` is strictly positive (otherwise we'd need to use deferred
        // calls):
        if rx_buffer.len() < rx_len && rx_len > 0 {
            return Err((ErrorCode::SIZE, rx_buffer));
        }

        // Store the receive buffer and byte count. We cannot call into the
        // generic receive routine here, as the client callback needs to be
        // called from another call stack. Hence simply enable interrupts here.
        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);
        self.rx_index.set(0);

        // Enable receive interrupts:
        self.regs.ier.modify(IER::ERBFI::SET);

        Ok(())
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
