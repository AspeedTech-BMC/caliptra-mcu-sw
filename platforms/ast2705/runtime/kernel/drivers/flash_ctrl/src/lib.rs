// Licensed under the Apache-2.0 license

#![cfg_attr(target_arch = "riscv32", no_std)]

use caliptra_mcu_aspeed_soc::{AhbApertureGate, AhbWindowGate};
use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use registers_peripherals::aspeed_spi::{AspeedSpiRegisters, Ce0Ctrl, CeCtrl, CmdCtrl};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
use tock_registers::registers::ReadWrite;

/// Ce0Ctrl clock-divider bits: [27:24] | [11:8]
const CLK_FREQ_MASK: u32 = 0x0f00_0f00;
/// cmd_ctrl: enable data byte 0 only
const CMD_BYTE0_ONLY: u32 = 0xfe;

// NOR flash geometry.
pub const PAGE_SIZE: usize = 256;
pub const ERASE_SECTOR_SIZE: usize = 0x1000;

// NOR flash opcodes and status bits.
const CMD_WREN: u8 = 0x06;
const CMD_PAGE_PROGRAM: u8 = 0x02;
const CMD_SECTOR_ERASE: u8 = 0x20;
#[allow(dead_code)]
const CMD_RDID: u8 = 0x9f;
const STATUS_WIP: u8 = 0x01;
#[allow(dead_code)]
pub const SNOR_SFDP_MAGIC: u32 = 0x5044_4653;

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SpiEngineError {
    Inval,
    Timeout,
}

/// Core clock driving `mcycle`; same value used by the USB driver's delay helpers.
const TIMER_FREQUENCY_HZ: u64 = 400_000_000;

fn udelay(us: u32) {
    let start = caliptra_mcu_romtime::mcycle();
    let ticks = TIMER_FREQUENCY_HZ / 1_000_000 * us as u64;
    while caliptra_mcu_romtime::mcycle().wrapping_sub(start) < ticks {}
}

/// Poll interval and iteration budget for `wait_not_busy()`, sized well above
/// typical SPI NOR datasheet max times (page program ~2-3ms, sector erase
/// ~300-400ms) to leave margin for slower/aged parts without hanging forever.
const PAGE_PROGRAM_POLL_INTERVAL_US: u32 = 50;
const PAGE_PROGRAM_TIMEOUT_ITERS: u32 = 200; // 50us * 200 = 10ms
const SECTOR_ERASE_POLL_INTERVAL_US: u32 = 500;
const SECTOR_ERASE_TIMEOUT_ITERS: u32 = 2000; // 500us * 2000 = 1s

/// One controller instance (register base + mmap window), shared by every CS on it.
#[derive(Clone, Copy)]
pub struct AspeedSpiController {
    pub regs: StaticRef<AspeedSpiRegisters>,
    /// Mmap base; each CS's `decode_start` offsets from here.
    pub mem_base: u32,
    pub window_gate: Option<AhbApertureGate>,
}

/// Per-chip-select config.
#[derive(Clone, Copy, Debug)]
pub struct AspeedSpiCsConfig {
    /// ceN register set (0-3).
    pub cs: u32,
    pub decode_start: u32,
    pub decode_size: u32,
    /// Ce0Ctrl clock-divider bits (ClkFreqHi[27:24] | ClkFreqLo[11:8]).
    pub clk_div_bits: u32,
    pub use_4byte_addr: bool,
    /// AutoRead opcode, e.g. 0x03 READ or 0x0B FAST_READ.
    pub read_opcode: u8,
    /// Dummy cycles (not bytes) after the address.
    pub read_dummy_clks: u32,
}

// Register-level SPI engine: one chip-select's view into a controller.

/// Generic ASPEED SPI-family engine (FMC CE0-CE3, or SPI0/SPI1's single CS).
pub struct AspeedSpiCtrl {
    controller: AspeedSpiController,
    cfg: AspeedSpiCsConfig,
}

impl AspeedSpiCtrl {
    pub fn new(controller: AspeedSpiController, cfg: AspeedSpiCsConfig) -> Self {
        Self { controller, cfg }
    }

