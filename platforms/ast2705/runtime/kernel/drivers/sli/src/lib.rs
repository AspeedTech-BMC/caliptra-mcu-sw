// Licensed under the Apache-2.0 license

#![no_std]

/// Calibration log, compiled out when the `debug-log` feature is disabled.
#[cfg(feature = "debug-log")]
macro_rules! sli_log {
    ($($arg:tt)*) => { caliptra_mcu_romtime::println!($($arg)*) };
}
#[cfg(not(feature = "debug-log"))]
macro_rules! sli_log {
    ($($arg:tt)*) => {};
}

use caliptra_mcu_capsules_runtime::bootdev::hil::{KernelImageClient, KernelImageLoader};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::StaticRef;
use registers_peripherals::ast2705::aspeed_scu0::{SCU0Registers, Scu0SliStatus};
use registers_peripherals::ast2705::aspeed_scu1::{SCU1Registers, Scu1SliStatus};
use registers_peripherals::ast2705::aspeed_sli::{
    AhbcRegisters, CtrlI, CtrlII, CtrlIIISlih, CtrlIIISlim, CtrlIIISliv, CtrlIVSlim, IntcProt,
    Intr, IoIntcRegisters, MarbFunc, SliHRegisters, SliMRegisters, SliVRegisters, AHBC0_BASE,
    AHBC1_BASE, IO_INTC_BASE, SLI0_BASE, SLI1_BASE, SLIH_OFFSET, SLIM_OFFSET, SLIV_OFFSET,
};

/// Common MMIO fields shared by all three SLI sub-blocks (H, M, V).
trait SliBlockCommon {
    fn intr_status(&self) -> &ReadWrite<u32, Intr::Register>;
    fn ctrl1(&self) -> &ReadWrite<u32, CtrlI::Register>;
}

impl SliBlockCommon for SliHRegisters {
    fn intr_status(&self) -> &ReadWrite<u32, Intr::Register> {
        &self.intr_status
    }
    fn ctrl1(&self) -> &ReadWrite<u32, CtrlI::Register> {
        &self.ctrl1
    }
}
impl SliBlockCommon for SliMRegisters {
    fn intr_status(&self) -> &ReadWrite<u32, Intr::Register> {
        &self.intr_status
    }
    fn ctrl1(&self) -> &ReadWrite<u32, CtrlI::Register> {
        &self.ctrl1
    }
}
impl SliBlockCommon for SliVRegisters {
    fn intr_status(&self) -> &ReadWrite<u32, Intr::Register> {
        &self.intr_status
    }
    fn ctrl1(&self) -> &ReadWrite<u32, CtrlI::Register> {
        &self.ctrl1
    }
}
impl<T: SliBlockCommon> SliBlockCommon for StaticRef<T> {
    fn intr_status(&self) -> &ReadWrite<u32, Intr::Register> {
        (**self).intr_status()
    }
    fn ctrl1(&self) -> &ReadWrite<u32, CtrlI::Register> {
        (**self).ctrl1()
    }
}

/// PHY TX clock frequency selection, matching `CtrlIIISlih::TxClkSel` values.
#[derive(Clone, Copy, Debug)]
pub enum TxClk {
    Mhz25 = 0,
    Mhz800 = 1,
    Mhz400 = 2,
    Mhz200 = 3,
    Ghz1 = 5,
    Mhz500 = 6,
    Mhz250 = 7,
}

const CAL_DELAY_US: u32 = 200;
const SET_DELAY_US: u32 = 8;

const SLIH_COARSE_D_BEGIN: i32 = 6;
const SLIH_COARSE_D_END: i32 = 28;

const SLIM_COARSE_D_BEGIN: i32 = 0;
const SLIM_COARSE_D_END: i32 = 28;
const SLIM_FINE_MARGIN: i32 = 5;
const SLIM_DEFAULT_DELAY: i32 = 12;
const SLIM_RETRY_COUNT: i32 = 50;

const SLIV_COARSE_D_BEGIN: i32 = 0;
const SLIV_COARSE_D_END: i32 = 28;

const SLI_MAX_POLL_CNT_CLEAR: i32 = 10;
const SLI_MAX_POLL_CNT_SUSPEND: i32 = 10;

const AHBC_MAX_TIMEOUT: u32 = 0x1ff;

const TIMER_FREQUENCY_HZ: u64 = 400_000_000;

