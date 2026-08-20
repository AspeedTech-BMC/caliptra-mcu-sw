// Licensed under the Apache-2.0 license

/* MCU-SS internal used address */
pub const ASPEED_SOC_ADDR: u32 = 0x1000_0000;

/* CPU die peripheral address */
pub const ASPEED_SDRAMC_ADDR: u32 = 0x32c0_0000;
pub const ASPEED_SCU0_ADDR: u32 = 0x32c0_2000;
pub const ASPEED_DRAM_PHY_ADDR: u32 = 0x3300_0000;

/* IO die peripheral address */
pub const ASPEED_FMC_CTRL_ADDR: u32 = 0x3400_0000;
pub const ASPEED_SCU1_ADDR: u32 = 0x34c0_2000;
const ASPEED_OTP_ADDR: u32 = 0x34c0_7000;
const ASPEED_WDT0_ADDR: u32 = 0x34c3_7000;
pub const ASPEED_IPC_TX_ADDR: u32 = 0x34c3_9880;
pub const ASPEED_IPC_RX_ADDR: u32 = 0x34c3_9800;
pub const ASPEED_IPC_SHM_ADDR: u32 = 0x34b9_f000;
pub const ASPEED_IPC_SHM_SIZE: u32 = 0x0000_1000;
pub const ASPEED_UART_ADDR: u32 = 0x34c3_3b00;
pub const ASPEED_FMC_MEM_ADDR: u32 = 0x4000_0000;
