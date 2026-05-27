// Licensed under the Apache-2.0 license

/* MCU-SS internal used address */
const ASPEED_SOC_ADDR: u32 = 0x1000_0000;

/* CPU die peripheral address */
pub const ASPEED_SDRAMC_ADDR: u32 = 0x32c0_0000;
pub const ASPEED_DRAM_PHY_ADDR: u32 = 0x3300_0000;

/* IO die peripheral address */
const ASPEED_SCU1_ADDR: u32 = 0x34c0_2000;
const ASPEED_OTP_ADDR: u32 = 0x34c0_7000;
const ASPEED_WDT0_ADDR: u32 = 0x34c3_7000;
pub const ASPEED_IPC_TX_ADDR: u32 = 0x34c3_9880;
pub const ASPEED_IPC_RX_ADDR: u32 = 0x34c3_9800;
pub const ASPEED_UART_ADDR: u32 = 0x34c3_3b00;