fn udelay(us: u32) {
    let start = caliptra_mcu_romtime::mcycle();
    let ticks = TIMER_FREQUENCY_HZ / 1_000_000 * us as u64;
    while caliptra_mcu_romtime::mcycle().wrapping_sub(start) < ticks {}
}

fn mdelay(ms: u32) {
    udelay(ms * 1_000);
}

/// Write a calibration window `(first, last)` into a scratch register.
/// Each of `first` and `last` occupies one byte at `bit_offset` and `bit_offset+8`.
fn sli_win_write(reg: &ReadWrite<u32>, bit_offset: u32, first: i32, last: i32) {
    let packed = ((last as u32 & 0xff) << (bit_offset + 8)) | ((first as u32 & 0xff) << bit_offset);
    let mask = 0xffffu32 << bit_offset;
    reg.set((reg.get() & !mask) | packed);
}

/// Read back a calibration window `(first, last)` from a scratch register.
fn sli_win_read(reg: &ReadWrite<u32>, bit_offset: u32) -> (i32, i32) {
    let v = reg.get();
    let first = ((v >> bit_offset) & 0xff) as i32;
    let last = ((v >> (bit_offset + 8)) & 0xff) as i32;
    (first, last)
}

// Raw bitmasks derived from register bitfields using `.value` (avoids Into<> ambiguity).
const INTR_RX_ERRORS: u32 =
    Intr::RxErr::SET.value | Intr::RxNack::SET.value | Intr::RxDisconn::SET.value;
const INTR_SUSPEND: u32 = Intr::TxSuspend::SET.value | Intr::RxSuspend::SET.value;

struct SliBlock {
    slim: StaticRef<SliMRegisters>,
    slih: StaticRef<SliHRegisters>,
    sliv: StaticRef<SliVRegisters>,
}

impl SliBlock {
    /// # Safety
    /// `base` must be the correct BootMCU-view base address for an SLI controller.
    unsafe fn new(base: u32) -> Self {
        Self {
            slim: StaticRef::new((base + SLIM_OFFSET) as *const SliMRegisters),
            slih: StaticRef::new((base + SLIH_OFFSET) as *const SliHRegisters),
            sliv: StaticRef::new((base + SLIV_OFFSET) as *const SliVRegisters),
        }
    }
}

pub struct AspeedSliCtrl {
    scu0: StaticRef<SCU0Registers>,
    scu1: StaticRef<SCU1Registers>,
    tx_clk: TxClk,
}

impl AspeedSliCtrl {
    pub const fn new(
        scu0: StaticRef<SCU0Registers>,
        scu1: StaticRef<SCU1Registers>,
        tx_clk: TxClk,
    ) -> Self {
        Self { scu0, scu1, tx_clk }
    }

    /// CPU-die TX calibration: AHB + M-bus delays.  Called from the CPU-die BootMCU.
    pub fn init_f(&self) {
        let die0 = unsafe { SliBlock::new(SLI0_BASE) };
        let die1 = unsafe { SliBlock::new(SLI1_BASE) };

        // Skip if already calibrated (engine clock already sped up).
        if die1.slih.ctrl3.read(CtrlIIISlih::ClkSel) != 0 {
            return;
        }

        // Skip on warm reboot.
        if self.scu1.sli_status.is_set(Scu1SliStatus::SliSkipCali) {
            sli_log!("[sli] SLI1 already initialized, skipping calibration");
            return;
        }

        ahbc_set_timeout(false);

        // Speed up engine clock on both dies.
        die1.slih.ctrl3.modify(CtrlIIISlih::ClkSel::Clk500M);
        die0.slih.ctrl3.modify(CtrlIIISlih::ClkSel::Clk500M);

        // Disable auto-clear and TX-clock-change reset (A1+ workaround).
        die1.slih.ctrl1.modify(
            CtrlI::AutoClrOffDat::SET + CtrlI::AutoClrOffClk::SET + CtrlI::NoRstTxClkChg::SET,
        );
        die0.slih.ctrl1.modify(
            CtrlI::AutoClrOffDat::SET + CtrlI::AutoClrOffClk::SET + CtrlI::NoRstTxClkChg::SET,
        );

        // Speed up CPU-die PHY TX clock; clear TX pad delays.
        die0.slih.ctrl3.modify(
            CtrlIIISlih::TxClkSel.val(self.tx_clk as u32)
                + CtrlIIISlih::PadDlyTx1.val(0)
                + CtrlIIISlih::PadDlyTx0.val(0),
        );

        calibrate_ahb_delay(&die1.slih, &self.scu1.sli_ahb_sliv_us_win);
        die1.slim.ctrl3.set(0);
        calibrate_mbus_delay(
            &die0.slim,
            &die1.slim,
            true,
            false,
            &self.scu1.sli_slim_ds_win_low,
            &self.scu1.sli_slim_ds_win_high,
        );

        sli_log!("[sli] SLI1 calibration completed, tx_clk={:?}", self.tx_clk);

        // Reset the remote (CPU-die) SLIH and wait for it to re-suspend.
        sli_clear(&die0.slih, CtrlI::ClearBus::SET.value);
        sli_wait_suspend(&die0.slih);
    }