    fn with_window<T>(&self, f: impl FnOnce() -> T) -> T {
        match self.controller.window_gate {
            Some(gate) => {
                let prev = gate.engage();
                let result = f();
                gate.restore(prev);
                result
            }
            None => f(),
        }
    }

    pub fn init(&self) {
        self.with_window(|| {
            let ce_type = self.controller.regs.conf.get();
            self.controller
                .regs
                .conf
                .set(ce_type | (1u32 << (16 + self.cfg.cs)));

            let start = self.cfg.decode_start;
            let end = self.cfg.decode_start + self.cfg.decode_size.saturating_sub(1);
            self.decoding_range_set(Self::segment_addr_val(start, end));

            if self.cfg.use_4byte_addr {
                let mode4b = match self.cfg.cs {
                    0 => CeCtrl::Ce0Mode4B::SET + CeCtrl::Ce0Mode4BExt::SET,
                    1 => CeCtrl::Ce1Mode4B::SET + CeCtrl::Ce1Mode4BExt::SET,
                    2 => CeCtrl::Ce2Mode4B::SET + CeCtrl::Ce2Mode4BExt::SET,
                    _ => CeCtrl::Ce3Mode4B::SET + CeCtrl::Ce3Mode4BExt::SET,
                };
                self.controller.regs.ce_ctrl.modify(mode4b);
            }

            let ctrl_val = self.cfg.clk_div_bits
                | (u32::from(self.cfg.read_opcode) << 16)
                | Self::dummy_clk_bits(self.cfg.read_dummy_clks);
            self.ce_ctrl_set(ctrl_val);
        })
    }

    /// Dummy clock cycles -> Ce0Ctrl FrdWait bits.
    fn dummy_clk_bits(clk_cycles: u32) -> u32 {
        let bytes = clk_cycles / 8;
        ((bytes & 0x3) << 6) | (((bytes & 0x4) >> 2) << 14)
    }

    /// start_addr <= decoded_range < end_addr
    fn segment_addr_val(start: u32, end: u32) -> u32 {
        ((start >> 16) & 0x0000_ffff) | ((end.wrapping_add(1)) & 0xffff_0000)
    }

    // ---- Per-CS register accessors ----

    fn decoding_range_set(&self, val: u32) {
        match self.cfg.cs {
            0 => self.controller.regs.ce0_addr_decoding.set(val),
            1 => self.controller.regs.ce1_addr_decoding.set(val),
            2 => self.controller.regs.ce2_addr_decoding.set(val),
            _ => self.controller.regs.ce3_addr_decoding.set(val),
        }
    }

    fn ce_ctrl_reg(&self) -> &ReadWrite<u32, Ce0Ctrl::Register> {
        match self.cfg.cs {
            0 => &self.controller.regs.ce0_ctrl,
            1 => &self.controller.regs.ce1_ctrl,
            2 => &self.controller.regs.ce2_ctrl,
            _ => &self.controller.regs.ce3_ctrl,
        }
    }

    fn ce_ctrl_get(&self) -> u32 {
        self.ce_ctrl_reg().get()
    }

    fn ce_ctrl_set(&self, val: u32) {
        self.ce_ctrl_reg().set(val)
    }

    fn mem_base(&self) -> u32 {
        self.controller.mem_base + self.cfg.decode_start
    }

    // ---- AutoRead-mode debug reads ----

    fn read_status_cmd(&self) -> u8 {
        self.with_window(|| {
            let flash_addr = self.mem_base();

            let ctrl_bak = self.ce_ctrl_get();
            self.ce_ctrl_set(
                (ctrl_bak & CLK_FREQ_MASK) | (0x05 << 16) | Ce0Ctrl::Mode::NormalRead.value,
            );
            self.controller.regs.cmd_ctrl.set(CMD_BYTE0_ONLY);

            let status = unsafe { core::ptr::read_volatile(flash_addr as *const u8) };

            self.ce_ctrl_set(
                (self.ce_ctrl_get() & CLK_FREQ_MASK)
                    | Ce0Ctrl::Mode::UserMode.value
                    | Ce0Ctrl::CsStop::Active.value,
            );
            self.ce_ctrl_set(ctrl_bak);
            self.controller.regs.cmd_ctrl.set(0x0);

            status
        })
    }

