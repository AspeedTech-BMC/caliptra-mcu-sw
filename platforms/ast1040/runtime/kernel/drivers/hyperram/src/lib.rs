// Licensed under the Apache-2.0 license.

#![no_std]

use caliptra_mcu_config_ast1040::peri;
use caliptra_mcu_romtime::StaticRef;
use registers_peripherals::aspeed_hyperram::{
    Ctrl0, Ctrl1, EccAddr, HyperRamArbRegisters, HyperRamCsrRegisters, Status0,
};
use registers_peripherals::ast1040::aspeed_scu1::{ClkSel1, OtpCfg22, SCU1Registers};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

const HYPERRAM_MAX_SIZE: u32 = 0x0100_0000;
const HYPERRAM_ECC_REDUNDANCY: u32 = 0x0020_0000;
const HYPERRAM_ECC_EFFECTIVE_SIZE: u32 = HYPERRAM_MAX_SIZE - HYPERRAM_ECC_REDUNDANCY;
const HYPERRAM_ECC_ADDR_UNIT: u32 = 0x0000_1000;

const DPD_CFG_VAL: u32 = 0x0194_97C8;
const HS_CFG_VAL: u32 = 0x003F_1510;
const TIMING_PGSZ_VAL: u32 = 0x0A97_E1F4;
const MM_RESET_VAL: u32 = 0x0001_43E8;
const DQSTMO_VAL: u32 = 0x0000_0026;
const CR0_TIMING_VAL: u32 = 0x0000_8F5F;

const RESET_DONE_RETRIES: u32 = 1_000_000;