    /// IO-die TX calibration: waits for CPU-die ready flag, then calibrates video.
    pub fn init_r(&self) {
        let die0 = unsafe { SliBlock::new(SLI0_BASE) };
        let die1 = unsafe { SliBlock::new(SLI1_BASE) };

        if self.scu1.sli_status.is_set(Scu1SliStatus::SliSkipCali) {
            sli_log!("[sli] SLI0 already initialized");
            mac_hotfix(&die1.slim);
            return;
        }

        // Wait for CPU-die SLI0 ready flag (up to 10 s).
        let mut retry = 100i32;
        let sli0_ready = loop {
            if self.scu1.sli_status.is_set(Scu1SliStatus::Sli0Ready) {
                break true;
            }
            retry -= 1;
            if retry <= 0 {
                break false;
            }
            mdelay(100);
        };

        if !sli0_ready {
            caliptra_mcu_romtime::println!("[sli] Timeout waiting for SLI0 calibration");
            return;
        }

        sli_clear(
            &die1.slih,
            CtrlI::ClearRx::SET.value | CtrlI::ClearBus::SET.value,
        );
        sli_wait_suspend(&die1.slih);
        udelay(CAL_DELAY_US);

        ahbc_set_timeout(true);
        caliptra_mcu_romtime::println!("[sli] SLI0 calibration completed");

        self.scu0.sli_status.modify(Scu0SliStatus::Sli1Ready::SET);
        self.scu1.sli_status.modify(Scu1SliStatus::SliSkipCali::SET);

        // Reset SLIM MARB before using the SLIM.
        die1.slim.marb_func_i.modify(MarbFunc::MarbClr::SET);

        // Clear IO INTC reset interrupt status.
        let intc = unsafe { &*(IO_INTC_BASE as *const IoIntcRegisters) };
        let status = intc.status.get();
        intc.status.set(status);

        intc.prot.modify(IntcProt::EnableCpudResetSrc::SET);

        // Calibrate video: US direction (IO→CPU), k on RX side.
        die1.sliv.ctrl3.set(0);
        calibrate_video_delay(
            &die0.sliv,
            &die1.sliv,
            false,
            true,
            &self.scu1.sli_ahb_sliv_us_win,
            16,
        );
        // Calibrate video: DS direction (CPU→IO), k on TX side.
        calibrate_video_delay(
            &die0.sliv,
            &die1.sliv,
            true,
            false,
            &self.scu0.sli_sliv_ds_win,
            16,
        );
    }
}

impl caliptra_mcu_dp_ast2705::DpReadyClient for AspeedSliCtrl {
    /// IO-die video calibration (`init_r`) depends on the DPMCU firmware
    /// being loaded and running, so it is deferred until DP signals ready.
    fn dp_ready(&self) {
        self.init_r();
    }
}

// ---------------------------------------------------------------------------
// AHBC timeout helpers
// ---------------------------------------------------------------------------

fn ahbc_set_timeout(enable: bool) {
    let val = if enable { AHBC_MAX_TIMEOUT } else { 0 };
    let ahbc1 = unsafe { &*(AHBC1_BASE as *const AhbcRegisters) };
    let ahbc0 = unsafe { &*(AHBC0_BASE as *const AhbcRegisters) };

    ahbc1.ch0_timeout.set(val);
    ahbc1.ch1_timeout.set(val);
    ahbc1.ch2_timeout.set(val);
    ahbc1.ch3_timeout.set(val);
    ahbc1.ch4_timeout.set(val);
    ahbc1.ch5_timeout.set(val);
    ahbc1.ch6_timeout.set(val);
    ahbc1.ch7_timeout.set(val);

    if enable {
        udelay(CAL_DELAY_US);
    }

    ahbc0.ch0_timeout.set(val);
    ahbc0.ch1_timeout.set(val);
    ahbc0.ch2_timeout.set(val);
    ahbc0.ch3_timeout.set(val);

    if enable {
        mdelay(10);
    }
}