    #[allow(dead_code)]
    pub fn sfdp_magic(&self) -> u32 {
        self.with_window(|| {
            let flash_addr = self.mem_base();

            let ctrl_bak = self.ce_ctrl_get();
            self.ce_ctrl_set(
                (ctrl_bak & CLK_FREQ_MASK)
                    | (0x5a << 16)
                    | Ce0Ctrl::FrdWaitLo.val(1).value
                    | Ce0Ctrl::Mode::NormalRead.value,
            );
            self.controller.regs.cmd_ctrl.modify(CmdCtrl::Force3B::SET); // SFDP is always 3-byte addressed

            let magic = unsafe { core::ptr::read_volatile(flash_addr as *const u32) };

            self.ce_ctrl_set(
                (self.ce_ctrl_get() & CLK_FREQ_MASK)
                    | Ce0Ctrl::Mode::UserMode.value
                    | Ce0Ctrl::CsStop::Active.value,
            );
            self.ce_ctrl_set(ctrl_bak);
            self.controller.regs.cmd_ctrl.set(0x0);

            magic
        })
    }

    // ---- User-mode bit-banged command primitives ----

    fn user_cs_assert(&self, ctrl_bak: u32) {
        let base = (ctrl_bak & CLK_FREQ_MASK) | Ce0Ctrl::Mode::UserMode.value;
        self.ce_ctrl_set(base | Ce0Ctrl::CsStop::Active.value);
        self.ce_ctrl_set(base);
        let _ = self.ce_ctrl_get(); // flush: CS must assert before MOSI traffic
    }

    fn user_cs_deassert(&self, ctrl_bak: u32) {
        let base = (ctrl_bak & CLK_FREQ_MASK) | Ce0Ctrl::Mode::UserMode.value;
        self.ce_ctrl_set(base | Ce0Ctrl::CsStop::Active.value);
        self.ce_ctrl_set(ctrl_bak);
    }

    fn user_write_byte(&self, byte: u8) {
        unsafe { core::ptr::write_volatile(self.mem_base() as *mut u8, byte) }
    }

    fn write_addr_bytes(&self, addr: u32) {
        if self.cfg.use_4byte_addr {
            self.user_write_byte(((addr >> 24) & 0xFF) as u8);
        }
        self.user_write_byte(((addr >> 16) & 0xFF) as u8);
        self.user_write_byte(((addr >> 8) & 0xFF) as u8);
        self.user_write_byte((addr & 0xFF) as u8);
    }

    #[allow(dead_code)]
    pub fn read_id(&self) -> [u8; 3] {
        self.with_window(|| {
            let ctrl_bak = self.ce_ctrl_get();

            self.user_cs_assert(ctrl_bak);
            self.user_write_byte(CMD_RDID);

            let mut id = [0u8; 3];
            for b in id.iter_mut() {
                *b = unsafe { core::ptr::read_volatile(self.mem_base() as *const u8) };
            }

            self.user_cs_deassert(ctrl_bak);
            id
        })
    }

    // ---- Program/erase ops (all synchronous, poll `wait_not_busy`) ----

    fn send_wren(&self, ctrl_bak: u32) {
        self.user_cs_assert(ctrl_bak);
        self.user_write_byte(CMD_WREN);
        self.user_cs_deassert(ctrl_bak);
    }

    fn wait_not_busy(
        &self,
        poll_interval_us: u32,
        timeout_iters: u32,
    ) -> Result<(), SpiEngineError> {
        for _ in 0..timeout_iters {
            if self.read_status_cmd() & STATUS_WIP == 0 {
                return Ok(());
            }
            udelay(poll_interval_us);
        }
        Err(SpiEngineError::Timeout)
    }

    /// `data` must fit within one page and the range must already be erased.
    fn program_chunk(&self, addr: u32, data: &[u8]) -> Result<(), SpiEngineError> {
        let ctrl_bak = self.ce_ctrl_get();
        self.send_wren(ctrl_bak);

        self.user_cs_assert(ctrl_bak);
        self.user_write_byte(CMD_PAGE_PROGRAM);
        self.write_addr_bytes(addr);
        for &b in data {
            self.user_write_byte(b);
        }
        self.user_cs_deassert(ctrl_bak);

        self.wait_not_busy(PAGE_PROGRAM_POLL_INTERVAL_US, PAGE_PROGRAM_TIMEOUT_ITERS)
    }

