// Licensed under the Apache-2.0 license

use tock_registers::registers::{Aliased, ReadWrite};
use tock_registers::{register_bitfields, register_structs};

register_structs! {
    /// uart
    pub Uart16550Registers {
        /// Receiving Buffer Register (DLAB = 0)
        (0x000 => pub rbr_thr: Aliased<u32, RBR::Register, THR::Register>),
        /// Interrupt Enable Register (DLAB = 0)
        (0x004 => pub ier: ReadWrite<u32, IER::Register>),
        /// Interrupt Identity Register
        (0x008 => pub iir_fcr: Aliased<u32, IIR::Register, FCR::Register>),
        /// Line Control Register
        (0x00C => pub lcr: ReadWrite<u32, LCR::Register>),
        /// Modem Control Register
        (0x010 => pub mcr: ReadWrite<u32, MCR::Register>),
        /// Line Status Register
        (0x014 => pub lsr: ReadWrite<u32, LSR::Register>),
        /// Modem Status Register
        (0x018 => pub msr: ReadWrite<u32, MSR::Register>),
        /// Scratch Register
        (0x01C => pub scr: ReadWrite<u32, SCR::Register>),
        /// Divisor Latch Low (DLAB = 1)
        (0x020 => pub dll: ReadWrite<u32, DLL::Register>),
        /// Divisor Latch High (DLAB = 1)
        (0x024 => pub dlh: ReadWrite<u32, DLH::Register>),
        (0x028 => @END),
    }
}