// ---------------------------------------------------------------------------
// Interrupt / suspend helpers — generic over all three SLI sub-block types
// ---------------------------------------------------------------------------

fn sli_clear_intr<R: SliBlockCommon>(r: &R) {
    r.intr_status().set(0xfffff);
}

fn is_suspend<R: SliBlockCommon>(r: &R) -> i32 {
    let v = r.intr_status().get();
    if v & INTR_RX_ERRORS != 0 {
        -1
    } else if v & INTR_SUSPEND == INTR_SUSPEND {
        1
    } else {
        0
    }
}

fn sli_wait_suspend<R: SliBlockCommon>(r: &R) {
    sli_clear_intr(r);
    for _ in 0..SLI_MAX_POLL_CNT_SUSPEND {
        let v = r.intr_status().get();
        if v & INTR_RX_ERRORS != 0 || v & INTR_SUSPEND == INTR_SUSPEND {
            return;
        }
        udelay(1);
    }
}

/// Set `bits` in ctrl1, then poll until the self-clearing bits clear.
/// ClearBus does not self-clear and is excluded from the poll mask.
fn sli_clear<R: SliBlockCommon>(r: &R, bits: u32) {
    r.ctrl1().set(r.ctrl1().get() | bits);
    let wait_mask = bits & !CtrlI::ClearBus::SET.value;
    if wait_mask == 0 {
        return;
    }
    for _ in 0..SLI_MAX_POLL_CNT_CLEAR {
        if r.ctrl1().get() & wait_mask == 0 {
            return;
        }
        udelay(1);
    }
}

// ---------------------------------------------------------------------------
// AHB calibration (SLIH, IO-die RX)
// ---------------------------------------------------------------------------

fn sli_set_ahb_rx_delay(r: &SliHRegisters, d0: i32, d1: i32) {
    r.ctrl3
        .modify(CtrlIIISlih::PadDlyRx1.val(d1 as u32) + CtrlIIISlih::PadDlyRx0.val(d0 as u32));
    let _ = r.ctrl3.get();
    udelay(SET_DELAY_US);
}

fn calibrate_ahb_delay(io_slih: &SliHRegisters, win: &ReadWrite<u32>) {
    io_slih.ctrl1.modify(CtrlI::AutoSendTrnOff::SET);

    let mut d_first = -1i32;
    let mut d_last = -1i32;
    let mut win_size = 0i32;

    for dc in SLIH_COARSE_D_BEGIN..SLIH_COARSE_D_END {
        sli_set_ahb_rx_delay(io_slih, dc, dc);
        sli_clear(
            io_slih,
            CtrlI::ClearRx::SET.value | CtrlI::ClearBus::SET.value,
        );

        sli_clear_intr(io_slih);
        udelay(CAL_DELAY_US);

        if is_suspend(io_slih) > 0 {
            if d_first == -1 {
                d_first = dc;
            }
            d_last = dc;
            sli_log!("[sli] dc={} found", dc);
        } else if d_last != -1 {
            let w = d_last - d_first;
            if w > win_size {
                win_size = w;
                sli_win_write(win, 0, d_first, d_last);
            }
            d_first = -1;
            d_last = -1;
        }
    }

    if d_last != -1 && (d_last - d_first) > win_size {
        sli_win_write(win, 0, d_first, d_last);
    } else {
        let (f, l) = sli_win_read(win, 0);
        if f != 0 || l != 0 {
            d_first = f;
            d_last = l;
        } else {
            sli_log!("[sli] SLIH coarse: no window found, using default dc=11");
            d_first = 11;
            d_last = 11;
        }
    }

    let dc = (d_first + d_last) >> 1;
    sli_log!(
        "[sli] SLIH coarse: d_first={} d_last={} -> dc={}",
        d_first,
        d_last,
        dc
    );
    sli_set_ahb_rx_delay(io_slih, dc, dc);

    sli_clear(
        io_slih,
        CtrlI::ClearRx::SET.value | CtrlI::ClearBus::SET.value,
    );
    io_slih.ctrl1.modify(CtrlI::AutoSendTrnOff::CLEAR);
    sli_wait_suspend(io_slih);
}