    fn erase_sector(&self, addr: u32) -> Result<(), SpiEngineError> {
        let ctrl_bak = self.ce_ctrl_get();
        self.send_wren(ctrl_bak);

        self.user_cs_assert(ctrl_bak);
        self.user_write_byte(CMD_SECTOR_ERASE);
        self.write_addr_bytes(addr);
        self.user_cs_deassert(ctrl_bak);

        self.wait_not_busy(SECTOR_ERASE_POLL_INTERVAL_US, SECTOR_ERASE_TIMEOUT_ITERS)
    }

    // ---- Public byte-addressed API ----

    pub fn capacity(&self) -> usize {
        self.cfg.decode_size as usize
    }

    fn check_bounds(&self, offset: usize, len: usize) -> Result<(), SpiEngineError> {
        match offset.checked_add(len) {
            Some(end) if end <= self.capacity() => Ok(()),
            _ => Err(SpiEngineError::Inval),
        }
    }

    pub fn read(&self, buffer: &mut [u8], address: usize) -> Result<(), SpiEngineError> {
        self.check_bounds(address, buffer.len())?;
        self.with_window(|| {
            let src = self.mem_base() as usize + address;
            unsafe {
                core::ptr::copy_nonoverlapping(src as *const u8, buffer.as_mut_ptr(), buffer.len());
            }
            Ok(())
        })
    }

    /// Chunks at page boundaries; caller must `erase()` first.
    pub fn write(&self, buffer: &[u8], address: usize) -> Result<(), SpiEngineError> {
        self.check_bounds(address, buffer.len())?;
        self.with_window(|| {
            let mut done = 0;
            while done < buffer.len() {
                let offset = address + done;
                let page_room = PAGE_SIZE - (offset % PAGE_SIZE);
                let chunk = core::cmp::min(page_room, buffer.len() - done);
                self.program_chunk(offset as u32, &buffer[done..done + chunk])?;
                done += chunk;
            }
            Ok(())
        })
    }

    /// Erases every 4 KB sector overlapping `[address, address + length)`.
    pub fn erase(&self, address: usize, length: usize) -> Result<(), SpiEngineError> {
        self.check_bounds(address, length)?;
        self.with_window(|| {
            if length == 0 {
                return Ok(());
            }
            let first_sector = address / ERASE_SECTOR_SIZE;
            let last_sector = (address + length - 1) / ERASE_SECTOR_SIZE;
            for sector in first_sector..=last_sector {
                self.erase_sector((sector * ERASE_SECTOR_SIZE) as u32)?;
            }
            Ok(())
        })
    }
}

fn map_engine_err(e: SpiEngineError) -> ErrorCode {
    match e {
        SpiEngineError::Inval => ErrorCode::INVAL,
        SpiEngineError::Timeout => ErrorCode::FAIL,
    }
}

// Tock `hil::flash::Flash` wrapper.

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
    engine: AspeedSpiCtrl,
    flash_client: OptionalCell<&'a dyn hil::flash::Client<Ast2705FlashCtrl<'a>>>,
    read_buf: TakeCell<'static, Ast2705FlashPage>,
    write_buf: TakeCell<'static, Ast2705FlashPage>,
    erase_pending: Cell<bool>,
    // Scratch sector for erase_page()'s read-modify-erase-write (HIL is page-granular, hardware isn't).
    erase_scratch: TakeCell<'static, [u8; ERASE_SECTOR_SIZE]>,
    deferred_call: DeferredCall,
}

impl<'a> Ast2705FlashCtrl<'a> {
    pub fn new(
        controller: AspeedSpiController,
        cfg: AspeedSpiCsConfig,
        erase_scratch: &'static mut [u8; ERASE_SECTOR_SIZE],
    ) -> Ast2705FlashCtrl<'a> {
        Ast2705FlashCtrl {
            engine: AspeedSpiCtrl::new(controller, cfg),
            flash_client: OptionalCell::empty(),
            read_buf: TakeCell::empty(),
            write_buf: TakeCell::empty(),
            erase_pending: Cell::new(false),
            erase_scratch: TakeCell::new(erase_scratch),
            deferred_call: DeferredCall::new(),
        }
    }

    /// Never call on the boot-critical FMC CE0 instance -- it may be XIP.
    pub fn init(&self) {
        self.engine.init();
    }

    pub fn read_id(&self) -> [u8; 3] {
        self.engine.read_id()
    }

    fn max_pages(&self) -> usize {
        self.engine.capacity() / PAGE_SIZE
    }

    fn busy(&self) -> bool {
        self.read_buf.is_some() || self.write_buf.is_some() || self.erase_pending.get()
    }
}

