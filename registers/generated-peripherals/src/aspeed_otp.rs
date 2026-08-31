// Licensed under the Apache-2.0 license.
//

// use tock_registers::registers::ReadWrite;

pub const ASPEED_OTP_CTRL_ADDR: u32 = 0x34c0_7000;
pub mod bits {
    //! Types that represent individual registers (bitfields).
    use tock_registers::register_bitfields;
    register_bitfields! {
        u32,
            pub OtpEnable [
                EnableMagic OFFSET(0) NUMBITS(32) [],
            ],
            pub OtpCmd [
                Command OFFSET(0) NUMBITS(32) [],
            ],
            pub OtpMasterPID [
                PID OFFSET(0) NUMBITS(6) [],
                Psecure OFFSET(9) NUMBITS(2) [],
                PufSel OFFSET(12) NUMBITS(2) [],
            ],
            pub RngBs [
                RngBs OFFSET(0) NUMBITS(3) [],
            ],
    }
}
pub mod regs {
    //! Types that represent registers.
    use tock_registers::register_structs;
    register_structs! {
        pub OtpCtrl {
            (0x0 => pub otp_enable: tock_registers::registers::ReadWrite<u32, crate::aspeed_otp::bits::OtpEnable::Register>),
            (0x4 => _reserved0),
            (0x64 => pub otp_cmd_m3: tock_registers::registers::ReadWrite<u32>),
            (0x68 => pub otp_wdata_m3_0: tock_registers::registers::WriteOnly<u32>),
            (0x6c => pub otp_wdata_m3_1: tock_registers::registers::WriteOnly<u32>),
            (0x70 => pub otp_wdata_m3_2: tock_registers::registers::WriteOnly<u32>),
            (0x74 => pub otp_wdata_m3_3: tock_registers::registers::WriteOnly<u32>),
            (0x78 => pub otp_status_m3: tock_registers::registers::ReadOnly<u32>),
            (0x7c => pub otp_addr_m3: tock_registers::registers::ReadWrite<u32>),
            (0x80 => pub otp_rdata_m3: tock_registers::registers::ReadOnly<u32>),
            (0x84 => _reserved1),
            (0xd0 => pub otp_master_pid: tock_registers::registers::ReadWrite<u32, crate::aspeed_otp::bits::OtpMasterPID::Register>),
            (0xd4 => _reserved2),
            (0x1f4 => pub otp_cmd_s2c: tock_registers::registers::ReadWrite<u32>),
            (0x1f8 => _reserved3),
            (0x620 => pub rng_bs20: tock_registers::registers::ReadWrite<u32>),
            (0x624 => pub rng_bs24: tock_registers::registers::ReadWrite<u32>),
            (0x628 => pub rng_bs28: tock_registers::registers::ReadWrite<u32>),
            (0x62c => pub rng_bs2c: tock_registers::registers::ReadWrite<u32>),
            (0x630 => pub rng_bs30: tock_registers::registers::ReadWrite<u32>),
            (0x634 => pub rng_bs34: tock_registers::registers::ReadWrite<u32>),
            (0x638 => pub rng_bs38: tock_registers::registers::ReadWrite<u32>),
            (0x63c => pub rng_bs3c: tock_registers::registers::ReadWrite<u32>),
            (0x640 => @END),
        }
    }
}
