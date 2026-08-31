// Licensed under the Apache-2.0 license

use tock_registers::registers::ReadWrite;
use tock_registers::{self, register_bitfields, register_structs};

register_structs! {
    /// wdt
    pub WdtRegisters {
        /// WDTn Counter Status Register
        (0x000 => pub cntsts: ReadWrite<u32>),
        /// WDTn Counter Reload Value Register
        (0x004 => pub relval: ReadWrite<u32>),
        /// WDTn Counter Restart Register
        (0x008 => pub cntrestart: ReadWrite<u32, CntRestart::Register>),
        /// WDTn Control Register
        (0x00C => pub ctrl: ReadWrite<u32, Ctrl::Register>),
        /// WDTn Timeout Status Register
        (0x010 => pub timeoutsts: ReadWrite<u32, TimeoutSts::Register>),
        /// WDTn Clear Timeout Status Register
        (0x014 => pub timeoutclr: ReadWrite<u32, TimeoutClr::Register>),
        /// WDTn Reset Width Register
        (0x018 => pub rstwidth: ReadWrite<u32, RstWidth::Register>),
        /// WDTn Reset Mask Register \#1
        (0x01C => pub rstmask1: ReadWrite<u32, RstMask1::Register>),
        /// WDTn Reset Mask Register \#2
        (0x020 => pub rstmask2: ReadWrite<u32, RstMask2::Register>),
        /// WDTn Reset Mask Register \#3
        (0x024 => pub rstmask3: ReadWrite<u32, RstMask3::Register>),
        /// WDTn Reset Mask Register \#4
        (0x028 => pub rstmask4: ReadWrite<u32, RstMask4::Register>),
        /// WDTn Reset Mask Register \#5
        (0x02C => pub rstmask5: ReadWrite<u32, RstMask5::Register>),
        /// WDTn Software Mode Reset Control Register
        (0x030 => pub swrstctrl: ReadWrite<u32, SwRstCtrl::Register>),
        /// WDTn Software Mode Reset Mask Register \#1)
        (0x034 => pub swrstmask1: ReadWrite<u32, SwRstMask1::Register>),
        /// WDTn Software Mode Reset Mask Register \#2
        (0x038 => pub swrstmask2: ReadWrite<u32, SwRstMask2::Register>),
        /// WDTn Software Mode Reset Mask Register \#3
        (0x03C => pub swrstmask3: ReadWrite<u32, SwRstMask3::Register>),
        /// WDTn Software Mode Reset Mask Register \#4
        (0x040 => pub swrstmask4: ReadWrite<u32, SwRstMask4::Register>),
        /// WDTn Software Mode Reset Mask Register \#5
        (0x044 => pub swrstmask5: ReadWrite<u32, SwRstMask5::Register>),
        /// WDTn Funtion Disable Control Register
        (0x048 => pub disctrl: ReadWrite<u32, DisCtrl::Register>),
        /// WDTn Scratch Register
        (0x04C => pub scratchreg: ReadWrite<u32, ScratchReg::Register>),
        /// WDTn Reset Mask Write Protection Register \#1
        (0x050 => pub rstmaskwriteprot1: ReadWrite<u32, RstMaskWriteProt1::Register>),
        /// WDTn Reset Mask Write Protection Register \#2
        (0x054 => pub rstmaskwriteprot2: ReadWrite<u32, RstMaskWriteProt2::Register>),
        /// WDTn Reset Mask Write Protection Register \#3
        (0x058 => pub rstmaskwriteprot3: ReadWrite<u32, RstMaskWriteProt3::Register>),
        /// WDTn Reset Mask Write Protection Register \#4
        (0x05C => pub rstmaskwriteprot4: ReadWrite<u32, RstMaskWriteProt4::Register>),
        /// WDTn Reset Mask Write Protection Register \#5
        (0x060 => pub rstmaskwriteprot5: ReadWrite<u32, RstMaskWriteProt5::Register>),
        /// WDTn Software Mode Reset Mask Write Protection Register \#1
        (0x064 => pub swrstmaskwriteprot1: ReadWrite<u32, SwRstMaskWriteProt1::Register>),
        /// WDTn Software Mode Reset Mask Write Protection Register \#2
        (0x068 => pub swrstmaskwriteprot2: ReadWrite<u32, SwRstMaskWriteProt2::Register>),
        /// WDTn Software Mode Reset Mask Write Protection Register \#3
        (0x06C => pub swrstmaskwriteprot3: ReadWrite<u32, SwRstMaskWriteProt3::Register>),
        /// WDTn Software Mode Reset Mask Write Protection Register \#4
        (0x070 => pub swrstmaskwriteprot4: ReadWrite<u32, SwRstMaskWriteProt4::Register>),
        /// WDTn Software Mode Reset Mask Write Protection Register \#5
        (0x074 => pub swrstmaskwriteprot5: ReadWrite<u32, SwRstMaskWriteProt5::Register>),
        (0x078 => _reserved0),
        /// WDTn Write Protection Register
        (0x07C => pub writeprot: ReadWrite<u32, WriteProt::Register>),
        (0x080 => @END),
    }
}

