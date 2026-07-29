// Licensed under the Apache-2.0 license
//
// Rust port of usb_init() from
// zephyr-mcu-runtime/aspeed-zephyr-project/apps/mcu-runtime/soc/ast2700/usb.c
//
// `usb_uart_init()` (USB2-UART-over-vHub) is gated in the reference behind
// `CONFIG_USB_UART_PORTS`, a Zephyr Kconfig with no equivalent configuration
// plumbing in this codebase, so it is omitted entirely here — equivalent to
// that Kconfig always being unset upstream.

#![no_std]

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use registers_peripherals::ast2705::aspeed_scu0::{
    ChipId1, ClkGate1, ModRst2, SCU0Registers, UsbFuncCtrl,
};
use registers_peripherals::ast2705::aspeed_scu1::{OTPStrap, SCU1Registers};
use registers_peripherals::ast2705::aspeed_usb::{
    BEhciRegisters, BehciFrmTiming, Phy2CtlSts2, Phy2CtlSts3, Phy3S00, UsbPhy2Registers,
    UsbPhy3Registers,
};

/// PHY PCS Protocol Setting defaults per SNPS documents.
const PHY3P00_DEFAULT: u32 = 0xCE70_000F;
const PHY3P04_DEFAULT: u32 = 0x49C0_0014;
const PHY3P08_DEFAULT: u32 = 0x5E40_6825;
const PHY3P0C_DEFAULT: u32 = 0x0000_0001;

const BURST_128: u32 = 0x0000_0006;
const BURST_256: u32 = 0x0000_000E;

/// Mirrors the Zephyr reference's `static bool phy_ext_load_quirk = false;`,
/// which nothing upstream ever sets true either; kept for parity.
const PHY_EXT_LOAD_QUIRK: bool = false;

const PHY3_INIT_TIMEOUT_ITERS: u32 = 100;

/// Frame Timing Adjustment workaround: preEOF1 = 0x100, preEOF2 = preEOF1 + MPS.
const BEHCI_PRE_EOF1_VAL: u32 = 0x100;
const BEHCI_WORKAROUND_MPS: u32 = 0x40;

const TIMER_FREQUENCY_HZ: u64 = 400_000_000;

fn udelay(us: u32) {
    let start = caliptra_mcu_romtime::mcycle();
    let ticks = TIMER_FREQUENCY_HZ / 1_000_000 * us as u64;
    while caliptra_mcu_romtime::mcycle().wrapping_sub(start) < ticks {}
}

fn mdelay(ms: u32) {
    udelay(ms * 1_000);
}

/// Selects which of PortA/PortB is being addressed.
#[derive(Clone, Copy, PartialEq, Eq)]
enum UsbPort {
    A,
    B,
}

impl UsbPort {
    fn label(self) -> &'static str {
        match self {
            UsbPort::A => "A",
            UsbPort::B => "B",
        }
    }
}

/// USB routing function per port, mirroring `enum scu0_usb_function`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UsbFunction {
    NotUsed,
    EhciVhubAndXhciPhy,
    XhciPhy,
    XhciVhub,
    EhciVhub,
    VhubPhy,
    EhciPhy,
}

impl UsbFunction {
    fn has_ehci(self) -> bool {
        matches!(
            self,
            UsbFunction::EhciVhubAndXhciPhy | UsbFunction::EhciVhub | UsbFunction::EhciPhy
        )
    }
}

/// Per-port config, mirroring the devicetree `usb-function` /
/// `enable-large-burst` properties the Zephyr reference reads via
/// `DT_ENUM_IDX_OR`/`DT_PROP` (this codebase has no devicetree, so these are
/// supplied directly by board init instead).
#[derive(Clone, Copy)]
pub struct AspeedUsbConfig {
    pub port_a_func: UsbFunction,
    pub port_b_func: UsbFunction,
    pub port_a_large_burst: bool,
    pub port_b_large_burst: bool,
}

/// One port's PHY3 (xHCI)/PHY2 (vHub1)/EHCI register blocks.
struct UsbPortRegs {
    phy3: StaticRef<UsbPhy3Registers>,
    phy2: StaticRef<UsbPhy2Registers>,
    ehci: StaticRef<BEhciRegisters>,
}