// Ops run synchronously; the deferred call only defers the completion callback (Tock split-phase contract).
impl DeferredCallClient for Ast2705FlashCtrl<'_> {
    fn handle_deferred_call(&self) {
        if let Some(buf) = self.read_buf.take() {
            self.flash_client.map(move |client| {
                client.read_complete(buf, Ok(()));
            });
        } else if let Some(buf) = self.write_buf.take() {
            self.flash_client.map(move |client| {
                client.write_complete(buf, Ok(()));
            });
        } else if self.erase_pending.take() {
            self.flash_client.map(|client| {
                client.erase_complete(Ok(()));
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
        if page_number >= self.max_pages() {
            return Err((ErrorCode::INVAL, buf));
        }
        if self.busy() {
            return Err((ErrorCode::BUSY, buf));
        }

        if let Err(e) = self.engine.read(buf.as_mut(), page_number * PAGE_SIZE) {
            return Err((map_engine_err(e), buf));
        }

        self.read_buf.replace(buf);
        self.deferred_call.set();

        Ok(())
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        if page_number >= self.max_pages() {
            return Err((ErrorCode::INVAL, buf));
        }
        if self.busy() {
            return Err((ErrorCode::BUSY, buf));
        }

        // NOR PROGRAM can only clear bits (1 -> 0); caller must erase first.
        if let Err(e) = self.engine.write(buf.as_mut(), page_number * PAGE_SIZE) {
            return Err((map_engine_err(e), buf));
        }

        self.write_buf.replace(buf);
        self.deferred_call.set();

        Ok(())
    }

    /// Erases just `page_number`'s 256 bytes, preserving the rest of its sector.
    fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode> {
        if page_number >= self.max_pages() {
            return Err(ErrorCode::INVAL);
        }
        if self.busy() {
            return Err(ErrorCode::BUSY);
        }

        let scratch = self.erase_scratch.take().ok_or(ErrorCode::FAIL)?;
        let (scratch, result) = self.erase_sector_preserving_pages(scratch, page_number);
        self.erase_scratch.replace(scratch);
        result?;

        self.erase_pending.set(true);
        self.deferred_call.set();

        Ok(())
    }
}

impl Ast2705FlashCtrl<'_> {
    /// Blanks `page_number`, reprograms the rest of its sector from `scratch`, always hands `scratch` back.
    fn erase_sector_preserving_pages(
        &self,
        scratch: &'static mut [u8; ERASE_SECTOR_SIZE],
        page_number: usize,
    ) -> (&'static mut [u8; ERASE_SECTOR_SIZE], Result<(), ErrorCode>) {
        let pages_per_sector = ERASE_SECTOR_SIZE / PAGE_SIZE;
        let sector_start_page = (page_number / pages_per_sector) * pages_per_sector;
        let sector_addr = sector_start_page * PAGE_SIZE;
        let target_offset = (page_number - sector_start_page) * PAGE_SIZE;

        if let Err(e) = self.engine.read(scratch.as_mut(), sector_addr) {
            return (scratch, Err(map_engine_err(e)));
        }

        scratch[target_offset..target_offset + PAGE_SIZE].fill(0xFF);

        if let Err(e) = self.engine.erase(sector_addr, ERASE_SECTOR_SIZE) {
            return (scratch, Err(map_engine_err(e)));
        }

        for (i, chunk) in scratch.chunks(PAGE_SIZE).enumerate() {
            let chunk_offset = i * PAGE_SIZE;
            if chunk_offset == target_offset {
                continue; // already erased above; programming 0xFF is a no-op
            }
            if let Err(e) = self.engine.write(chunk, sector_addr + chunk_offset) {
                return (scratch, Err(map_engine_err(e)));
            }
        }

        (scratch, Ok(()))
    }
}
