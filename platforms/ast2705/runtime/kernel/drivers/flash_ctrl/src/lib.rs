// Licensed under the Apache-2.0 license

#![cfg_attr(target_arch = "riscv32", no_std)]

use caliptra_mcu_registers_generated::primary_flash_ctrl::{
    bits::{CtrlRegwen, FlControl, FlInterruptEnable, FlInterruptState, OpStatus},
    regs::PrimaryFlashCtrl,
    PRIMARY_FLASH_CTRL_ADDR,
};
use caliptra_mcu_registers_generated::secondary_flash_ctrl::SECONDARY_FLASH_CTRL_ADDR;
use core::ops::{Index, IndexMut};
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use registers_peripherals::aspeed_spi::AspeedSpiRegisters;

pub const PAGE_SIZE: usize = 256;
pub const ERASE_SECTOR_SIZE: usize = 4096;
const FLASH_MAX_SIZE: usize = 256 * 1024 * 1024;
pub const FLASH_MAX_PAGES: usize = 256 * 1024 * 1024 / PAGE_SIZE;

const FMC_MAP_MEMORY: usize = 0x0C00_0000;

#[derive(Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum FlashOperation {
    ReadPage = 1,
    WritePage = 2,
    ErasePage = 3,
}

impl TryInto<FlashOperation> for u32 {
    type Error = ();

    fn try_into(self) -> Result<FlashOperation, Self::Error> {
        match self {
            1 => Ok(FlashOperation::ReadPage),
            2 => Ok(FlashOperation::WritePage),
            3 => Ok(FlashOperation::ErasePage),
            _ => Err(()),
        }
    }
}

pub struct Ast2705FlashPage(pub [u8; PAGE_SIZE]);

impl Default for Ast2705FlashPage {
    fn default() -> Self {
        Self([0; PAGE_SIZE])
    }
}

impl Index<usize> for Ast2705FlashPage {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for Ast2705FlashPage {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl AsMut<[u8]> for Ast2705FlashPage {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

pub struct Ast2705FlashCtrl<'a> {
    registers: StaticRef<AspeedSpiRegisters>,
    flash_client: OptionalCell<&'a dyn hil::flash::Client<Ast2705FlashCtrl<'a>>>,
    read_buf: TakeCell<'static, Ast2705FlashPage>,
    write_buf: TakeCell<'static, Ast2705FlashPage>,
    deferred_call: DeferredCall,
}

impl<'a> Ast2705FlashCtrl<'a> {
    pub fn new(base: StaticRef<AspeedSpiRegisters>) -> Ast2705FlashCtrl<'a> {
        Ast2705FlashCtrl {
            registers: base,
            flash_client: OptionalCell::empty(),
            read_buf: TakeCell::empty(),
            write_buf: TakeCell::empty(),
            deferred_call: DeferredCall::new(),
        }
    }

    pub fn init(&self) {}
}

// Completes a read that was already performed synchronously in `read_page`
// (the AHB-mapped flash window is plain memory, so there is nothing to
// poll). The completion is delivered through a deferred call rather than
// directly from `read_page` because Tock's split-phase HIL contract forbids
// invoking the client callback within the same call that started the
// operation. Boards must call `register()` on the driver during init.
impl DeferredCallClient for Ast2705FlashCtrl<'_> {
    fn handle_deferred_call(&self) {
        if let Some(buf) = self.read_buf.take() {
            self.flash_client.map(move |client| {
                client.read_complete(buf, Ok(()));
            });
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

impl<C: hil::flash::Client<Self>> hil::flash::HasClient<'static, C> for Ast2705FlashCtrl<'_> {
    fn set_client(&self, client: &'static C) {
        self.flash_client.set(client);
    }
}

impl hil::flash::Flash for Ast2705FlashCtrl<'_> {
    type Page = Ast2705FlashPage;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        // Check if the page number is valid
        if page_number >= FLASH_MAX_PAGES {
            return Err((ErrorCode::INVAL, buf));
        }

        if self.read_buf.is_some() {
            return Err((ErrorCode::BUSY, buf));
        }

        // The AHB-mapped flash content window sits at a fixed offset above
        // the FMC/SPI controller register base (see FMC_MAP_MEMORY); reading
        // it is a plain memory load, same as
        // platforms/ast2705/rom/src/flash/flash_drv.rs's `read`.
        let ctrl_base = &*self.registers as *const AspeedSpiRegisters as usize;
        let flash_ptr = (ctrl_base + FMC_MAP_MEMORY + page_number * PAGE_SIZE) as *const u8;

        // SAFETY: `flash_ptr` points into the AHB-mapped flash content
        // window, which decodes read-only memory (not registers with side
        // effects) for the flash device's full capacity, for the lifetime
        // of this driver.
        let src = unsafe { core::slice::from_raw_parts(flash_ptr, PAGE_SIZE) };
        buf.as_mut().copy_from_slice(src);

        // Save the buffer
        self.read_buf.replace(buf);
        self.deferred_call.set();

        Ok(())
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        // Check if the page number is valid
        if page_number >= FLASH_MAX_PAGES {
            return Err((ErrorCode::INVAL, buf));
        }

        Ok(())
    }

    fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode> {
        if page_number >= FLASH_MAX_PAGES {
            return Err(ErrorCode::INVAL);
        }

        Ok(())
    }
}