// ---------------------------------------------------------------------------
// M-bus (SLIM) calibration
// ---------------------------------------------------------------------------

fn slim_set_delay_all(r: &SliMRegisters, d: i32, is_rx: bool) {
    if is_rx {
        r.ctrl3.modify(
            CtrlIIISlim::PadDly3.val(d as u32)
                + CtrlIIISlim::PadDly2.val(d as u32)
                + CtrlIIISlim::PadDly1.val(d as u32)
                + CtrlIIISlim::PadDly0.val(d as u32),
        );
        // CPUD reg access needs a dummy read to make sure it's accepted
        let _ = r.ctrl3.get();
    } else {
        r.ctrl4.modify(
            CtrlIVSlim::PadDlyTx3.val(d as u32)
                + CtrlIVSlim::PadDlyTx2.val(d as u32)
                + CtrlIVSlim::PadDlyTx1.val(d as u32)
                + CtrlIVSlim::PadDlyTx0.val(d as u32),
        );
        // CPUD reg access needs a dummy read to make sure it's accepted
        let _ = r.ctrl4.get();
    }
    udelay(SET_DELAY_US);
}

fn slim_set_delay_lane(r: &SliMRegisters, index: u32, d: i32, is_rx: bool) {
    let offset = index * 6;
    if is_rx {
        let v = r.ctrl3.get() & !(0x3f << offset);
        r.ctrl3.set(v | ((d as u32 & 0x3f) << offset));
        // CPUD reg access needs a dummy read to make sure it's accepted
        let _ = r.ctrl3.get();
    } else {
        let v = r.ctrl4.get() & !(0x3f << offset);
        r.ctrl4.set(v | ((d as u32 & 0x3f) << offset));
        // CPUD reg access needs a dummy read to make sure it's accepted
        let _ = r.ctrl4.get();
    }
    udelay(SET_DELAY_US);
}

fn slim_calibrate_lane(
    cpu_slim: &SliMRegisters,
    io_slim: &SliMRegisters,
    index: u32,
    begin: i32,
    end: i32,
    is_ds: bool,
    is_k_rx: bool,
    win: &ReadWrite<u32>,
) -> i32 {
    let (tx, rx, kx) = slim_dir(cpu_slim, io_slim, is_ds, is_k_rx);

    let mut d_first = -1i32;
    let mut d_last = -1i32;
    let dir = if is_ds { "DS" } else { "US" };
    let die = if is_ds ^ is_k_rx { "CPU" } else { "IO" };

    for _ in 0..SLIM_RETRY_COUNT {
        for d in begin..end {
            slim_set_delay_lane(kx, index, d, is_k_rx);
            sli_clear(tx, CtrlI::ResetTrigger::SET.value);
            sli_clear(rx, CtrlI::ResetTrigger::SET.value);

            sli_clear_intr(rx);
            udelay(CAL_DELAY_US);

            if is_suspend(rx) > 0 {
                if d_first == -1 {
                    d_first = d;
                }
                d_last = d;
                sli_log!("[slim-{}]({})({}) d={} found", index, dir, die, d);
            } else if d_last != -1 {
                break;
            }
        }

        if d_first != -1 && (d_last - d_first) >= 3 {
            break;
        }
        d_first = -1;
        d_last = -1;
        sli_log!("[slim-{}]({})({}) retry", index, dir, die);
    }

    if d_first == -1 {
        (begin + end) >> 1
    } else {
        let bit_offset = (index % 2) * 16;
        sli_win_write(win, bit_offset, d_first, d_last);
        (d_first + d_last) >> 1
    }
}

fn slim_dir<'a>(
    cpu: &'a SliMRegisters,
    io: &'a SliMRegisters,
    is_ds: bool,
    is_k_rx: bool,
) -> (&'a SliMRegisters, &'a SliMRegisters, &'a SliMRegisters) {
    let (tx, rx) = if is_ds { (cpu, io) } else { (io, cpu) };
    let kx = if is_k_rx { rx } else { tx };
    (tx, rx, kx)
}