register_bitfields![u32,
CntSts [
    /// Counter status
    CounterSts OFFSET(0) NUMBITS(32) []
],
pub RelVal [
    /// Counter reload value register
    CounterReloadValueReg OFFSET(0) NUMBITS(32) []
],
pub CntRestart [
    /// Reserved (0)
    Reserved0 OFFSET(16) NUMBITS(16) [],
    /// Restart register
    RestartReg OFFSET(0) NUMBITS(16) []
],
pub Ctrl [
    /// Pre-timeout value for interrupt generation
    PretimeoutValueForINTGeneration OFFSET(10) NUMBITS(22) [],
    /// Reserved (0)
    Reserved0 OFFSET(7) NUMBITS(3) [],
    /// Reset system mode
    RstSysMode OFFSET(5) NUMBITS(2) [
        /// SOC system ewver{(gated by reset mask registers)
        SoCSystem = 0,
        /// Full chip
        FullChip = 1,
        /// CPU/FMC only, just reboot firmware, no any other IPs will be reset.
        SoCSystemFwOnly = 2,
        FullChipFwOnly = 3,
    ],
    /// Enable WDT(n) to be reset by SOC reset
    EnblWDTnToBeRstBySOCReset OFFSET(4) NUMBITS(1) [
        /// WDT(n) can not be reset by SOC reset, only SRST\# or Full chip reset can reset i
        NotRstBySoCRst = 0,
        /// WDT(n) can be reset by SOC reset.
        RstBySoCRst = 1,
    ],
    /// wdt_ext: External signal enable after timeout
    WdtextExtSigEnblAfterTimeout OFFSET(3) NUMBITS(1) [
        /// disable
        Disable = 0,
        /// enable
        Enable = 1,
    ],
    /// wdt_intr: Generate interrupt when timer count matches to the pre-timeout setting
    WdtIntrGen OFFSET(2) NUMBITS(1) [
        /// disable
        Disable = 0,
        /// enable
        Enable = 1,
    ],
    /// Reset system after timeout
    RstSysAfterTimeout OFFSET(1) NUMBITS(1) [
        /// disable
        Disable = 0,
        /// enable
        Enable = 1,
    ],
    /// WDT enable signal
    WDTEnblSig OFFSET(0) NUMBITS(1) [
        /// disable
        Disable = 0,
        /// enable
        Enable = 1,
    ]
],
TimeoutSts [
    /// Reserved (0)
    Reserved0 OFFSET(24) NUMBITS(8) [],
    /// Watchdog event counter : ARM reset
    WDEventCounterARMRst OFFSET(20) NUMBITS(4) [],
    /// Watchdog event counter : Full reset
    WDEventCounterFullRst OFFSET(16) NUMBITS(4) [],
    /// Watchdog event counter : SOC reset
    WDEventCounterSOCRst OFFSET(8) NUMBITS(8) [],
    /// Reserved (0)
    Reserved01 OFFSET(3) NUMBITS(5) [],
    /// Indicate interrupt
    IndicateINT OFFSET(2) NUMBITS(1) [],
    /// Reserved (0)
    Reserved02 OFFSET(1) NUMBITS(1) [],
    /// Indicate timeout
    IndicateTimeout OFFSET(0) NUMBITS(1) [
        /// timeout never occur
        TimeoutNeverOccur = 0,
        /// timeout occur
        TimeoutOccur = 1,
    ]
],
pub TimeoutClr [
    /// Reserved (0)
    Reserved0 OFFSET(8) NUMBITS(24) [],
    /// Clear timeout counter status
    ClearTimeoutCounterSts OFFSET(1) NUMBITS(7) [],
    /// Clear timeout, boot code selection and interrupt status
    ClearTimeoutAndIrq OFFSET(0) NUMBITS(1) []
],
RstWidth [
    /// Reset pulse polarity selection
    RstPulsePolaritySel OFFSET(31) NUMBITS(1) [
        /// active low output
        ActiveLowOutput = 0,
        /// active high output
        ActiveHighOutput = 1,
    ],
    /// Reset pulse output driving type
    RstPulseOutputDrivingType OFFSET(30) NUMBITS(1) [
        /// open-drain
        OpenDrain = 0,
        /// push-pull
        PushPull = 1,
    ],
    /// Reset pulse generation trigger selection
    RstPulseGenerationTriggerSel OFFSET(29) NUMBITS(1) [
        /// trigger by timeout
        TriggerByTimeout = 0,
        /// trigger by pre-timeout
        TriggerByPreTimeout = 1,
    ],
    /// Reserved (0)
    Reserved0 OFFSET(20) NUMBITS(9) [],
    /// Reset width
    RstWidth OFFSET(0) NUMBITS(20) []
],
RstMask1 [
    /// Enable EXTRSTN of H2x Controller
    EnRstH2xCtrl OFFSET(31) NUMBITS(1) [],
    /// Enable EXTRSTN of GP MCU
    EnRstGpMcu OFFSET(30) NUMBITS(1) [],
    /// Enable EXTRSTN of DP MCU
    EnRstDpMcu OFFSET(29) NUMBITS(1) [],
    /// Enable EXTRSTN of DP Controller
    EnRstDpCtrl OFFSET(28) NUMBITS(1) [],
    /// Enable EXTRSTN of XDMA \#1
    EnRstXDMA1 OFFSET(27) NUMBITS(1) [],
    /// Enable EXTRSTN of Graphic CRT
    EnRstGraphicCrt OFFSET(26) NUMBITS(1) [],
    /// Enable EXTRSTN of Jtag \#0
    EnRstJtag0 OFFSET(25) NUMBITS(1) [],
    /// Enable EXTRSTN of MCTP
    EnRstMctp OFFSET(24) NUMBITS(1) [],
    /// Enable EXTRSTN of MCTP8
    EnRstMctp8 OFFSET(23) NUMBITS(1) [],
    /// Enable EXTRSTN of XDMA \#0
    EnRstXDMA0 OFFSET(22) NUMBITS(1) [],
    /// Reserved (0)
    Reserved0 OFFSET(19) NUMBITS(3) [],
    /// Enable EXTRSTN of H2x8 Controller
    EnRstH2x8Ctrl OFFSET(18) NUMBITS(1) [],
    /// Enable EXTRSTN of SD/SDIO \#0
    EnRstSdSdio0 OFFSET(17) NUMBITS(1) [],
    /// Enable EXTRSTN of UFS Controller
    EnRstUfsCtrl OFFSET(16) NUMBITS(1) [],
    /// Enable EXTRSTN of USB 1.1 UHCI Host Controller
    EnRstUsb11UhciHost OFFSET(15) NUMBITS(1) [],
    /// Enable EXTRSTN of USB 2.0 Host/Hub Controller
    EnRstUsb20HostHub OFFSET(14) NUMBITS(1) [],
    /// Enable EXTRSTN of Graphics CRT \#0
    EnRstGraphicsCrt0 OFFSET(13) NUMBITS(1) [],
    /// Enable EXTRSTN of TSP
    EnRstTsp OFFSET(12) NUMBITS(1) [],
    /// Enable EXTRSTN of SSP
    EnRstSsp OFFSET(11) NUMBITS(1) [],
    /// Enable EXTRSTN of GPIO \#0
    EnRstGpio0 OFFSET(10) NUMBITS(1) [],
    /// Enable EXTRSTN of RVAS \#0
    EnRstRvas0 OFFSET(9) NUMBITS(1) [],
    /// Enable reset EXTRSTN of RVAS \#1
    EnRstRvas1 OFFSET(8) NUMBITS(1) [],
    /// Enable EXTRSTN of CRT mode 2D Engine
    EnRstCrt2D OFFSET(7) NUMBITS(1) [],
    /// Enable EXTRSTN of Video Engine
    EnRstVideo OFFSET(6) NUMBITS(1) [],
    /// Enable EXTRSTN of SoC Contrller
    EnRstSoCCtrl OFFSET(5) NUMBITS(1) [],
    /// Enable EXTRSTN of HACE
    EnRstHace OFFSET(4) NUMBITS(1) [],
    /// Enable EXTRSTN of EHCI
    EnRstEhci OFFSET(3) NUMBITS(1) [],
    /// Enable EXTRSTN of AHB \#0 and SLI \#0
    EnRstAhbSli0 OFFSET(2) NUMBITS(1) [],
    /// Enable EXTRSTN of SDRAM controller
    EnRstSdramCtrl OFFSET(1) NUMBITS(1) [],
    /// Enable EXTRSTN of ARM Related Controller
    EnRstArm OFFSET(0) NUMBITS(1) []
],
RstMask2 [
    /// Reserved (0)
    Reserved0 OFFSET(10) NUMBITS(22) [],
    /// Enable EXTRSTN of RSA
    EnRstRsa OFFSET(9) NUMBITS(1) [],
    /// Enable EXTRSTN of SHA3
    EnRstSha3 OFFSET(8) NUMBITS(1) [],
    /// Enable EXTRSTN of SM4
    EnRstSm4 OFFSET(7) NUMBITS(1) [],
    /// Enable EXTRSTN of SM3
    EnRstSm3 OFFSET(6) NUMBITS(1) [],
    /// Enable EXTRSTN of USB3 PortB VHUB2 Controller
    EnRstUsb3PortBVhub2Ctrl OFFSET(5) NUMBITS(1) [],
    /// Enable EXTRSTN of USB3 PortB VHUB3 Controller
    EnRstUsb3PortBVhub3Ctrl OFFSET(4) NUMBITS(1) [],
    /// Enable EXTRSTN of USB3 PortB Host Controller
    EnRstUsb3PortBHostCtrl OFFSET(3) NUMBITS(1) [],
    /// Enable EXTRSTN of USB3 PortA VHUB2 Controller
    EnRstUsb3PortAVhub2Ctrl OFFSET(2) NUMBITS(1) [],
    /// Enable EXTRSTN of USB3 PortA VHUB3 Controller
    EnRstUsb3PortAVhub3Ctrl OFFSET(1) NUMBITS(1) [],
    /// Enable EXTRSTN of USB3 PortA Host Controller
    EnRstUsb3PortAHostCtrl OFFSET(0) NUMBITS(1) []
],
RstMask3 [
    /// Enable EXTRSTN of I3C15
    EnRstI3C15 OFFSET(31) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C14
    EnRstI3C14 OFFSET(30) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C13
    EnRstI3C13 OFFSET(29) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C12
    EnRstI3C12 OFFSET(28) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C11
    EnRstI3C11 OFFSET(27) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C10
    EnRstI3C10 OFFSET(26) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C9
    EnRstI3C9 OFFSET(25) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C8
    EnRstI3C8 OFFSET(24) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C7
    EnRstI3C7 OFFSET(23) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C6
    EnRstI3C6 OFFSET(22) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C5
    EnRstI3C5 OFFSET(21) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C4
    EnRstI3C4 OFFSET(20) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C3
    EnRstI3C3 OFFSET(19) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C2
    EnRstI3C2 OFFSET(18) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C1
    EnRstI3C1 OFFSET(17) NUMBITS(1) [],
    /// Enable EXTRSTN of I3C0
    EnRstI3C0 OFFSET(16) NUMBITS(1) [],
    /// Enable EXTRSTN of SPI2
    EnRstSpi2 OFFSET(15) NUMBITS(1) [],
    /// Enable EXTRSTN of SPI1
    EnRstSpi1 OFFSET(14) NUMBITS(1) [],
    /// Enable EXTRSTN of SPI0
    EnRstSpi0 OFFSET(13) NUMBITS(1) [],
    /// Enable EXTRSTN of Jtag
    EnRstJtag OFFSET(12) NUMBITS(1) [],
    /// Enable EXTRSTN of eSPI1
    EnRstEspi1 OFFSET(11) NUMBITS(1) [],
    /// Enable EXTRSTN of eSPI0
    EnRstEspi0 OFFSET(10) NUMBITS(1) [],
    /// Enable EXTRSTN of SDIO
    EnRstSdio OFFSET(9) NUMBITS(1) [],
    /// Enable reset EXTRSTN ADC
    EnRstAdc OFFSET(8) NUMBITS(1) [],
    /// Enable EXTRSTN of MAC2
    EnRstMac2 OFFSET(7) NUMBITS(1) [],
    /// Enable EXTRSTN of MAC1
    EnRstMac1 OFFSET(6) NUMBITS(1) [],
    /// Enable EXTRSTN of MAC0
    EnRstMac0 OFFSET(5) NUMBITS(1) [],
    /// Enable EXTRSTN of PWM
    EnRstPwm OFFSET(4) NUMBITS(1) [],
    /// Enable EXTRSTN of PECI
    EnRstPeci OFFSET(3) NUMBITS(1) [],
    /// Enable EXTRSTN of MDC/MDIO
    EnRstMdcMdio OFFSET(2) NUMBITS(1) [],
    /// Enable EXTRSTN of LPCI1
    EnRstLpci1 OFFSET(1) NUMBITS(1) [],
    /// Enable EXTRSTN of LPCI0
    EnRstLpci0 OFFSET(0) NUMBITS(1) []
],
RstMask4 [
    /// Enable EXTRSTN of I3C DMA
    EnRstI3CDma OFFSET(31) NUMBITS(1) [],
    /// Enable EXTRSTN of H2X
    EnRstH2X OFFSET(30) NUMBITS(1) [],
    /// Enable EXTRSTN of LTPI GPIO1
    EnRstLTPIGpio1 OFFSET(29) NUMBITS(1) [],
    /// Enable EXTRSTN of LTPI GPIO0
    EnRstLTPIGpio0 OFFSET(28) NUMBITS(1) [],
    /// Reserved (0)
    Reserved0 OFFSET(27) NUMBITS(1) [],
    /// Enable EXTRSTN of ACE
    EnRstACE OFFSET(26) NUMBITS(1) [],
    /// Reserved (1)
    Reserved1 OFFSET(25) NUMBITS(1) [],
    /// Enable EXTRSTN of LTPI1
    EnRstLTPI1 OFFSET(24) NUMBITS(1) [],
    /// Enable EXTRSTN of VGA Link
    EnRstVGALink OFFSET(23) NUMBITS(1) [],
    /// Enable EXTRSTN of LTPI0
    EnRstLTPI0 OFFSET(22) NUMBITS(1) [],
    /// Enable EXTRSTN of IOMCU
    EnRstIOMCU OFFSET(21) NUMBITS(1) [],
    /// Enable EXTRSTN of BootMCU
    EnRstBootMCU OFFSET(20) NUMBITS(1) [],
    /// Enable EXTRSTN of UART8
    EnRstUART8 OFFSET(19) NUMBITS(1) [],
    /// Enable EXTRSTN of UART7
    EnRstUART7 OFFSET(18) NUMBITS(1) [],
    /// Enable EXTRSTN of UART6
    EnRstUART6 OFFSET(17) NUMBITS(1) [],
    /// Enable EXTRSTN of UART5
    EnRstUART5 OFFSET(16) NUMBITS(1) [],
    /// Enable EXTRSTN of XDMA
    EnRstXDMA OFFSET(15) NUMBITS(1) [],
    /// Enable EXTRSTN of MCTP
    EnRstMCTP OFFSET(14) NUMBITS(1) [],
    /// Enable EXTRSTN of CANBUS
    EnRstCANBUS OFFSET(13) NUMBITS(1) [],
    /// Enable EXTRSTN of FSI
    EnRstFSI OFFSET(12) NUMBITS(1) [],
    /// Enable EXTRSTN of GPIO
    EnRstGPIO OFFSET(11) NUMBITS(1) [],
    /// Enable EXTRSTN of H2AS1
    EnRstH2AS1 OFFSET(10) NUMBITS(1) [],
    /// Enable EXTRSTN of H2AS0
    EnRstH2AS0 OFFSET(9) NUMBITS(1) [],
    /// Enable reset EXTRSTN I2C Filter
    EnRstI2CFilter OFFSET(8) NUMBITS(1) [],
    /// Enable EXTRSTN of UART3
    EnRstUART3 OFFSET(7) NUMBITS(1) [],
    /// Enable EXTRSTN of UART2
    EnRstUART2 OFFSET(6) NUMBITS(1) [],
    /// Enable EXTRSTN of UART1
    EnRstUART1 OFFSET(5) NUMBITS(1) [],
    /// Enable EXTRSTN of UART0
    EnRstUART0 OFFSET(4) NUMBITS(1) [],
    /// Enable EXTRSTN of SLI
    EnRstSLI OFFSET(3) NUMBITS(1) [],
    /// Enable EXTRSTN of MDC/AHB
    EnRstAHB OFFSET(2) NUMBITS(1) [],
    /// Enable EXTRSTN of SOC
    EnRstSOC OFFSET(1) NUMBITS(1) [],
    /// Enable EXTRSTN of IOFMC
    EnRstIOFMC OFFSET(0) NUMBITS(1) []
],
pub RstMask5 [
    /// Reserved
    Reserved OFFSET(27) NUMBITS(5) [],
    /// Enable EXTRSTN of RTC
    EnRstRTC OFFSET(26) NUMBITS(1) [],
    /// Enable EXTRSTN of Ethernet PHY
    EnRstEthernetPHY OFFSET(25) NUMBITS(1) [],
    /// Enable EXTRSTN of Caliptra SS
    EnRstCptraSS OFFSET(24) NUMBITS(1) [],
    /// Enable EXTRSTN of VGA clock domain
    EnRstVgaClk OFFSET(23) NUMBITS(1) [],
    /// Enable EXTRSTN of VGA SLIV clock domain
    EnRstVgaSLIVClk OFFSET(22) NUMBITS(1) [],
    /// Enable EXTRSTN of USB2D
    EnRstUSB2D OFFSET(21) NUMBITS(1) [],
    /// Enable EXTRSTN of USB2UARTD
    EnRstUSB2UARTD OFFSET(20) NUMBITS(1) [],
    /// Enable EXTRSTN of USB2C
    EnRstUSB2C OFFSET(19) NUMBITS(1) [],
    /// Enable EXTRSTN of USB2UARTC
    EnRstUSB2UARTC OFFSET(18) NUMBITS(1) [],
    /// Enable EXTRSTN of UHCI
    EnRstUHCI OFFSET(17) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C15
    EnRstI2C15 OFFSET(16) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C14
    EnRstI2C14 OFFSET(15) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C13
    EnRstI2C13 OFFSET(14) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C12
    EnRstI2C12 OFFSET(13) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C11
    EnRstI2C11 OFFSET(12) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C10
    EnRstI2C10 OFFSET(11) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C9
    EnRstI2C9 OFFSET(10) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C8
    EnRstI2C8 OFFSET(9) NUMBITS(1) [],
    /// Enable reset EXTRSTN I2C I2C7
    EnRstI2C7 OFFSET(8) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C6
    EnRstI2C6 OFFSET(7) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C5
    EnRstI2C5 OFFSET(6) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C4
    EnRstI2C4 OFFSET(5) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C3
    EnRstI2C3 OFFSET(4) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C2
    EnRstI2C2 OFFSET(3) NUMBITS(1) [],
    /// Enable EXTRSTN of MDC/I2C1
    EnRstI2C1 OFFSET(2) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C0
    EnRstI2C0 OFFSET(1) NUMBITS(1) [],
    /// Enable EXTRSTN of I2C Global
    EnRstI2CGlobal OFFSET(0) NUMBITS(1) []
],
SwRstCtrl [
    /// Reserved
    Reserved OFFSET(8) NUMBITS(24) [],
    /// Watchdog event counter
    WDEventCounter OFFSET(4) NUMBITS(4) [],
    /// Reserved
    Reserved1 OFFSET(1) NUMBITS(3) [],
    /// Enable trigger software mode reset
    EnblTriggerSwModeRst OFFSET(0) NUMBITS(1) []
],
SwRstMask1 [
    /// Enable Software Mode Reset of H2x Controller
    EnRstH2xCtrl OFFSET(31) NUMBITS(1) [],
    /// Enable Software Mode Reset of GP MCU
    EnRstGpMcu OFFSET(30) NUMBITS(1) [],
    /// Enable Software Mode Reset of DP MCU
    EnRstDpMcu OFFSET(29) NUMBITS(1) [],
    /// Enable Software Mode Reset of DP Controller
    EnRstDpCtrl OFFSET(28) NUMBITS(1) [],
    /// Enable Software Mode Reset of XDMA \#1
    EnRstXDMA1 OFFSET(27) NUMBITS(1) [],
    /// Enable Software Mode Reset of Graphic CRT
    EnRstGraphicCrt OFFSET(26) NUMBITS(1) [],
    /// Enable Software Mode Reset of Jtag \#0
    EnRstJtag0 OFFSET(25) NUMBITS(1) [],
    /// Enable Software Mode Reset of MCTP
    EnRstMctp OFFSET(24) NUMBITS(1) [],
    /// Enable Software Mode Reset of MCTP8
    EnRstMctp8 OFFSET(23) NUMBITS(1) [],
    /// Enable Software Mode Reset of XDMA \#0
    EnRstXDMA0 OFFSET(22) NUMBITS(1) [],
    /// Reserved (0)
    Reserved0 OFFSET(19) NUMBITS(3) [],
    /// Enable Software Mode Reset of H2x8 Controller
    EnRstH2x8Ctrl OFFSET(18) NUMBITS(1) [],
    /// Enable Software Mode Reset of SD/SDIO \#0
    EnRstSdSdio0 OFFSET(17) NUMBITS(1) [],
    /// Enable Software Mode Reset of UFS Controller
    EnRstUfsCtrl OFFSET(16) NUMBITS(1) [],
    /// Enable Software Mode Reset of USB 1.1 UHCI Host Controller
    EnRstUsb11UhciHost OFFSET(15) NUMBITS(1) [],
    /// Enable Software Mode Reset of USB 2.0 Host/Hub Controller
    EnRstUsb20HostHub OFFSET(14) NUMBITS(1) [],
    /// Enable Software Mode Reset of Graphics CRT \#0
    EnRstGraphicsCrt0 OFFSET(13) NUMBITS(1) [],
    /// Enable Software Mode Reset of TSP
    EnRstTsp OFFSET(12) NUMBITS(1) [],
    /// Enable Software Mode Reset of SSP
    EnRstSsp OFFSET(11) NUMBITS(1) [],
    /// Enable Software Mode Reset of GPIO \#0
    EnRstGpio0 OFFSET(10) NUMBITS(1) [],
    /// Enable Software Mode Reset of RVAS \#0
    EnRstRvas0 OFFSET(9) NUMBITS(1) [],
    /// Enable reset Software Mode Reset of RVAS \#1
    EnRstRvas1 OFFSET(8) NUMBITS(1) [],
    /// Enable Software Mode Reset of CRT mode 2D Engine
    EnRstCrt2D OFFSET(7) NUMBITS(1) [],
    /// Enable Software Mode Reset of Video Engine
    EnRstVideo OFFSET(6) NUMBITS(1) [],
    /// Enable Software Mode Reset of SoC Contrller
    EnRstSoCCtrl OFFSET(5) NUMBITS(1) [],
    /// Enable Software Mode Reset of HACE
    EnRstHace OFFSET(4) NUMBITS(1) [],
    /// Enable Software Mode Reset of EHCI
    EnRstEhci OFFSET(3) NUMBITS(1) [],
    /// Enable Software Mode Reset of AHB \#0 and SLI \#0
    EnRstAhbSli0 OFFSET(2) NUMBITS(1) [],
    /// Enable Software Mode Reset of SDRAM controller
    EnRstSdramCtrl OFFSET(1) NUMBITS(1) [],
    /// Enable Software Mode Reset of ARM Related Controller
    EnRstArm OFFSET(0) NUMBITS(1) []
],
SwRstMask2 [
    /// Reserved (0)
    Reserved0 OFFSET(10) NUMBITS(22) [],
    /// Enable Software Mode Reset of RSA
    EnRstRsa OFFSET(9) NUMBITS(1) [],
    /// Enable Software Mode Reset of SHA3
    EnRstSha3 OFFSET(8) NUMBITS(1) [],
    /// Enable Software Mode Reset of SM4
    EnRstSm4 OFFSET(7) NUMBITS(1) [],
    /// Enable Software Mode Reset of SM3
    EnRstSm3 OFFSET(6) NUMBITS(1) [],
    /// Enable Software Mode Reset of USB3 PortB VHUB2 Controller
    EnRstUsb3PortBVhub2Ctrl OFFSET(5) NUMBITS(1) [],
    /// Enable Software Mode Reset of USB3 PortB VHUB3 Controller
    EnRstUsb3PortBVhub3Ctrl OFFSET(4) NUMBITS(1) [],
    /// Enable Software Mode Reset of USB3 PortB Host Controller
    EnRstUsb3PortBHostCtrl OFFSET(3) NUMBITS(1) [],
    /// Enable Software Mode Reset of USB3 PortA VHUB2 Controller
    EnRstUsb3PortAVhub2Ctrl OFFSET(2) NUMBITS(1) [],
    /// Enable Software Mode Reset of USB3 PortA VHUB3 Controller
    EnRstUsb3PortAVhub3Ctrl OFFSET(1) NUMBITS(1) [],
    /// Enable Software Mode Reset of USB3 PortA Host Controller
    EnRstUsb3PortAHostCtrl OFFSET(0) NUMBITS(1) []
],
SwRstMask3 [
    /// Enable Software Mode Reset of I3C15
    EnRstI3C15 OFFSET(31) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C14
    EnRstI3C14 OFFSET(30) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C13
    EnRstI3C13 OFFSET(29) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C12
    EnRstI3C12 OFFSET(28) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C11
    EnRstI3C11 OFFSET(27) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C10
    EnRstI3C10 OFFSET(26) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C9
    EnRstI3C9 OFFSET(25) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C8
    EnRstI3C8 OFFSET(24) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C7
    EnRstI3C7 OFFSET(23) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C6
    EnRstI3C6 OFFSET(22) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C5
    EnRstI3C5 OFFSET(21) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C4
    EnRstI3C4 OFFSET(20) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C3
    EnRstI3C3 OFFSET(19) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C2
    EnRstI3C2 OFFSET(18) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C1
    EnRstI3C1 OFFSET(17) NUMBITS(1) [],
    /// Enable Software Mode Reset of I3C0
    EnRstI3C0 OFFSET(16) NUMBITS(1) [],
    /// Enable Software Mode Reset of SPI2
    EnRstSpi2 OFFSET(15) NUMBITS(1) [],
    /// Enable Software Mode Reset of SPI1
    EnRstSpi1 OFFSET(14) NUMBITS(1) [],
    /// Enable Software Mode Reset of SPI0
    EnRstSpi0 OFFSET(13) NUMBITS(1) [],
    /// Enable Software Mode Reset of Jtag
    EnRstJtag OFFSET(12) NUMBITS(1) [],
    /// Enable Software Mode Reset of eSPI1
    EnRstEspi1 OFFSET(11) NUMBITS(1) [],
    /// Enable Software Mode Reset of eSPI0
    EnRstEspi0 OFFSET(10) NUMBITS(1) [],
    /// Enable Software Mode Reset of SDIO
    EnRstSdio OFFSET(9) NUMBITS(1) [],
    /// Enable reset Software Mode Reset ADC
    EnRstAdc OFFSET(8) NUMBITS(1) [],
    /// Enable Software Mode Reset of MAC2
    EnRstMac2 OFFSET(7) NUMBITS(1) [],
    /// Enable Software Mode Reset of MAC1
    EnRstMac1 OFFSET(6) NUMBITS(1) [],
    /// Enable Software Mode Reset of MAC0
    EnRstMac0 OFFSET(5) NUMBITS(1) [],
    /// Enable Software Mode Reset of PWM
    EnRstPwm OFFSET(4) NUMBITS(1) [],
    /// Enable Software Mode Reset of PECI
    EnRstPeci OFFSET(3) NUMBITS(1) [],
    /// Enable Software Mode Reset of MDC/MDIO
    EnRstMdcMdio OFFSET(2) NUMBITS(1) [],
    /// Enable Software Mode Reset of LPCI1
    EnRstLpci1 OFFSET(1) NUMBITS(1) [],
    /// Enable Software Mode Reset of LPCI0
    EnRstLpci0 OFFSET(0) NUMBITS(1) []
],
SwRstMask4 [
    /// Enable Software Mode Reset of I3C DMA
    EnRstI3CDma OFFSET(31) NUMBITS(1) [],
    /// Enable Software Mode Reset of H2X
    EnRstH2X OFFSET(30) NUMBITS(1) [],
    /// Enable Software Mode Reset of LTPI GPIO1
    EnRstLTPIGpio1 OFFSET(29) NUMBITS(1) [],
    /// Enable Software Mode Reset of LTPI GPIO0
    EnRstLTPIGpio0 OFFSET(28) NUMBITS(1) [],
    /// Reserved (0)
    Reserved0 OFFSET(27) NUMBITS(1) [],
    /// Enable Software Mode Reset of ACE
    EnRstACE OFFSET(26) NUMBITS(1) [],
    /// Reserved (1)
    Reserved1 OFFSET(25) NUMBITS(1) [],
    /// Enable Software Mode Reset of LTPI1
    EnRstLTPI1 OFFSET(24) NUMBITS(1) [],
    /// Enable Software Mode Reset of VGA Link
    EnRstVGALink OFFSET(23) NUMBITS(1) [],
    /// Enable Software Mode Reset of LTPI0
    EnRstLTPI0 OFFSET(22) NUMBITS(1) [],
    /// Enable Software Mode Reset of IOMCU
    EnRstIOMCU OFFSET(21) NUMBITS(1) [],
    /// Enable Software Mode Reset of BootMCU
    EnRstBootMCU OFFSET(20) NUMBITS(1) [],
    /// Enable Software Mode Reset of UART8
    EnRstUART8 OFFSET(19) NUMBITS(1) [],
    /// Enable Software Mode Reset of UART7
    EnRstUART7 OFFSET(18) NUMBITS(1) [],
    /// Enable Software Mode Reset of UART6
    EnRstUART6 OFFSET(17) NUMBITS(1) [],
    /// Enable Software Mode Reset of UART5
    EnRstUART5 OFFSET(16) NUMBITS(1) [],
    /// Enable Software Mode Reset of XDMA
    EnRstXDMA OFFSET(15) NUMBITS(1) [],
    /// Enable Software Mode Reset of MCTP
    EnRstMCTP OFFSET(14) NUMBITS(1) [],
    /// Enable Software Mode Reset of CANBUS
    EnRstCANBUS OFFSET(13) NUMBITS(1) [],
    /// Enable Software Mode Reset of FSI
    EnRstFSI OFFSET(12) NUMBITS(1) [],
    /// Enable Software Mode Reset of GPIO
    EnRstGPIO OFFSET(11) NUMBITS(1) [],
    /// Enable Software Mode Reset of H2AS1
    EnRstH2AS1 OFFSET(10) NUMBITS(1) [],
    /// Enable Software Mode Reset of H2AS0
    EnRstH2AS0 OFFSET(9) NUMBITS(1) [],
    /// Enable reset Software Mode Reset I2C Filter
    EnRstI2CFilter OFFSET(8) NUMBITS(1) [],
    /// Enable Software Mode Reset of UART3
    EnRstUART3 OFFSET(7) NUMBITS(1) [],
    /// Enable Software Mode Reset of UART2
    EnRstUART2 OFFSET(6) NUMBITS(1) [],
    /// Enable Software Mode Reset of UART1
    EnRstUART1 OFFSET(5) NUMBITS(1) [],
    /// Enable Software Mode Reset of UART0
    EnRstUART0 OFFSET(4) NUMBITS(1) [],
    /// Enable Software Mode Reset of SLI
    EnRstSLI OFFSET(3) NUMBITS(1) [],
    /// Enable Software Mode Reset of MDC/AHB
    EnRstAHB OFFSET(2) NUMBITS(1) [],
    /// Enable Software Mode Reset of SOC
    EnRstSOC OFFSET(1) NUMBITS(1) [],
    /// Enable Software Mode Reset of IOFMC
    EnRstIOFMC OFFSET(0) NUMBITS(1) []
],
SwRstMask5 [
    /// Reserved
    Reserved OFFSET(27) NUMBITS(5) [],
    /// Enable Software Mode Reset of RTC
    EnRstRTC OFFSET(26) NUMBITS(1) [],
    /// Enable Software Mode Reset of Ethernet PHY
    EnRstEthernetPHY OFFSET(25) NUMBITS(1) [],
    /// Enable Software Mode Reset of Caliptra SS
    EnRstCptraSS OFFSET(24) NUMBITS(1) [],
    /// Enable Software Mode Reset of VGA clock domain
    EnRstVgaClk OFFSET(23) NUMBITS(1) [],
    /// Enable Software Mode Reset of VGA SLIV clock domain
    EnRstVgaSLIVClk OFFSET(22) NUMBITS(1) [],
    /// Enable Software Mode Reset of USB2D
    EnRstUSB2D OFFSET(21) NUMBITS(1) [],
    /// Enable Software Mode Reset of USB2UARTD
    EnRstUSB2UARTD OFFSET(20) NUMBITS(1) [],
    /// Enable Software Mode Reset of USB2C
    EnRstUSB2C OFFSET(19) NUMBITS(1) [],
    /// Enable Software Mode Reset of USB2UARTC
    EnRstUSB2UARTC OFFSET(18) NUMBITS(1) [],
    /// Enable Software Mode Reset of UHCI
    EnRstUHCI OFFSET(17) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C15
    EnRstI2C15 OFFSET(16) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C14
    EnRstI2C14 OFFSET(15) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C13
    EnRstI2C13 OFFSET(14) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C12
    EnRstI2C12 OFFSET(13) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C11
    EnRstI2C11 OFFSET(12) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C10
    EnRstI2C10 OFFSET(11) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C9
    EnRstI2C9 OFFSET(10) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C8
    EnRstI2C8 OFFSET(9) NUMBITS(1) [],
    /// Enable reset Software Mode Reset I2C I2C7
    EnRstI2C7 OFFSET(8) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C6
    EnRstI2C6 OFFSET(7) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C5
    EnRstI2C5 OFFSET(6) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C4
    EnRstI2C4 OFFSET(5) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C3
    EnRstI2C3 OFFSET(4) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C2
    EnRstI2C2 OFFSET(3) NUMBITS(1) [],
    /// Enable Software Mode Reset of MDC/I2C1
    EnRstI2C1 OFFSET(2) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C0
    EnRstI2C0 OFFSET(1) NUMBITS(1) [],
    /// Enable Software Mode Reset of I2C Global
    EnRstI2CGlobal OFFSET(0) NUMBITS(1) []
],
DisCtrl [
    /// Disable watchdog Software mode reset
    DisWatchdogSwModeRst OFFSET(5) NUMBITS(1) [],
    /// Disable watchdog interrupt generation
    DisWatchdogINTGeneration OFFSET(4) NUMBITS(1) [],
    /// Disable watchdog CPU/FMC reset mode
    DisWatchdogCPUFMCRstMode OFFSET(3) NUMBITS(1) [],
    /// Disable watchdog Full reset mode
    DisWatchdogFullRstMode OFFSET(2) NUMBITS(1) [],
    /// Disable watchdog SOC reset mode
    DisWatchdogSOCRstMode OFFSET(1) NUMBITS(1) [],
    /// Disable full watchdog function
    DisFullWatchdogFn OFFSET(0) NUMBITS(1) []
],
ScratchReg [
    /// Write 0xea to Clear Scratch Register
    ClrScratch OFFSET(24) NUMBITS(8) [],
    /// Reserved (0)
    Reserved0 OFFSET(8) NUMBITS(16) [],
    /// Scratch Register
    ScratchReg OFFSET(0) NUMBITS(8) []
],
RstMaskWriteProt1 [
    /// Write Protection of Enable EXTRSTN of H2x Controller
    WpEnRstH2xCtrl OFFSET(31) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of GP MCU
    WpEnRstGpMcu OFFSET(30) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of DP MCU
    WpEnRstDpMcu OFFSET(29) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of DP Controller
    WpEnRstDpCtrl OFFSET(28) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of XDMA \#1
    WpEnRstXDMA1 OFFSET(27) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of Graphic CRT
    WpEnRstGraphicCrt OFFSET(26) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of Jtag \#0
    WpEnRstJtag0 OFFSET(25) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of MCTP
    WpEnRstMctp OFFSET(24) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of MCTP8
    WpEnRstMctp8 OFFSET(23) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of XDMA \#0
    WpEnRstXDMA0 OFFSET(22) NUMBITS(1) [],
    /// Reserved (0)
    Reserved0 OFFSET(19) NUMBITS(3) [],
    /// Write Protection of Enable EXTRSTN of H2x8 Controller
    WpEnRstH2x8Ctrl OFFSET(18) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of SD/SDIO \#0
    WpEnRstSdSdio0 OFFSET(17) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of UFS Controller
    WpEnRstUfsCtrl OFFSET(16) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of USB 1.1 UHCI Host Controller
    WpEnRstUsb11UhciHost OFFSET(15) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of USB 2.0 Host/Hub Controller
    WpEnRstUsb20HostHub OFFSET(14) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of Graphics CRT \#0
    WpEnRstGraphicsCrt0 OFFSET(13) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of TSP
    WpEnRstTsp OFFSET(12) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of SSP
    WpEnRstSsp OFFSET(11) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of GPIO \#0
    WpEnRstGpio0 OFFSET(10) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of RVAS \#0
    WpEnRstRvas0 OFFSET(9) NUMBITS(1) [],
    /// Enable Write Protection of reset of RVAS \#1
    WpEnRstRvas1 OFFSET(8) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of CRT mode 2D Engine
    WpEnRstCrt2D OFFSET(7) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of Video Engine
    WpEnRstVideo OFFSET(6) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of SoC Contrller
    WpEnRstSoCCtrl OFFSET(5) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of HACE
    WpEnRstHace OFFSET(4) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of EHCI
    WpEnRstEhci OFFSET(3) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of AHB \#0 and SLI \#0
    WpEnRstAhbSli0 OFFSET(2) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of SDRAM controller
    WpEnRstSdramCtrl OFFSET(1) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of ARM Related Controller
    WpEnRstArm OFFSET(0) NUMBITS(1) []
],
RstMaskWriteProt2 [
    /// Reserved (0)
    Reserved0 OFFSET(10) NUMBITS(22) [],
    /// Write Protect of Enable EXTRSTN of RSA
    WpEnRstRsa OFFSET(9) NUMBITS(1) [],
    /// Write Protect of Enable EXTRSTN of SHA3
    WpEnRstSha3 OFFSET(8) NUMBITS(1) [],
    /// Write Protect of Enable EXTRSTN of SM4
    WpEnRstSm4 OFFSET(7) NUMBITS(1) [],
    /// Write Protect of Enable EXTRSTN of SM3
    WpEnRstSm3 OFFSET(6) NUMBITS(1) [],
    /// Write Protect of Enable EXTRSTN of USB3 PortB VHUB2 Controller
    WpEnRstUsb3PortBVhub2Ctrl OFFSET(5) NUMBITS(1) [],
    /// Write Protect of Enable EXTRSTN of USB3 PortB VHUB3 Controller
    WpEnRstUsb3PortBVhub3Ctrl OFFSET(4) NUMBITS(1) [],
    /// Write Protect of Enable EXTRSTN of USB3 PortB Host Controller
    WpEnRstUsb3PortBHostCtrl OFFSET(3) NUMBITS(1) [],
    /// Write Protect of Enable EXTRSTN of USB3 PortA VHUB2 Controller
    WpEnRstUsb3PortAVhub2Ctrl OFFSET(2) NUMBITS(1) [],
    /// Write Protect of Enable EXTRSTN of USB3 PortA VHUB3 Controller
    WpEnRstUsb3PortAVhub3Ctrl OFFSET(1) NUMBITS(1) [],
    /// Write Protect of Enable EXTRSTN of USB3 PortA Host Controller
    WpEnRstUsb3PortAHostCtrl OFFSET(0) NUMBITS(1) []
],
RstMaskWriteProt3 [
    /// Write Protection of Enable EXTRSTN of I3C15
    WpEnRstI3C15 OFFSET(31) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C14
    WpEnRstI3C14 OFFSET(30) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C13
    WpEnRstI3C13 OFFSET(29) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C12
    WpEnRstI3C12 OFFSET(28) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C11
    WpEnRstI3C11 OFFSET(27) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C10
    WpEnRstI3C10 OFFSET(26) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C9
    WpEnRstI3C9 OFFSET(25) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C8
    WpEnRstI3C8 OFFSET(24) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C7
    WpEnRstI3C7 OFFSET(23) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C6
    WpEnRstI3C6 OFFSET(22) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C5
    WpEnRstI3C5 OFFSET(21) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C4
    WpEnRstI3C4 OFFSET(20) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C3
    WpEnRstI3C3 OFFSET(19) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C2
    WpEnRstI3C2 OFFSET(18) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C1
    WpEnRstI3C1 OFFSET(17) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I3C0
    WpEnRstI3C0 OFFSET(16) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of SPI2
    WpEnRstSpi2 OFFSET(15) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of SPI1
    WpEnRstSpi1 OFFSET(14) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of SPI0
    WpEnRstSpi0 OFFSET(13) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of Jtag
    WpEnRstJtag OFFSET(12) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of eSPI1
    WpEnRstEspi1 OFFSET(11) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of eSPI0
    WpEnRstEspi0 OFFSET(10) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of SDIO
    WpEnRstSdio OFFSET(9) NUMBITS(1) [],
    /// Enable Write Protection of reset EXTRSTN ADC
    WpEnRstAdc OFFSET(8) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of MAC2
    WpEnRstMac2 OFFSET(7) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of MAC1
    WpEnRstMac1 OFFSET(6) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of MAC0
    WpEnRstMac0 OFFSET(5) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of PWM
    WpEnRstPwm OFFSET(4) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of PECI
    WpEnRstPeci OFFSET(3) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of MDC/MDIO
    WpEnRstMdcMdio OFFSET(2) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of LPCI1
    WpEnRstLpci1 OFFSET(1) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of LPCI0
    WpEnRstLpci0 OFFSET(0) NUMBITS(1) []
],
RstMaskWriteProt4 [
    /// Write Protection of Enable EXTRSTN of I3C DMA
    WpEnRstI3CDma OFFSET(31) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of H2X
    WpEnRstH2X OFFSET(30) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of LTPI GPIO1
    WpEnRstLTPIGpio1 OFFSET(29) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of LTPI GPIO0
    WpEnRstLTPIGpio0 OFFSET(28) NUMBITS(1) [],
    /// Reserved (0)
    Reserved0 OFFSET(27) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of ACE
    WpEnRstACE OFFSET(26) NUMBITS(1) [],
    /// Reserved (1)
    Reserved1 OFFSET(25) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of LTPI1
    WpEnRstLTPI1 OFFSET(24) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of VGA Link
    WpEnRstVGALink OFFSET(23) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of LTPI0
    WpEnRstLTPI0 OFFSET(22) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of IOMCU
    WpEnRstIOMCU OFFSET(21) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of BootMCU
    WpEnRstBootMCU OFFSET(20) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of UART8
    WpEnRstUART8 OFFSET(19) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of UART7
    WpEnRstUART7 OFFSET(18) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of UART6
    WpEnRstUART6 OFFSET(17) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of UART5
    WpEnRstUART5 OFFSET(16) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of XDMA
    WpEnRstXDMA OFFSET(15) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of MCTP
    WpEnRstMCTP OFFSET(14) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of CANBUS
    WpEnRstCANBUS OFFSET(13) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of FSI
    WpEnRstFSI OFFSET(12) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of GPIO
    WpEnRstGPIO OFFSET(11) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of H2AS1
    WpEnRstH2AS1 OFFSET(10) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of H2AS0
    WpEnRstH2AS0 OFFSET(9) NUMBITS(1) [],
    /// Enable Write Protection of reset EXTRSTN I2C Filter
    WpEnRstI2CFilter OFFSET(8) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of UART3
    WpEnRstUART3 OFFSET(7) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of UART2
    WpEnRstUART2 OFFSET(6) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of UART1
    WpEnRstUART1 OFFSET(5) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of UART0
    WpEnRstUART0 OFFSET(4) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of SLI
    WpEnRstSLI OFFSET(3) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of MDC/AHB
    WpEnRstAHB OFFSET(2) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of SOC
    WpEnRstSOC OFFSET(1) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of IOFMC
    WpEnRstIOFMC OFFSET(0) NUMBITS(1) []
],
RstMaskWriteProt5 [
    /// Reserved
    Reserved OFFSET(27) NUMBITS(5) [],
    /// Write Protection of Enable EXTRSTN of RTC
    WpEnRstRTC OFFSET(26) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of Ethernet PHY
    WpEnRstEthernetPHY OFFSET(25) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of Caliptra SS
    WpEnRstCptraSS OFFSET(24) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of VGA clock domain
    WpEnRstVgaClk OFFSET(23) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of VGA SLIV clock domain
    WpEnRstVgaSLIVClk OFFSET(22) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of USB2D
    WpEnRstUSB2D OFFSET(21) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of USB2UARTD
    WpEnRstUSB2UARTD OFFSET(20) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of USB2C
    WpEnRstUSB2C OFFSET(19) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of USB2UARTC
    WpEnRstUSB2UARTC OFFSET(18) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of UHCI
    WpEnRstUHCI OFFSET(17) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C15
    WpEnRstI2C15 OFFSET(16) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C14
    WpEnRstI2C14 OFFSET(15) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C13
    WpEnRstI2C13 OFFSET(14) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C12
    WpEnRstI2C12 OFFSET(13) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C11
    WpEnRstI2C11 OFFSET(12) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C10
    WpEnRstI2C10 OFFSET(11) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C9
    WpEnRstI2C9 OFFSET(10) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C8
    WpEnRstI2C8 OFFSET(9) NUMBITS(1) [],
    /// Enable Write Protection of reset EXTRSTN I2C I2C7
    WpEnRstI2C7 OFFSET(8) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C6
    WpEnRstI2C6 OFFSET(7) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C5
    WpEnRstI2C5 OFFSET(6) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C4
    WpEnRstI2C4 OFFSET(5) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C3
    WpEnRstI2C3 OFFSET(4) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C2
    WpEnRstI2C2 OFFSET(3) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of MDC/I2C1
    WpEnRstI2C1 OFFSET(2) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C0
    WpEnRstI2C0 OFFSET(1) NUMBITS(1) [],
    /// Write Protection of Enable EXTRSTN of I2C Global
    WpEnRstI2CGlobal OFFSET(0) NUMBITS(1) []
],
SwRstMaskWriteProt1 [
    /// Write Protection of Enable Software Mode Reset of H2x Controller
    WpEnRstH2xCtrl OFFSET(31) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of GP MCU
    WpEnRstGpMcu OFFSET(30) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of DP MCU
    WpEnRstDpMcu OFFSET(29) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of DP Controller
    WpEnRstDpCtrl OFFSET(28) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of XDMA \#1
    WpEnRstXDMA1 OFFSET(27) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of Graphic CRT
    WpEnRstGraphicCrt OFFSET(26) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of Jtag \#0
    WpEnRstJtag0 OFFSET(25) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of MCTP
    WpEnRstMctp OFFSET(24) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of MCTP8
    WpEnRstMctp8 OFFSET(23) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of XDMA \#0
    WpEnRstXDMA0 OFFSET(22) NUMBITS(1) [],
    /// Reserved (0)
    Reserved0 OFFSET(19) NUMBITS(3) [],
    /// Write Protection of Enable Software Mode Reset of H2x8 Controller
    WpEnRstH2x8Ctrl OFFSET(18) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of SD/SDIO \#0
    WpEnRstSdSdio0 OFFSET(17) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of UFS Controller
    WpEnRstUfsCtrl OFFSET(16) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of USB 1.1 UHCI Host Controller
    WpEnRstUsb11UhciHost OFFSET(15) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of USB 2.0 Host/Hub Controller
    WpEnRstUsb20HostHub OFFSET(14) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of Graphics CRT \#0
    WpEnRstGraphicsCrt0 OFFSET(13) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of TSP
    WpEnRstTsp OFFSET(12) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of SSP
    WpEnRstSsp OFFSET(11) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of GPIO \#0
    WpEnRstGpio0 OFFSET(10) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of RVAS \#0
    WpEnRstRvas0 OFFSET(9) NUMBITS(1) [],
    /// Enable Write Protection of reset of RVAS \#1
    WpEnRstRvas1 OFFSET(8) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of CRT mode 2D Engine
    WpEnRstCrt2D OFFSET(7) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of Video Engine
    WpEnRstVideo OFFSET(6) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of SoC Contrller
    WpEnRstSoCCtrl OFFSET(5) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of HACE
    WpEnRstHace OFFSET(4) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of EHCI
    WpEnRstEhci OFFSET(3) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of AHB \#0 and SLI \#0
    WpEnRstAhbSli0 OFFSET(2) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of SDRAM controller
    WpEnRstSdramCtrl OFFSET(1) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of ARM Related Controller
    WpEnRstArm OFFSET(0) NUMBITS(1) []
],
SwRstMaskWriteProt2 [
    /// Reserved (0)
    Reserved0 OFFSET(10) NUMBITS(22) [],
    /// Write Protect of Enable Software Mode Reset of RSA
    WpEnRstRsa OFFSET(9) NUMBITS(1) [],
    /// Write Protect of Enable Software Mode Reset of SHA3
    WpEnRstSha3 OFFSET(8) NUMBITS(1) [],
    /// Write Protect of Enable Software Mode Reset of SM4
    WpEnRstSm4 OFFSET(7) NUMBITS(1) [],
    /// Write Protect of Enable Software Mode Reset of SM3
    WpEnRstSm3 OFFSET(6) NUMBITS(1) [],
    /// Write Protect of Enable Software Mode Reset of USB3 PortB VHUB2 Controller
    WpEnRstUsb3PortBVhub2Ctrl OFFSET(5) NUMBITS(1) [],
    /// Write Protect of Enable Software Mode Reset of USB3 PortB VHUB3 Controller
    WpEnRstUsb3PortBVhub3Ctrl OFFSET(4) NUMBITS(1) [],
    /// Write Protect of Enable Software Mode Reset of USB3 PortB Host Controller
    WpEnRstUsb3PortBHostCtrl OFFSET(3) NUMBITS(1) [],
    /// Write Protect of Enable Software Mode Reset of USB3 PortA VHUB2 Controller
    WpEnRstUsb3PortAVhub2Ctrl OFFSET(2) NUMBITS(1) [],
    /// Write Protect of Enable Software Mode Reset of USB3 PortA VHUB3 Controller
    WpEnRstUsb3PortAVhub3Ctrl OFFSET(1) NUMBITS(1) [],
    /// Write Protect of Enable Software Mode Reset of USB3 PortA Host Controller
    WpEnRstUsb3PortAHostCtrl OFFSET(0) NUMBITS(1) []
],
SwRstMaskWriteProt3 [
    /// Write Protection of Enable Software Mode Reset of I3C15
    WpEnRstI3C15 OFFSET(31) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C14
    WpEnRstI3C14 OFFSET(30) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C13
    WpEnRstI3C13 OFFSET(29) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C12
    WpEnRstI3C12 OFFSET(28) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C11
    WpEnRstI3C11 OFFSET(27) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C10
    WpEnRstI3C10 OFFSET(26) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C9
    WpEnRstI3C9 OFFSET(25) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C8
    WpEnRstI3C8 OFFSET(24) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C7
    WpEnRstI3C7 OFFSET(23) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C6
    WpEnRstI3C6 OFFSET(22) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C5
    WpEnRstI3C5 OFFSET(21) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C4
    WpEnRstI3C4 OFFSET(20) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C3
    WpEnRstI3C3 OFFSET(19) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C2
    WpEnRstI3C2 OFFSET(18) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C1
    WpEnRstI3C1 OFFSET(17) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I3C0
    WpEnRstI3C0 OFFSET(16) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of SPI2
    WpEnRstSpi2 OFFSET(15) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of SPI1
    WpEnRstSpi1 OFFSET(14) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of SPI0
    WpEnRstSpi0 OFFSET(13) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of Jtag
    WpEnRstJtag OFFSET(12) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of eSPI1
    WpEnRstEspi1 OFFSET(11) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of eSPI0
    WpEnRstEspi0 OFFSET(10) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of SDIO
    WpEnRstSdio OFFSET(9) NUMBITS(1) [],
    /// Enable Write Protection of reset Software Mode Reset ADC
    WpEnRstAdc OFFSET(8) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of MAC2
    WpEnRstMac2 OFFSET(7) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of MAC1
    WpEnRstMac1 OFFSET(6) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of MAC0
    WpEnRstMac0 OFFSET(5) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of PWM
    WpEnRstPwm OFFSET(4) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of PECI
    WpEnRstPeci OFFSET(3) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of MDC/MDIO
    WpEnRstMdcMdio OFFSET(2) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of LPCI1
    WpEnRstLpci1 OFFSET(1) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of LPCI0
    WpEnRstLpci0 OFFSET(0) NUMBITS(1) []
],
SwRstMaskWriteProt4 [
    /// Write Protection of Enable Software Mode Reset of I3C DMA
    WpEnRstI3CDma OFFSET(31) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of H2X
    WpEnRstH2X OFFSET(30) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of LTPI GPIO1
    WpEnRstLTPIGpio1 OFFSET(29) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of LTPI GPIO0
    WpEnRstLTPIGpio0 OFFSET(28) NUMBITS(1) [],
    /// Reserved (0)
    Reserved0 OFFSET(27) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of ACE
    WpEnRstACE OFFSET(26) NUMBITS(1) [],
    /// Reserved (1)
    Reserved1 OFFSET(25) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of LTPI1
    WpEnRstLTPI1 OFFSET(24) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of VGA Link
    WpEnRstVGALink OFFSET(23) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of LTPI0
    WpEnRstLTPI0 OFFSET(22) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of IOMCU
    WpEnRstIOMCU OFFSET(21) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of BootMCU
    WpEnRstBootMCU OFFSET(20) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of UART8
    WpEnRstUART8 OFFSET(19) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of UART7
    WpEnRstUART7 OFFSET(18) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of UART6
    WpEnRstUART6 OFFSET(17) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of UART5
    WpEnRstUART5 OFFSET(16) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of XDMA
    WpEnRstXDMA OFFSET(15) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of MCTP
    WpEnRstMCTP OFFSET(14) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of CANBUS
    WpEnRstCANBUS OFFSET(13) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of FSI
    WpEnRstFSI OFFSET(12) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of GPIO
    WpEnRstGPIO OFFSET(11) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of H2AS1
    WpEnRstH2AS1 OFFSET(10) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of H2AS0
    WpEnRstH2AS0 OFFSET(9) NUMBITS(1) [],
    /// Enable Write Protection of reset Software Mode Reset I2C Filter
    WpEnRstI2CFilter OFFSET(8) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of UART3
    WpEnRstUART3 OFFSET(7) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of UART2
    WpEnRstUART2 OFFSET(6) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of UART1
    WpEnRstUART1 OFFSET(5) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of UART0
    WpEnRstUART0 OFFSET(4) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of SLI
    WpEnRstSLI OFFSET(3) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of MDC/AHB
    WpEnRstAHB OFFSET(2) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of SOC
    WpEnRstSOC OFFSET(1) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of IOFMC
    WpEnRstIOFMC OFFSET(0) NUMBITS(1) []
],
SwRstMaskWriteProt5 [
    /// Reserved
    Reserved OFFSET(27) NUMBITS(5) [],
    /// Write Protection of Enable Software Mode Reset of RTC
    WpEnRstRTC OFFSET(26) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of Ethernet PHY
    WpEnRstEthernetPHY OFFSET(25) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of Caliptra SS
    WpEnRstCptraSS OFFSET(24) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of VGA clock domain
    WpEnRstVgaClk OFFSET(23) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of VGA SLIV clock domain
    WpEnRstVgaSLIVClk OFFSET(22) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of USB2D
    WpEnRstUSB2D OFFSET(21) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of USB2UARTD
    WpEnRstUSB2UARTD OFFSET(20) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of USB2C
    WpEnRstUSB2C OFFSET(19) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of USB2UARTC
    WpEnRstUSB2UARTC OFFSET(18) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of UHCI
    WpEnRstUHCI OFFSET(17) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C15
    WpEnRstI2C15 OFFSET(16) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C14
    WpEnRstI2C14 OFFSET(15) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C13
    WpEnRstI2C13 OFFSET(14) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C12
    WpEnRstI2C12 OFFSET(13) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C11
    WpEnRstI2C11 OFFSET(12) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C10
    WpEnRstI2C10 OFFSET(11) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C9
    WpEnRstI2C9 OFFSET(10) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C8
    WpEnRstI2C8 OFFSET(9) NUMBITS(1) [],
    /// Enable Write Protection of reset Software Mode Reset I2C I2C7
    WpEnRstI2C7 OFFSET(8) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C6
    WpEnRstI2C6 OFFSET(7) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C5
    WpEnRstI2C5 OFFSET(6) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C4
    WpEnRstI2C4 OFFSET(5) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C3
    WpEnRstI2C3 OFFSET(4) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C2
    WpEnRstI2C2 OFFSET(3) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of MDC/I2C1
    WpEnRstI2C1 OFFSET(2) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C0
    WpEnRstI2C0 OFFSET(1) NUMBITS(1) [],
    /// Write Protection of Enable Software Mode Reset of I2C Global
    WpEnRstI2CGlobal OFFSET(0) NUMBITS(1) []
],
WriteProt [
    /// Eanble Write Protection of WDT7C
    EanbleWrProtOfWDT7C OFFSET(31) NUMBITS(1) [],
    /// Reserved
    Reserved OFFSET(30) NUMBITS(1) [],
    /// Eanble Write Protection of WDT74
    EanbleWrProtOfWDT74 OFFSET(29) NUMBITS(1) [],
    /// Eanble Write Protection of WDT70
    EanbleWrProtOfWDT70 OFFSET(28) NUMBITS(1) [],
    /// Eanble Write Protection of WDT6C
    EanbleWrProtOfWDT6C OFFSET(27) NUMBITS(1) [],
    /// Eanble Write Protection of WDT68
    EanbleWrProtOfWDT68 OFFSET(26) NUMBITS(1) [],
    /// Eanble Write Protection of WDT64
    EanbleWrProtOfWDT64 OFFSET(25) NUMBITS(1) [],
    /// Eanble Write Protection of WDT60
    EanbleWrProtOfWDT60 OFFSET(24) NUMBITS(1) [],
    /// Eanble Write Protection of WDT5C
    EanbleWrProtOfWDT5C OFFSET(23) NUMBITS(1) [],
    /// Eanble Write Protection of WDT58
    EanbleWrProtOfWDT58 OFFSET(22) NUMBITS(1) [],
    /// Eanble Write Protection of WDT54
    EanbleWrProtOfWDT54 OFFSET(21) NUMBITS(1) [],
    /// Eanble Write Protection of WDT50
    EanbleWrProtOfWDT50 OFFSET(20) NUMBITS(1) [],
    /// Eanble Write Protection of WDT4C
    EanbleWrProtOfWDT4C OFFSET(19) NUMBITS(1) [],
    /// Eanble Write Protection of WDT48
    EanbleWrProtOfWDT48 OFFSET(18) NUMBITS(1) [],
    /// Eanble Write Protection of WDT44
    EanbleWrProtOfWDT44 OFFSET(17) NUMBITS(1) [],
    /// Eanble Write Protection of WDT40
    EanbleWrProtOfWDT40 OFFSET(16) NUMBITS(1) [],
    /// Eanble Write Protection of WDT3C
    EanbleWrProtOfWDT3C OFFSET(15) NUMBITS(1) [],
    /// Eanble Write Protection of WDT38
    EanbleWrProtOfWDT38 OFFSET(14) NUMBITS(1) [],
    /// Eanble Write Protection of WDT34
    EanbleWrProtOfWDT34 OFFSET(13) NUMBITS(1) [],
    /// Eanble Write Protection of WDT30
    EanbleWrProtOfWDT30 OFFSET(12) NUMBITS(1) [],
    /// Eanble Write Protection of WDT2C
    EanbleWrProtOfWDT2C OFFSET(11) NUMBITS(1) [],
    /// Eanble Write Protection of WDT28
    EanbleWrProtOfWDT28 OFFSET(10) NUMBITS(1) [],
    /// Eanble Write Protection of WDT24
    EanbleWrProtOfWDT24 OFFSET(9) NUMBITS(1) [],
    /// Eanble Write Protection of WDT20
    EanbleWrProtOfWDT20 OFFSET(8) NUMBITS(1) [],
    /// Eanble Write Protection of WDT1C
    EanbleWrProtOfWDT1C OFFSET(7) NUMBITS(1) [],
    /// Eanble Write Protection of WDT18
    EanbleWrProtOfWDT18 OFFSET(6) NUMBITS(1) [],
    /// Eanble Write Protection of WDT14
    EanbleWrProtOfWDT14 OFFSET(5) NUMBITS(1) [],
    /// Eanble Write Protection of WDT10
    EanbleWrProtOfWDT10 OFFSET(4) NUMBITS(1) [],
    /// Eanble Write Protection of WDT0C
    EanbleWrProtOfWDT0C OFFSET(3) NUMBITS(1) [],
    /// Eanble Write Protection of WDT08
    EanbleWrProtOfWDT08 OFFSET(2) NUMBITS(1) [],
    /// Eanble Write Protection of WDT04
    EanbleWrProtOfWDT04 OFFSET(1) NUMBITS(1) [],
    /// Eanble Write Protection of WDT00
    EanbleWrProtOfWDT00 OFFSET(0) NUMBITS(1) []
]
];
