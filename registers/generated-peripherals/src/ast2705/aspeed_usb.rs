// Licensed under the Apache-2.0 license
//
// Register layout ported from
// zephyr-mcu-runtime/aspeed-zephyr-project/apps/mcu-runtime/soc/ast2700/include/usb.h

use tock_registers::registers::ReadWrite;
use tock_registers::{self, register_bitfields, register_structs};

/// USB PortA PHY3/xHCI base (BootMCU view; CA35 view: 0x1201_0000)
pub const USB0_PHY3_BASE: u32 = 0x3201_0000;
/// USB PortA vHub1/PHY2 base (BootMCU view; CA35 view: 0x1201_1000)
pub const USB0_PHY2_BASE: u32 = 0x3201_1000;
/// USB PortA EHCI (BEHCI) base (BootMCU view; CA35 view: 0x1206_1800)
pub const USB0_EHCI_BASE: u32 = 0x3206_1800;
/// USB PortB PHY3/xHCI base (BootMCU view; CA35 view: 0x1202_0000)
pub const USB1_PHY3_BASE: u32 = 0x3202_0000;
/// USB PortB vHub1/PHY2 base (BootMCU view; CA35 view: 0x1202_1000)
pub const USB1_PHY2_BASE: u32 = 0x3202_1000;
/// USB PortB EHCI (BEHCI) base (BootMCU view; CA35 view: 0x1206_3800)
pub const USB1_EHCI_BASE: u32 = 0x3206_3800;

register_structs! {
    /// USB3.1 PHY control/status + DWC3 command block
    /// (`ASPEED_UPHY3{A,B}_BASE`).
    pub UsbPhy3Registers {
        /// PHY SRAM control/status #1
        (0x00 => pub s00: ReadWrite<u32, Phy3S00::Register>),
        /// PHY SRAM control/status #2
        (0x04 => pub s04: ReadWrite<u32>),
        /// PHY PCS control/status #1
        (0x08 => pub c00: ReadWrite<u32>),
        /// PHY PCS control/status #2
        (0x0C => pub c04: ReadWrite<u32>),
        /// PHY PCS protocol setting #1
        (0x10 => pub p00: ReadWrite<u32>),
        /// PHY PCS protocol setting #2
        (0x14 => pub p04: ReadWrite<u32>),
        /// PHY PCS protocol setting #3
        (0x18 => pub p08: ReadWrite<u32>),
        /// PHY PCS protocol setting #4
        (0x1C => pub p0c: ReadWrite<u32>),
        (0x20 => _reserved20),
        /// DWC3 command block: 5 words backing 3 packed 48-bit
        /// {address,data} commands (see `usb_usb3_init`'s bit-packing loop).
        (0x40 => pub dwc: [ReadWrite<u32>; 5]),
        (0x54 => @END),
    }
}

register_structs! {
    /// USB2 PHY2 (inside vHub1) control/status block, at the vHub1 base
    /// (`ASPEED_VHUBA1_BASE`/`ASPEED_VHUBB1_BASE`).
    pub UsbPhy2Registers {
        (0x000 => _reserved000),
        /// PHY control/status #1
        (0x800 => pub ctl_sts_1: ReadWrite<u32>),
        /// PHY control/status #2 — xHCI-to-vHub1 clock rate select
        (0x804 => pub ctl_sts_2: ReadWrite<u32, Phy2CtlSts2::Register>),
        /// PHY control/status #3 — pre-emphasis current setting
        (0x808 => pub ctl_sts_3: ReadWrite<u32, Phy2CtlSts3::Register>),
        /// PHY control/status #4
        (0x80C => pub ctl_sts_4: ReadWrite<u32>),
        (0x810 => @END),
    }
}

register_structs! {
    /// BEHCI (USB2 EHCI) register block (`ASPEED_BEHCI{0,1}_BASE`).
    pub BEhciRegisters {
        (0x00 => _reserved00),
        /// Frame Timing Adjustment (EOF1/EOF2), workaround for the DMA
        /// termination bug.
        (0x88 => pub frm_timing: ReadWrite<u32, BehciFrmTiming::Register>),
        (0x8C => @END),
    }
}

register_bitfields![u32,
/// PHY3 SRAM control/status #1 (`ASPEED_USB_PHY3_S00`)
pub Phy3S00 [
    /// USB3.1 PHY SRAM external load done (USB_PHY3_SRAM_EXT_LOAD)
    SramExtLoad OFFSET(6)  NUMBITS(1) [],
    /// USB3.1 PHY SRAM bypass (USB_PHY3_SRAM_BYPASS)
    SramBypass  OFFSET(7)  NUMBITS(1) [],
    /// USB3.1 PHY internal SRAM initialization done (USB_PHY3_INIT_DONE)
    InitDone    OFFSET(15) NUMBITS(1) [],
],
/// PHY2 control/status #2
pub Phy2CtlSts2 [
    /// xHCI-to-vHub1 clock rate select: 3 = 60 MHz
    XhciVhubClk OFFSET(26) NUMBITS(2) [],
],
/// PHY2 control/status #3
pub Phy2CtlSts3 [
    /// Pre-emphasis current setting
    PreEmphasis OFFSET(21) NUMBITS(2) [],
],
/// BEHCI Frame Timing Adjustment register
pub BehciFrmTiming [
    /// preEOF1 timing
    PreEof1 OFFSET(12) NUMBITS(10) [],
    /// preEOF2 timing
    PreEof2 OFFSET(22) NUMBITS(10) [],
],
];
