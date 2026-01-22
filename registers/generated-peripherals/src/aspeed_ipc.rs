// Licensed under the Apache-2.0 license

pub const ASPEED_IPC_DATA_SIZE_PER_CH: u32 = 0x1C;
pub mod bits {
    //! Types that represent individual registers (bitfields).
    use tock_registers::register_bitfields;
    register_bitfields! {
        u32,
            pub Ctrl [
                Enable OFFSET(0) NUMBITS(1) [],
                Lock OFFSET(31) NUMBITS(1) [],
            ],
            pub Ch_Ctrl [
                Ch0 OFFSET(0) NUMBITS(1) [],
                Ch1 OFFSET(1) NUMBITS(1) [],
                Ch2 OFFSET(2) NUMBITS(1) [],
                Ch3 OFFSET(3) NUMBITS(1) [],
            ],
    }
}
pub mod rx_regs {
    //! Types that represent registers.
    use tock_registers::register_structs;
    register_structs! {
        pub IpcRxCtrl {
            (0x000 => pub trig: tock_registers::registers::ReadOnly<u32, crate::aspeed_ipc::bits::Ch_Ctrl::Register>),
            (0x004 => pub enable: tock_registers::registers::ReadWrite<u32, crate::aspeed_ipc::bits::Ch_Ctrl::Register>),
            (0x008 => pub status: tock_registers::registers::ReadWrite<u32, crate::aspeed_ipc::bits::Ch_Ctrl::Register>),
            (0x00c => pub ctrl: tock_registers::registers::ReadWrite<u32, crate::aspeed_ipc::bits::Ctrl::Register>),
            (0x010 => pub data: [tock_registers::registers::ReadOnly<u32>; 28]),
            (0x080 => @END),
        }
    }
}

pub mod tx_regs {
    //! Types that represent registers.
    use tock_registers::register_structs;
    register_structs! {
        pub IpcTxCtrl {
            (0x000 => pub trig: tock_registers::registers::ReadWrite<u32, crate::aspeed_ipc::bits::Ch_Ctrl::Register>),
            (0x004 => pub enable: tock_registers::registers::ReadOnly<u32, crate::aspeed_ipc::bits::Ch_Ctrl::Register>),
            (0x008 => pub status: tock_registers::registers::ReadOnly<u32, crate::aspeed_ipc::bits::Ch_Ctrl::Register>),
            (0x00c => pub ctrl: tock_registers::registers::ReadWrite<u32, crate::aspeed_ipc::bits::Ctrl::Register>),
            (0x010 => pub data: [tock_registers::registers::ReadWrite<u32>; 28]),
            (0x080 => @END),
        }
    }
}
