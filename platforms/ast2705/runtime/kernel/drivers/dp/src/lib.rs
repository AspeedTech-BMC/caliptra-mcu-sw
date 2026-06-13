// Licensed under the Apache-2.0 license

#![no_std]

use caliptra_mcu_capsules_runtime::bootdev::hil::{KernelImageClient, KernelImageLoader};
use core::ptr::copy_nonoverlapping;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use registers_peripherals::ast2705::aspeed_dpmcu::{
    DpMcuRegisters, DpRegisters, MCU_DMEM_BASE, MCU_IMEM_BASE,
};
use registers_peripherals::ast2705::aspeed_scu0::{SCU0Registers, VgaFuncCtrl, VgaScratch};

/// Caliptra DPMCU firmware image identifier.
pub const DP_FW: u32 = 0x1006;

pub struct AspeedDpCtrl {
    mcu: StaticRef<DpMcuRegisters>,
    dp: StaticRef<DpRegisters>,
    scu0: StaticRef<SCU0Registers>,
    bootdev: OptionalCell<&'static dyn KernelImageLoader>,
}

impl AspeedDpCtrl {
    pub const fn new(
        mcu: StaticRef<DpMcuRegisters>,
        dp: StaticRef<DpRegisters>,
        scu0: StaticRef<SCU0Registers>,
    ) -> Self {
        Self {
            mcu,
            dp,
            scu0,
            bootdev: OptionalCell::empty(),
        }
    }

    /// Initialize the DisplayPort subsystem.
    ///
    /// If the DPMCU was not already running, this resets the DP and DPMCU
    /// hardware, then asynchronously loads the firmware via `bootdev`.
    /// `image_ready` will be called back when loading completes.
    pub fn init(&'static self, bootdev: &'static dyn KernelImageLoader) -> Result<(), ErrorCode> {
        self.bootdev.set(bootdev);

        // Check DpMcuFwExecuting in the scratch register for the active VGA port.
        let vga1 = self
            .scu0
            .vga_func_ctrl
            .matches_all(VgaFuncCtrl::DpSrc::Vga1);
        let is_mcu_stop = if vga1 {
            !self.scu0.vga1_scratch1.is_set(VgaScratch::DpMcuFwExecuting)
        } else {
            !self.scu0.vga0_scratch1.is_set(VgaScratch::DpMcuFwExecuting)
        };

        if is_mcu_stop {
            // Assert reset for DPTX and DPMCU, then re-enable the DP clock.
            use registers_peripherals::ast2705::aspeed_scu0::{ClkGate1, ModRst1};
            self.scu0
                .modrst1_ctrl
                .modify(ModRst1::Dp::SET + ModRst1::DpMcu::SET);
            udelay(100);
            self.scu0.clock_stop_clr.modify(ClkGate1::Dp::SET);
            udelay(10);
            self.scu0
                .modrst1_clr
                .modify(ModRst1::Dp::SET + ModRst1::DpMcu::SET);
            udelay(1);
        }

        // Verify the DP controller is accessible.
        if self.dp.version.get() == 0 {
            caliptra_mcu_romtime::println!("[dp] Can't access DP controller (version == 0)");
            return Err(ErrorCode::FAIL);
        }

        // Clear display master select and EDID passthrough bits.
        use registers_peripherals::ast2705::aspeed_dpmcu::DpHandshake;
        self.dp
            .handshake
            .modify(DpHandshake::HostReadEdid::CLEAR + DpHandshake::VideoFmtSrc::CLEAR);

        // Clear the DPMCU display-format / enable region in DMEM.
        unsafe {
            core::ptr::write_volatile((MCU_DMEM_BASE + 0x0de0) as *mut u32, 0);
        }

        if is_mcu_stop {
            use registers_peripherals::ast2705::aspeed_dpmcu::McuCtrl;

            // Shut down both memories and hold the AHB slave in reset.
            self.mcu.ctrl.write(
                McuCtrl::Config::SET
                    + McuCtrl::ImemClkOff::SET
                    + McuCtrl::ImemShutDown::SET
                    + McuCtrl::DmemClkOff::SET
                    + McuCtrl::DmemShutDown::SET
                    + McuCtrl::AhbsSwRst::SET,
            );
            // Power up memories.
            self.mcu
                .ctrl
                .modify(McuCtrl::ImemShutDown::CLEAR + McuCtrl::DmemShutDown::CLEAR);
            // Enable clocks.
            self.mcu
                .ctrl
                .modify(McuCtrl::ImemClkOff::CLEAR + McuCtrl::DmemClkOff::CLEAR);
            // Allow AHB slave to write into IMEM so firmware can be loaded.
            self.mcu.ctrl.modify(McuCtrl::AhbsImemEn::SET);

            // Request the DPMCU firmware image; image_ready() will finish boot.
            bootdev.request_image(DP_FW, self)?;
        } else {
            caliptra_mcu_romtime::println!("[dp] already running");
            // DPMCU was already running — just mark both scratch registers
            self.update_vga_scratch();
        }

        Ok(())
    }

    /// Set TxType = DpmcuDp in both VGA scratch registers.
    fn update_vga_scratch(&self) {
        self.scu0.vga0_scratch1.modify(VgaScratch::TxType::DpmcuDp);
        self.scu0.vga1_scratch1.modify(VgaScratch::TxType::DpmcuDp);
    }
}

/// MCU cycle counter frequency in Hz (matches TIMER_FREQUENCY_HZ in the runtime).
const TIMER_FREQUENCY_HZ: u64 = 400_000_000;

fn udelay(us: u32) {
    let start = caliptra_mcu_romtime::mcycle();
    let ticks = TIMER_FREQUENCY_HZ / 1_000_000 * us as u64;
    while caliptra_mcu_romtime::mcycle().wrapping_sub(start) < ticks {}
}

impl KernelImageClient for AspeedDpCtrl {
    fn image_ready(&self, image_id: u32, addr: u64, size: u32) {
        if image_id != DP_FW {
            return;
        }

        // Copy firmware from the loader buffer into DPMCU IMEM.
        unsafe {
            let src = addr as *const u32;
            let dst = MCU_IMEM_BASE as *mut u32;
            copy_nonoverlapping(src, dst, (size as usize + 3) / 4);
        }

        use registers_peripherals::ast2705::aspeed_dpmcu::{McuCtrl, McuIntrCtrl};

        // Disable AHB slave IMEM access now that firmware is in place.
        self.mcu.ctrl.modify(McuCtrl::AhbsImemEn::CLEAR);

        // Release the DPMCU core and AHB master from reset to start execution.
        self.mcu
            .ctrl
            .modify(McuCtrl::CoreSwRst::SET + McuCtrl::AhbmSwRst::SET);

        // Mask all DPMCU interrupt sources.
        self.mcu.intr_ctrl.write(McuIntrCtrl::En.val(0xff));

        // Update VGA scratch registers to reflect DPMCU is now handling display.
        self.update_vga_scratch();

        if let Some(b) = self.bootdev.get() {
            b.notify_init_done(DP_FW, true);
        }
    }
}