fn calibrate_mbus_delay(
    cpu_slim: &SliMRegisters,
    io_slim: &SliMRegisters,
    is_ds: bool,
    is_k_rx: bool,
    win_low: &ReadWrite<u32>,
    win_high: &ReadWrite<u32>,
) {
    let (tx, rx, kx) = slim_dir(cpu_slim, io_slim, is_ds, is_k_rx);
    let dir = if is_ds { "DS" } else { "US" };
    let die = if is_ds ^ is_k_rx { "CPU" } else { "IO" };

    rx.ctrl1.modify(CtrlI::AutoSendTrnOff::SET);

    let mut d_first = -1i32;
    let mut d_last = -1i32;
    let mut win_size = 0i32;
    let mut dc_best = SLIM_DEFAULT_DELAY;

    'outer: for _ in 0..SLIM_RETRY_COUNT {
        for dc in SLIM_COARSE_D_BEGIN..SLIM_COARSE_D_END {
            slim_set_delay_all(kx, dc, is_k_rx);
            sli_clear(tx, CtrlI::ResetTrigger::SET.value);
            sli_clear(rx, CtrlI::ResetTrigger::SET.value);

            sli_clear_intr(rx);
            udelay(CAL_DELAY_US);

            if is_suspend(rx) > 0 {
                if d_first == -1 {
                    d_first = dc;
                }
                d_last = dc;
                sli_log!("[slim]({})({}) dc={} found", dir, die, dc);
            } else if d_last != -1 {
                let w = d_last - d_first;
                if w > win_size {
                    win_size = w;
                    dc_best = (d_first + d_last) >> 1;
                }
                d_first = -1;
                d_last = -1;
            }
        }

        if d_last != -1 && (d_last - d_first) > win_size {
            win_size = d_last - d_first;
            dc_best = (d_first + d_last) >> 1;
        }

        if win_size >= 3 {
            break 'outer;
        }
        d_first = -1;
        d_last = -1;
        sli_log!("[slim]({})({}) retry", dir, die);
    }

    if dc_best == 0 {
        sli_log!(
            "[slim]({})({}): no window found, using default dc={}",
            dir,
            die,
            SLIM_DEFAULT_DELAY
        );
        dc_best = SLIM_DEFAULT_DELAY;
    } else {
        sli_log!(
            "[slim]({})({}): win={} -> dc_best={}",
            dir,
            die,
            win_size,
            dc_best
        );
    }

    slim_set_delay_all(kx, dc_best, is_k_rx);

    let begin = (dc_best - SLIM_FINE_MARGIN).max(0);
    let end = (dc_best + SLIM_FINE_MARGIN).min(31);

    if win_size > 0 {
        let d0 = slim_calibrate_lane(cpu_slim, io_slim, 0, begin, end, is_ds, is_k_rx, win_low);
        sli_log!("[slim-0]({})({}) dc={} found", dir, die, d0);
        slim_set_delay_lane(kx, 0, d0, is_k_rx);
        let d1 = slim_calibrate_lane(cpu_slim, io_slim, 1, begin, end, is_ds, is_k_rx, win_low);
        sli_log!("[slim-1]({})({}) dc={} found", dir, die, d1);
        slim_set_delay_lane(kx, 1, d1, is_k_rx);
        let d2 = slim_calibrate_lane(cpu_slim, io_slim, 2, begin, end, is_ds, is_k_rx, win_high);
        sli_log!("[slim-2]({})({}) dc={} found", dir, die, d2);
        slim_set_delay_lane(kx, 2, d2, is_k_rx);
        let d3 = slim_calibrate_lane(cpu_slim, io_slim, 3, begin, end, is_ds, is_k_rx, win_high);
        sli_log!("[slim-3]({})({}) dc={} found", dir, die, d3);
        slim_set_delay_lane(kx, 3, d3, is_k_rx);
    }

    sli_clear(tx, CtrlI::ResetTrigger::SET.value);
    sli_clear(rx, CtrlI::ResetTrigger::SET.value);

    rx.ctrl1.modify(CtrlI::AutoSendTrnOff::CLEAR);
    sli_wait_suspend(rx);

    rx.marb_func_i.modify(MarbFunc::MarbRr::SET);
}

// ---------------------------------------------------------------------------
// Video (SLIV) calibration
// ---------------------------------------------------------------------------