const CAL_TEST_START: u32 = peri::ASPEED_HYPERRAM_ADDR;
const CAL_TEST_SIZE: u32 = 0x0001_0000;
const CAL_VALUE_COUNT: usize = 64;
const CAL_MIN_WIDTH: usize = 3;
const CAL_CFG1_SEED: u8 = 0x0e;
const PHYCFG_CAL_MASK: u32 = 0xff;
// AST1040 production mcycle frequency is 200 MHz. A 200-cycle delay is at
// least 1 us there and conservatively longer on the 24 MHz FPGA build.
const CAL_SETTLE_CYCLES: u64 = 200;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HyperRamInitStatus {
    /// HYPERRAM_STATUS_0.RESET_DONE was already set, so no registers or RAM
    /// were modified.
    Skipped,
    Initialized {
        ecc_enabled: bool,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HyperRamError {
    ResetTimeout,
    CalibrationSeed,
    CalibrationCfg0,
    CalibrationCfg1,
    CalibrationCfg0Refine,
    CalibrationCfg1Refine,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct CalRange {
    start: u8,
    end: u8,
    selected: u8,
    width: u8,
}

pub struct AspeedHyperRamCtrl {
    csr: StaticRef<HyperRamCsrRegisters>,
    arb: StaticRef<HyperRamArbRegisters>,
    scu: StaticRef<SCU1Registers>,
}

impl AspeedHyperRamCtrl {
    pub const fn new(
        csr: StaticRef<HyperRamCsrRegisters>,
        arb: StaticRef<HyperRamArbRegisters>,
        scu: StaticRef<SCU1Registers>,
    ) -> Self {
        Self { csr, arb, scu }
    }

    /// Initialize HyperRAM unless the arbiter reports that reset has already
    /// completed. The skip check is deliberately first: when RESET_DONE is set,
    /// this routine must not rewrite clocks, controller state, calibration
    /// storage, or ECC-protected memory.
    pub fn init(&self) -> Result<HyperRamInitStatus, HyperRamError> {
        if self.arb.status0.is_set(Status0::RESET_DONE) {
            caliptra_mcu_romtime::println!(
                "[hyperram:init] skipped: status0.RESET_DONE is already set"
            );
            return Ok(HyperRamInitStatus::Skipped);
        }

        // programs MEMCLK_DIV=1 for the 250 MHz setting.
        self.scu.clk_sel1.modify(ClkSel1::MEMCLK_DIV.val(0x1));

        self.arb.ctrl0.modify(Ctrl0::IP_RST::SET);

        self.csr.dpd_cfg.set(DPD_CFG_VAL);
        self.csr.hs_cfg.set(HS_CFG_VAL);
        self.csr.timing_pgsz.set(TIMING_PGSZ_VAL);
        self.csr.mm_reset.set(MM_RESET_VAL);
        self.csr.dqstmo.set(DQSTMO_VAL);

        let mut retries = RESET_DONE_RETRIES;
        while !self.arb.status0.is_set(Status0::RESET_DONE) {
            retries -= 1;
            if retries == 0 {
                caliptra_mcu_romtime::println!(
                    "[hyperram:init] failed: reset timeout, status0={:#010x}",
                    self.arb.status0.get()
                );
                return Err(HyperRamError::ResetTimeout);
            }
        }
        caliptra_mcu_romtime::println!(
            "[hyperram:init] reset complete: status0={:#010x}, polls={}",
            self.arb.status0.get(),
            RESET_DONE_RETRIES - retries
        );

        self.arb.hyperram_config0.set(CR0_TIMING_VAL);
        self.csr.mr_ctrl.set(0);
        self.arb.mr_ctrl.set(0);

        self.phy_calibrate()?;

        let ecc_enabled = self.scu.otpcfg22.is_set(OtpCfg22::HYPERECC_ENABLE);
        caliptra_mcu_romtime::println!(
            "[hyperram:init] ECC requested by OTP: {}, otpcfg22={:#010x}",
            ecc_enabled,
            self.scu.otpcfg22.get()
        );
        if ecc_enabled {
            self.ecc_enable();
        }

        caliptra_mcu_romtime::println!(
            "[hyperram:init] complete: status0={:#010x}, phycfg0={:#010x}, phycfg1={:#010x}, ctrl1={:#010x}, eccaddr={:#010x}",
            self.arb.status0.get(),
            self.arb.phycfg[0].get(),
            self.arb.phycfg[1].get(),
            self.arb.ctrl1.get(),
            self.arb.eccaddr.get()
        );
        Ok(HyperRamInitStatus::Initialized { ecc_enabled })
    }

    fn delay_cal_settle() {
        let start = caliptra_mcu_romtime::mcycle();
        while caliptra_mcu_romtime::mcycle().wrapping_sub(start) < CAL_SETTLE_CYCLES {}
    }

    fn calibration_march_b_test(&self) -> bool {
        const DATA_0: u32 = 0x0000_0000;
        const DATA_1: u32 = 0xffff_ffff;
        let words = CAL_TEST_SIZE / core::mem::size_of::<u32>() as u32;
        let base = CAL_TEST_START as *mut u32;

        // SAFETY: initialization owns the scratch HyperRAM contents, and the
        // configured range lies inside its memory-mapped data window.
        unsafe {
            // M0: up(w0)
            for offset in 0..words {
                core::ptr::write_volatile(base.add(offset as usize), DATA_0);
            }

            // M1: up(r0,w1,r1,w0,r0,w1)
            for offset in 0..words {
                let addr = base.add(offset as usize);
                if core::ptr::read_volatile(addr) != DATA_0 {
                    return false;
                }
                core::ptr::write_volatile(addr, DATA_1);
                if core::ptr::read_volatile(addr) != DATA_1 {
                    return false;
                }
                core::ptr::write_volatile(addr, DATA_0);
                if core::ptr::read_volatile(addr) != DATA_0 {
                    return false;
                }
                core::ptr::write_volatile(addr, DATA_1);
            }

            // M2: up(r1,w0,w1)
            for offset in 0..words {
                let addr = base.add(offset as usize);
                if core::ptr::read_volatile(addr) != DATA_1 {
                    return false;
                }
                core::ptr::write_volatile(addr, DATA_0);
                core::ptr::write_volatile(addr, DATA_1);
            }

            // M3: down(r1,w0,w1,w0)
            for offset in (0..words).rev() {
                let addr = base.add(offset as usize);
                if core::ptr::read_volatile(addr) != DATA_1 {
                    return false;
                }
                core::ptr::write_volatile(addr, DATA_0);
                core::ptr::write_volatile(addr, DATA_1);
                core::ptr::write_volatile(addr, DATA_0);
            }

            // M4: down(r0,w1,w0)
            for offset in (0..words).rev() {
                let addr = base.add(offset as usize);
                if core::ptr::read_volatile(addr) != DATA_0 {
                    return false;
                }
                core::ptr::write_volatile(addr, DATA_1);
                core::ptr::write_volatile(addr, DATA_0);
            }
        }

        true
    }

    fn phy_set_value(&self, reg: usize, value: u8) -> bool {
        self.arb.phycfg[reg]
            .set((self.arb.phycfg[reg].get() & !PHYCFG_CAL_MASK) | u32::from(value));
        Self::delay_cal_settle();
        let readback = self.arb.phycfg[reg].get() & PHYCFG_CAL_MASK;
        if readback != u32::from(value) {
            caliptra_mcu_romtime::println!(
                "[hyperram:init] PHYCFG{} readback mismatch: wrote={:#04x}, read={:#04x}",
                reg,
                value,
                readback
            );
            return false;
        }
        true
    }

    fn find_cal_range(pass: &[bool; CAL_VALUE_COUNT], preferred: u8) -> Option<CalRange> {
        let mut best_start = 0;
        let mut best_width = 0;
        let mut best_contains_preferred = false;
        let mut i = 0;

        while i < CAL_VALUE_COUNT {
            if !pass[i] {
                i += 1;
                continue;
            }

            let start = i;
            while i < CAL_VALUE_COUNT && pass[i] {
                i += 1;
            }
            let width = i - start;
            let contains_preferred = usize::from(preferred) >= start && usize::from(preferred) < i;

            if width > best_width
                || (width == best_width && contains_preferred && !best_contains_preferred)
            {
                best_start = start;
                best_width = width;
                best_contains_preferred = contains_preferred;
            }
        }

        if best_width < CAL_MIN_WIDTH {
            return None;
        }

        Some(CalRange {
            start: best_start as u8,
            end: (best_start + best_width - 1) as u8,
            selected: (best_start + (best_width - 1) / 2) as u8,
            width: best_width as u8,
        })
    }

    fn phy_sweep(&self, reg: usize, preferred: u8) -> Option<CalRange> {
        let mut pass = [false; CAL_VALUE_COUNT];

        for (value, passed) in pass.iter_mut().enumerate() {
            *passed = self.phy_set_value(reg, value as u8) && self.calibration_march_b_test();
        }

        let pass_count = pass.iter().filter(|passed| **passed).count();
        let Some(range) = Self::find_cal_range(&pass, preferred) else {
            caliptra_mcu_romtime::println!(
                "[hyperram:init] PHYCFG{} sweep failed: passing_values={}",
                reg,
                pass_count
            );
            return None;
        };
        if !self.phy_set_value(reg, range.selected) {
            return None;
        }
        caliptra_mcu_romtime::println!(
            "[hyperram:init] PHYCFG{} sweep complete: pass={}/{}, range={:#04x}-{:#04x}, width={}, selected={:#04x}",
            reg,
            pass_count,
            CAL_VALUE_COUNT,
            range.start,
            range.end,
            range.width,
            range.selected
        );
        Some(range)
    }

    fn phy_calibrate(&self) -> Result<(), HyperRamError> {
        let original_cfg0 = self.arb.phycfg[0].get();
        let original_cfg1 = self.arb.phycfg[1].get();
        let preferred_cfg0 = (original_cfg0 & PHYCFG_CAL_MASK) as u8;
        let mut preferred_cfg1 = (original_cfg1 & PHYCFG_CAL_MASK) as u8;
        caliptra_mcu_romtime::println!(
            "[hyperram:init] PHY initial values: cfg0={:#04x}, cfg1={:#04x}, scratch={:#010x}+{:#x}",
            preferred_cfg0,
            preferred_cfg1,
            CAL_TEST_START,
            CAL_TEST_SIZE
        );

        if !self.calibration_march_b_test() {
            caliptra_mcu_romtime::println!(
                "[hyperram:init] initial March B failed; applying cfg1 seed={:#04x}",
                CAL_CFG1_SEED
            );
            preferred_cfg1 = CAL_CFG1_SEED;
            if !self.phy_set_value(1, preferred_cfg1) {
                return Err(HyperRamError::CalibrationSeed);
            }
        }

        let cfg0 = self
            .phy_sweep(0, preferred_cfg0)
            .ok_or(HyperRamError::CalibrationCfg0)?;
        let cfg1 = self
            .phy_sweep(1, preferred_cfg1)
            .ok_or(HyperRamError::CalibrationCfg1)?;
        let cfg0_refined = self
            .phy_sweep(0, cfg0.selected)
            .ok_or(HyperRamError::CalibrationCfg0Refine)?;

        if cfg0_refined.selected != cfg0.selected {
            caliptra_mcu_romtime::println!(
                "[hyperram:init] cfg0 selection moved {:#04x}->{:#04x}; refining cfg1",
                cfg0.selected,
                cfg0_refined.selected
            );
            self.phy_sweep(1, cfg1.selected)
                .ok_or(HyperRamError::CalibrationCfg1Refine)?;
        }

        Ok(())
    }

    /// Configure HYPERARB_ECCADDR.
    ///
    /// Bits `[11:0]` hold the ECC start address and bits `[27:16]` hold the
    /// ECC length. Both values are encoded in 4 KiB units.
    fn ecc_config(&self, start_addr: u32, length: u32) {
        self.arb.eccaddr.write(
            EccAddr::START.val(start_addr / HYPERRAM_ECC_ADDR_UNIT)
                + EccAddr::LEN.val(length / HYPERRAM_ECC_ADDR_UNIT),
        );
    }

    fn ecc_clear(&self) {
        let start_addr = (self.arb.eccaddr.read(EccAddr::START) * HYPERRAM_ECC_ADDR_UNIT)
            .wrapping_add(peri::ASPEED_HYPERRAM_ADDR);
        let length = self.arb.eccaddr.read(EccAddr::LEN) * HYPERRAM_ECC_ADDR_UNIT;
        let mut addr = start_addr as *mut u32;
        caliptra_mcu_romtime::println!(
            "[hyperram:init] ECC scrub start: start={:#010x}, len={:#010x}, end={:#010x}",
            start_addr,
            length,
            start_addr.wrapping_add(length)
        );

        // SAFETY: ECC configuration limits the decoded range to HyperRAM.
        unsafe {
            let end = addr.add((length / core::mem::size_of::<u32>() as u32) as usize);
            while addr < end {
                core::ptr::write_volatile(addr, 0);
                addr = addr.add(1);
            }
        }
    }

    fn ecc_enable(&self) {
        self.ecc_config(0, HYPERRAM_ECC_EFFECTIVE_SIZE);
        self.ecc_clear();
        self.arb
            .ctrl1
            .modify(Ctrl1::ECC_ENC_EN::SET + Ctrl1::ECC_DEC_EN::SET);
    }
}