register_bitfields![u32,
pub RBR [
    /// Reserved (0)
    Reserved OFFSET(8) NUMBITS(24) [],
    /// Receiving Buffer Register
    Data OFFSET(0) NUMBITS(8) [],
],
pub THR [
    /// Reserved (0)
    Reserved1 OFFSET(8) NUMBITS(24) [],
    /// Transmit Holding Register
    Data OFFSET(0) NUMBITS(8) []
],
pub IER [
    /// Reserved (0)
    Reserved OFFSET(8) NUMBITS(24) [],
    /// Reserved (0)
    Reserved1 OFFSET(4) NUMBITS(4) [],
    /// Enable Modem Status Interrupt
    EDSSI OFFSET(3) NUMBITS(1) [],
    /// Enable Receiver Line Status Interrupt
    ELSI OFFSET(2) NUMBITS(1) [],
    /// Enable Transmitter Holding Register Empty Interrupt
    ETBEI OFFSET(1) NUMBITS(1) [],
    /// Enable Received Data Available Interrupt
    ERBFI OFFSET(0) NUMBITS(1) []
],
pub IIR [
    /// Reserved (0)
    Reserved OFFSET(8) NUMBITS(24) [],
    /// FIFO-Enabled Bits
    FIFO OFFSET(6) NUMBITS(2) [
        FIFODisabled = 0,
        FIFOsEnabled = 1,
    ],
    /// Reserved (0)
    Reserved1 OFFSET(4) NUMBITS(2) [],
    /// Interrupt Decoding Table
    Identification OFFSET(1) NUMBITS(3) [
        ModemStatusChange = 0,
        ThrEmpty = 1,
        RxDataAvailable = 2,
        RxLineStatus = 3,
        Timeout = 6,
    ],
    /// When it's "1", no interrupt is pending.
    Pending OFFSET(0) NUMBITS(1) [
        Pending = 0,
        NotPending = 1,
    ]
],
pub FCR [
    /// Reserved (0)
    Reserved OFFSET(8) NUMBITS(24) [],
    /// Define the Receiver FIFO Interrupt trigger level.
    DefineTheRxrFIFOINTTriggerLevel OFFSET(6) NUMBITS(2) [
        Bytes1 = 0,
        Bytes4 = 1,
        Bytes8 = 2,
        Bytes14 = 3,
    ],
    /// Reserved (0)
    Reserved1 OFFSET(3) NUMBITS(3) [],
    /// Transmit FIFO Reset
    TxFIFOClr OFFSET(2) NUMBITS(1) [],
    /// Receive FIFO Reset
    RxFIFOClr OFFSET(1) NUMBITS(1) [],
    /// Enable UART FIFO
    EnableFIFO OFFSET(0) NUMBITS(1) []
],
pub LCR [
    /// Reserved (0)
    Reserved OFFSET(8) NUMBITS(24) [],
    /// Divisor latch access bit
    DivLatch OFFSET(7) NUMBITS(1) [],
    /// Break Control bit.
    BreakCtrlBit OFFSET(6) NUMBITS(1) [],
    /// Reserved (0)
    Reserved1 OFFSET(5) NUMBITS(1) [],
    /// Parity mode selection
    EPS OFFSET(4) NUMBITS(1) [],
    /// Enable parity bit
    PEN OFFSET(3) NUMBITS(1) [],
    /// Number of stop bits transmitted
    STOP OFFSET(2) NUMBITS(1) [
        One = 0,
        OneHalfTwo = 1,
    ],
    /// Select number of bits per character
    DataWordLen OFFSET(0) NUMBITS(2) [
        Bits5 = 0,
        Bits6 = 1,
        Bits7 = 2,
        Bits8 = 3,
    ]
],
pub MCR [
    /// Reserved (0)
    Reserved OFFSET(5) NUMBITS(27) [],
    /// Loopback mode.
    LoopbackMode OFFSET(4) NUMBITS(1) [],
    /// Out2.
    AuxOut2 OFFSET(3) NUMBITS(1) [],
    /// Out1.
    AuxOut1 OFFSET(2) NUMBITS(1) [],
    /// Request To Send (nRTS) signal control.
    ReqToSend OFFSET(1) NUMBITS(1) [],
    /// Data Terminal Ready (nDTR) signal control.
    DataTerminalReady OFFSET(0) NUMBITS(1) []
],
pub LSR [
    /// Reserved (0)
    Reserved OFFSET(8) NUMBITS(24) [],
    /// Error in Receiver FIFO (Read clear)
    ErrInRxrFIFOReadClear OFFSET(7) NUMBITS(1) [],
    /// Transmitter empty
    TxterEmpty OFFSET(6) NUMBITS(1) [],
    /// Transmitter holding register empty
    THRE OFFSET(5) NUMBITS(1) [],
    /// Break interrupt (Read clear)
    BI OFFSET(4) NUMBITS(1) [],
    /// Framing error (Read clear)
    FE OFFSET(3) NUMBITS(1) [],
    /// Parity error (Read clear)
    PE OFFSET(2) NUMBITS(1) [],
    /// Overrun error (Read clear)
    OE OFFSET(1) NUMBITS(1) [],
    /// Data ready
    DR OFFSET(0) NUMBITS(1) []
],
pub MSR [
    /// Reserved (0)
    Reserved OFFSET(8) NUMBITS(24) [],
    /// Complement of the nDCD input or equals to Out2(MCR[3]) in loopback mode.
    ComplementOfTheNDCDOut2MCR3 OFFSET(7) NUMBITS(1) [],
    /// Complement of the nRI input or equals to Out1(MCR[2]) in loopback mode.
    ComplementOfTheNRIOut1MCR2 OFFSET(6) NUMBITS(1) [],
    /// Complement of the nDSR input or equals to nDTR(MCR[0]) in loopback mode.
    ComplementOfTheNDSRNDTRMCR0 OFFSET(5) NUMBITS(1) [],
    /// Complement of the nCTS input or equals to nRTS(MCR[1]) in loopback mode.
    ComplementOfTheNCTSNRTSMCR1 OFFSET(4) NUMBITS(1) [],
    /// Delta Data Carrier Detect (DDCD) indicator (Read clear)
    DeltaDataCarrierDetect OFFSET(3) NUMBITS(1) [],
    /// Trailing Edge of Ring Indicator (TERI) detector (Read clear)
    TrailingEdgeOfRingIndicator OFFSET(2) NUMBITS(1) [],
    /// Delta Data Set Ready (DDSR) indicator (Read clear)
    DeltaDataSetReady OFFSET(1) NUMBITS(1) [],
    /// Delta Clear To Send (DCTS) indicator (Read clear)
    DeltaClearToSend OFFSET(0) NUMBITS(1) []
],
pub SCR [
    /// Reserved (0)
    Reserved OFFSET(8) NUMBITS(24) [],
    /// Scratch bits
    ScratchBits OFFSET(0) NUMBITS(8) []
],
pub DLL [
    /// Reserved.
    Reserved OFFSET(8) NUMBITS(24) [],
    /// The LSB of the baud-rate divisor latch.
    LSBDivLatch OFFSET(0) NUMBITS(8) []
],
pub DLH [
    /// Reserved.
    Reserved OFFSET(8) NUMBITS(24) [],
    /// The MSB of the baud-rate divisor latch.
    MSBDivLatch OFFSET(0) NUMBITS(8) []
]
];