fn sliv_set_rx_delay(r: &SliVRegisters, d0: i32, d1: i32, is_k_rx: bool) {
    if is_k_rx {
        r.ctrl3
            .modify(CtrlIIISliv::PadDlyRx1.val(d1 as u32) + CtrlIIISliv::PadDlyRx0.val(d0 as u32));
    } else {
        r.ctrl3
            .modify(CtrlIIISliv::PadDlyTx1.val(d1 as u32) + CtrlIIISliv::PadDlyTx0.val(d0 as u32));
    }
    // CPUD reg access needs a dummy read to make sure it's accepted
    let _ = r.ctrl3.get();
    udelay(8);
}

fn calibrate_video_delay(
    cpu_sliv: &SliVRegisters,
    io_sliv: &SliVRegisters,
    is_ds: bool,
    is_k_rx: bool,
    win: &ReadWrite<u32>,
    bit_offset: u32,
) {
    let (tx, rx) = if is_ds {
        (cpu_sliv, io_sliv)
    } else {
        (io_sliv, cpu_sliv)
    };
    let kx = if is_k_rx { rx } else { tx };
    let dir = if is_ds { "DS" } else { "US" };
    let die = if is_ds ^ is_k_rx { "CPU" } else { "IO" };

    rx.ctrl1.modify(CtrlI::AutoSendTrnOff::SET);
    tx.ctrl1.modify(CtrlI::AutoSendTrnOff::SET);

    rx.ctrl1
        .modify(CtrlI::TxMode::CLEAR + CtrlI::SlivRawMode::SET);
    tx.ctrl1
        .modify(CtrlI::SlivRawMode::SET + CtrlI::TxMode::SET);
    tx.ctrl2.modify(CtrlII::SlivTxEntSuspend.val(3));

    let mut d_first = -1i32;
    let mut d_last = -1i32;
    let mut win_size = 0i32;
    let d_def = 12i32;

    for d in SLIV_COARSE_D_BEGIN..SLIV_COARSE_D_END {
        sliv_set_rx_delay(kx, d, d, is_k_rx);

        sli_clear(
            rx,
            CtrlI::ClearBus::SET.value | CtrlI::ResetTrigger::SET.value,
        );
        sli_clear(
            tx,
            CtrlI::ClearBus::SET.value | CtrlI::ResetTrigger::SET.value,
        );

        sli_clear_intr(rx);
        udelay(CAL_DELAY_US);

        if is_suspend(rx) > 0 {
            if d_first == -1 {
                d_first = d;
            }
            d_last = d;
            sli_log!("[sliv]({})({}) dc={} found", dir, die, d);
        } else if d_last != -1 {
            let w = d_last - d_first;
            if w > win_size {
                win_size = w;
                sli_win_write(win, bit_offset, d_first, d_last);
            }
            d_first = -1;
            d_last = -1;
        }
    }

    if d_last != -1 && (d_last - d_first) > win_size {
        sli_win_write(win, bit_offset, d_first, d_last);
    } else if d_first == -1 {
        let (f, l) = sli_win_read(win, bit_offset);
        if f != 0 || l != 0 {
            d_first = f;
            d_last = l;
        } else {
            sli_log!(
                "[sliv]({})({}): no window found, using default d={}",
                dir,
                die,
                d_def
            );
            d_first = d_def;
            d_last = d_def;
        }
    }

    let mut d = (d_first + d_last) >> 1;
    if d == 0 {
        d = d_def;
    }

    sli_log!(
        "[sliv]({})({}): d_first={} d_last={} -> d={}",
        dir,
        die,
        d_first,
        d_last,
        d
    );
    sliv_set_rx_delay(kx, d, d, is_k_rx);

    sli_clear(
        rx,
        CtrlI::ClearBus::SET.value | CtrlI::ResetTrigger::SET.value,
    );
    sli_clear(
        tx,
        CtrlI::ClearBus::SET.value | CtrlI::ResetTrigger::SET.value,
    );

    udelay(CAL_DELAY_US);
    rx.ctrl1.modify(CtrlI::AutoSendTrnOff::CLEAR);
    tx.ctrl1.modify(CtrlI::AutoSendTrnOff::CLEAR);
    sli_wait_suspend(rx);
}

// ---------------------------------------------------------------------------
// MAC hotfix (applied on warm reboot)
// ---------------------------------------------------------------------------

fn mac_hotfix(io_slim: &SliMRegisters) {
    let val = io_slim.marb_status.get() & 0xe00;
    if val == 0 {
        return;
    }
    io_slim.marb_func_ii.set(val);
    io_slim.marb_func_i.modify(MarbFunc::MarbClr::SET);
}