pub struct AspeedUsbCtrl {
    scu0: StaticRef<SCU0Registers>,
    scu1: StaticRef<SCU1Registers>,
    port_a: UsbPortRegs,
    port_b: UsbPortRegs,
    config: AspeedUsbConfig,
}

impl AspeedUsbCtrl {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        scu0: StaticRef<SCU0Registers>,
        scu1: StaticRef<SCU1Registers>,
        port_a_phy3: StaticRef<UsbPhy3Registers>,
        port_a_phy2: StaticRef<UsbPhy2Registers>,
        port_a_ehci: StaticRef<BEhciRegisters>,
        port_b_phy3: StaticRef<UsbPhy3Registers>,
        port_b_phy2: StaticRef<UsbPhy2Registers>,
        port_b_ehci: StaticRef<BEhciRegisters>,
        config: AspeedUsbConfig,
    ) -> Self {
        Self {
            scu0,
            scu1,
            port_a: UsbPortRegs {
                phy3: port_a_phy3,
                phy2: port_a_phy2,
                ehci: port_a_ehci,
            },
            port_b: UsbPortRegs {
                phy3: port_b_phy3,
                phy2: port_b_phy2,
                ehci: port_b_ehci,
            },
            config,
        }
    }

    fn port_regs(&self, port: UsbPort) -> &UsbPortRegs {
        match port {
            UsbPort::A => &self.port_a,
            UsbPort::B => &self.port_b,
        }
    }

    /// Initializes USB PHY3 and the xHCI controller for `port`
    /// (`usb_usb3_init`).
    fn usb3_init(&self, port: UsbPort, large_burst: bool) -> Result<(), ErrorCode> {
        let regs = self.port_regs(port);

        let (clk, rst) = match port {
            UsbPort::A => (
                ClkGate1::UsbA::SET,
                ModRst2::UsbaPhy3::SET + ModRst2::UsbaXhci::SET,
            ),
            UsbPort::B => (
                ClkGate1::UsbB::SET,
                ModRst2::UsbbPhy3::SET + ModRst2::UsbbXhci::SET,
            ),
        };

        // Clk/Reset for USB PHY3 and xHCI controllers.
        self.scu0.clock_stop_clr.modify(clk);
        mdelay(10);
        self.scu0.modrst2_clr.modify(rst);
        udelay(1);

        let mut timeout = PHY3_INIT_TIMEOUT_ITERS;
        while !regs.phy3.s00.is_set(Phy3S00::InitDone) {
            udelay(100);
            timeout -= 1;
            if timeout == 0 {
                caliptra_mcu_romtime::println!("[usb] Wait phy3 init timed out");
                return Err(ErrorCode::FAIL);
            }
        }

        if PHY_EXT_LOAD_QUIRK {
            regs.phy3.s00.modify(Phy3S00::SramExtLoad::SET);
        } else {
            regs.phy3.s00.modify(Phy3S00::SramBypass::SET);
        }

        // Default protocol1_ext PCS settings per SNPS documents.
        regs.phy3.p00.set(PHY3P00_DEFAULT);
        regs.phy3.p04.set(PHY3P04_DEFAULT);
        regs.phy3.p08.set(PHY3P08_DEFAULT);
        regs.phy3.p0c.set(PHY3P0C_DEFAULT);

        // xHCI bus burst size per PCIe max payload size (512/256 or 128).
        let burst = if large_burst { BURST_256 } else { BURST_128 };
        // (offset, value) pairs mirroring `usb_dwc3_ctrl ctrl_data[]`.
        let dwc_cmds: [(u32, u32); 3] = [
            (0xc100, burst),       // GSBUSCFG0: Bus Burst Type
            (0xc12c, 0x0c85_4802), // GUCTL: ref_clk
            (0xc630, 0x0c80_0020), // GLADJ: ref_clk
        ];
        write_dwc_commands(&regs.phy3.dwc, &dwc_cmds);

        caliptra_mcu_romtime::println!(
            "[usb] Initialized Port{} USB3{}",
            port.label(),
            if large_burst { " (Large Burst)" } else { "" }
        );
        Ok(())
    }

    /// Initializes USB2 EHCI and vHub (PHY2) for `port` (`usb_usb2_init`).
    fn usb2_init(&self, port: UsbPort, func: UsbFunction) {
        let regs = self.port_regs(port);
        let has_ehci = func.has_ehci();

        let (clk, mut rst) = match port {
            UsbPort::A => (ClkGate1::UsbA::SET, ModRst2::UsbaVhub1::SET),
            UsbPort::B => (ClkGate1::UsbB::SET, ModRst2::UsbbVhub1::SET),
        };
        if has_ehci {
            rst = rst
                + match port {
                    UsbPort::A => ModRst2::UsbaEhciVhub0::SET,
                    UsbPort::B => ModRst2::UsbbEhciVhub0::SET,
                };
        }

        // Clk/Reset for EHCI/vHub1 (Port 2.0 PHY).
        self.scu0.clock_stop_clr.modify(clk);
        mdelay(10);
        self.scu0.modrst2_clr.modify(rst);
        udelay(1);

        // xHCI-to-vHub1 clock rate: 60 MHz.
        regs.phy2.ctl_sts_2.modify(Phy2CtlSts2::XhciVhubClk.val(3));
        // Pre-emphasis current setting.
        regs.phy2.ctl_sts_3.modify(Phy2CtlSts3::PreEmphasis.val(2));

        // EOF1/EOF2 timing workaround for the DMA termination bug.
        if has_ehci {
            regs.ehci.frm_timing.write(
                BehciFrmTiming::PreEof1.val(BEHCI_PRE_EOF1_VAL)
                    + BehciFrmTiming::PreEof2.val(BEHCI_PRE_EOF1_VAL + BEHCI_WORKAROUND_MPS),
            );
        }

        caliptra_mcu_romtime::println!("[usb] Initialized Port{} USB2", port.label());
    }

    /// Switches PortA/PortB EHCI/vHub/xHCI routing (`usb_func_init`).
    fn func_init(&self, port: UsbPort, func: UsbFunction) {
        match port {
            UsbPort::A => self.func_init_a(func),
            UsbPort::B => self.func_init_b(func),
        }
    }

    fn func_init_a(&self, func: UsbFunction) {
        use UsbFunction::*;
        match func {
            EhciVhub => {
                self.scu0
                    .usb_func_ctrl
                    .modify(UsbFuncCtrl::VhubEhciA.val(0));
            }
            EhciVhubAndXhciPhy | XhciPhy => {
                self.scu0.usb_func_ctrl.modify(
                    UsbFuncCtrl::VhubEhciA.val(0)
                        + UsbFuncCtrl::U3XhciA.val(2)
                        + UsbFuncCtrl::U2XhciA.val(2),
                );
            }
            VhubPhy => {
                self.scu0
                    .usb_func_ctrl
                    .modify(UsbFuncCtrl::U2XhciA.val(1) + UsbFuncCtrl::VhubEhciA.val(0));
            }
            EhciPhy => {
                self.scu0
                    .usb_func_ctrl
                    .modify(UsbFuncCtrl::VhubEhciA.val(3));
            }
            XhciVhub => {
                self.scu0.usb_func_ctrl.modify(
                    UsbFuncCtrl::U3XhciA.val(2)
                        + UsbFuncCtrl::U2XhciA.val(0)
                        + UsbFuncCtrl::VhubEhciA.val(0),
                );
            }
            NotUsed => {}
        }
    }

    fn func_init_b(&self, func: UsbFunction) {
        use UsbFunction::*;
        match func {
            EhciVhub => {
                self.scu0
                    .usb_func_ctrl
                    .modify(UsbFuncCtrl::VhubEhciB.val(0));
            }
            EhciVhubAndXhciPhy | XhciPhy => {
                self.scu0.usb_func_ctrl.modify(
                    UsbFuncCtrl::VhubEhciB.val(0)
                        + UsbFuncCtrl::U3XhciB.val(2)
                        + UsbFuncCtrl::U2XhciB.val(2),
                );
            }
            VhubPhy => {
                self.scu0
                    .usb_func_ctrl
                    .modify(UsbFuncCtrl::U2XhciB.val(1) + UsbFuncCtrl::VhubEhciB.val(0));
            }
            EhciPhy => {
                self.scu0
                    .usb_func_ctrl
                    .modify(UsbFuncCtrl::VhubEhciB.val(3));
            }
            XhciVhub => {
                self.scu0.usb_func_ctrl.modify(
                    UsbFuncCtrl::U3XhciB.val(2)
                        + UsbFuncCtrl::U2XhciB.val(0)
                        + UsbFuncCtrl::VhubEhciB.val(0),
                );
            }
            NotUsed => {}
        }
    }

    /// Brings up `port` according to `func` (`usb_port_init`): xHCI-capable
    /// functions initialize PHY3/xHCI first, then always fall through to
    /// the shared USB2 EHCI/vHub1 (PHY2) init.
    fn port_init(
        &self,
        port: UsbPort,
        func: UsbFunction,
        large_burst: bool,
    ) -> Result<(), ErrorCode> {
        use UsbFunction::*;
        match func {
            EhciVhubAndXhciPhy | XhciPhy | XhciVhub => {
                self.usb3_init(port, large_burst)?;
                self.usb2_init(port, func);
            }
            EhciVhub | VhubPhy | EhciPhy => {
                self.usb2_init(port, func);
            }
            NotUsed => {}
        }
        Ok(())
    }

    /// Initializes both USB ports (`usb_init`).
    pub fn init(&'static self) -> Result<(), ErrorCode> {
        // A0 silicon: nothing to do.
        if self.scu0.chip_id1.read(ChipId1::HwRev) == 0 {
            caliptra_mcu_romtime::println!("[usb] Do nothing in A0.");
            return Ok(());
        }

        let mut port_a_func = self.config.port_a_func;
        let mut port_b_func = self.config.port_b_func;

        // USB recovery boot mode overrides the recovery port to vHub-over-PHY.
        if self
            .scu1
            .strap
            .matches_all(OTPStrap::RecoveryInterface::Usb)
        {
            let port = self.scu1.strap.read(OTPStrap::RecoveryUsbPort);
            caliptra_mcu_romtime::println!("[usb] USB Recovery Mode: Port = {}", port);
            if port == 0 {
                port_a_func = UsbFunction::VhubPhy;
            } else if port == 1 {
                port_b_func = UsbFunction::VhubPhy;
            }
        }

        // Switch PortA and PortB USB function.
        self.func_init(UsbPort::A, port_a_func);
        self.func_init(UsbPort::B, port_b_func);
        caliptra_mcu_romtime::println!(
            "[usb] A = {:?}, B = {:?} (0x{:08X})",
            port_a_func,
            port_b_func,
            self.scu0.usb_func_ctrl.get()
        );

        self.port_init(UsbPort::A, port_a_func, self.config.port_a_large_burst)?;
        self.port_init(UsbPort::B, port_b_func, self.config.port_b_large_burst)?;

        Ok(())
    }
}

/// Packs 3 (offset, value) commands into the 5-word DWC3 command block,
/// mirroring `usb_usb3_init`'s 48-bit-command packing loop: each command is
/// a 32-bit value + 16-bit offset, and consecutive commands share the
/// unused 16 bits of intermediate words rather than padding to a full word
/// each.
fn write_dwc_commands(
    dwc: &[kernel::utilities::registers::ReadWrite<u32>],
    entries: &[(u32, u32); 3],
) {
    let mut idx = 0usize;
    for (i, &(offset, value)) in entries.iter().enumerate() {
        if i % 2 == 0 {
            dwc[idx].set(value);
            idx += 1;
            dwc[idx].set(offset & 0xFFFF);
        } else {
            let mut val = dwc[idx].get() & 0xFFFF;
            val |= (value & 0xFFFF) << 16;
            dwc[idx].set(val);
            idx += 1;

            let val2 = (offset << 16) | (value >> 16);
            dwc[idx].set(val2);
            idx += 1;
        }
    }
}
