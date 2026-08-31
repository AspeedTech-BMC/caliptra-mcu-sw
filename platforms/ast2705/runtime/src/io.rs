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
use core::fmt::Write;
use core::panic::PanicInfo;
#[cfg(not(feature = "release"))]
use core::ptr::addr_of;
use core::ptr::addr_of_mut;
use core::ptr::{read_volatile, write_volatile};
#[cfg(not(feature = "release"))]
use kernel::debug;
use kernel::debug::IoWrite;

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
