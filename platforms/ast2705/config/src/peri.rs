// Licensed under the Apache-2.0 license

/* MCU-SS internal used address */
pub const ASPEED_SOC_ADDR: u32 = 0x1000_0000;

/* CPU die peripheral address */
pub const ASPEED_SDRAMC_ADDR: u32 = 0x32c0_0000;
pub const ASPEED_SCU0_ADDR: u32 = 0x32c0_2000;
pub const ASPEED_DRAM_PHY_ADDR: u32 = 0x3300_0000;

/* IO die peripheral address */
pub const ASPEED_FMC_CTRL_ADDR: u32 = 0x3400_0000;
pub const ASPEED_SPI0_CTRL_ADDR: u32 = 0x3401_0000;
pub const ASPEED_SPI1_CTRL_ADDR: u32 = 0x3402_0000;
pub const ASPEED_SPI2_CTRL_ADDR: u32 = 0x3403_0000;
pub const ASPEED_SCU1_ADDR: u32 = 0x34c0_2000;
pub const ASPEED_IPC_TX_ADDR: u32 = 0x34c3_9880;
pub const ASPEED_IPC_RX_ADDR: u32 = 0x34c3_9800;
pub const ASPEED_IPC_SHM_ADDR: u32 = 0x34b9_f000;
pub const ASPEED_IPC_SHM_SIZE: u32 = 0x0000_1000;
pub const ASPEED_UART_ADDR: u32 = 0x34c3_3b00;
pub const ASPEED_FMC_MEM_ADDR: u32 = 0x4000_0000;
pub const ASPEED_FMC_FLASH_SIZE_PER_CS: u32 = 0x0400_0000; // 64 MB
pub const ASPEED_SHARED_AHBC_WINDOW_BASE: u32 = 0x3000_0000;
pub const ASPEED_SPI0_FLASH_SIZE_PER_CS: u32 = 0x0100_0000; // 16 MB
pub const ASPEED_SPI0_AHB_MSB_OFFSET: u8 = 0x18; // AXI 0x1_8000_0000
pub const ASPEED_SPI1_FLASH_SIZE_PER_CS: u32 = 0x0100_0000; // 16 MB
pub const ASPEED_SPI1_AHB_MSB_OFFSET: u8 = 0x20; // AXI 0x2_0000_0000
pub const ASPEED_SPI2_FLASH_SIZE_PER_CS: u32 = 0x0080_0000; // 8 MB
pub const ASPEED_SPI2_AHB_MSB_OFFSET: u8 = 0x28; // AXI 0x2_8000_0000

/* EDAF bridge address */
pub const EDAF_BRIDGE_GLOBAL_ADDR: u32 = 0x340e_c100;
pub const EDAF_BRIDGE0_ADDR: u32 = 0x34c3_8000;
pub const EDAF_BRIDGE1_ADDR: u32 = 0x34c3_8200;

/* EDAF bridge configuration */
pub const EDAF_BRIDGE0_ENABLE: bool = true;
pub const EDAF_BRIDGE0_DDR_MODE: bool = false;
pub const EDAF_BRIDGE0_ERASE_WITH_ONE: bool = true;
pub const EDAF_BRIDGE0_ESPI_BASE: u32 = 0x34c0_5000;
pub const EDAF_BRIDGE0_SPI_MEM_BASE: u64 = 0x0000_0001_8000_0000;
pub const EDAF_BRIDGE0_DDR_MEM_BASE: u64 = 0x0000_0004_2800_0000;
pub const EDAF_BRIDGE0_CTL_BASE: Option<u64> = Some(0x0000_0000_1401_0000);
pub const EDAF_BRIDGE0_CMD_READ: Option<u32> = Some(0x0013_0401);
pub const EDAF_BRIDGE0_CMD_WRITE_ENABLE: Option<u32> = Some(0x0006_041e);
pub const EDAF_BRIDGE0_CMD_WRITE: Option<u32> = Some(0x0012_0403);
pub const EDAF_BRIDGE0_CMD_READ_STATUS: Option<u32> = Some(0x0005_0415);
pub const EDAF_BRIDGE0_CMD_ERASE_4K: Option<u32> = Some(0x0021_040e);
pub const EDAF_BRIDGE0_CMD_ERASE_32K: Option<u32> = Some(0x005c_040e);
pub const EDAF_BRIDGE0_CMD_ERASE_64K: Option<u32> = Some(0x00dc_040e);

pub const EDAF_BRIDGE1_ENABLE: bool = true;
pub const EDAF_BRIDGE1_DDR_MODE: bool = false;
pub const EDAF_BRIDGE1_ERASE_WITH_ONE: bool = false;
pub const EDAF_BRIDGE1_ESPI_BASE: u32 = 0x34c0_6000;
pub const EDAF_BRIDGE1_MEM_BASE: u64 = 0x0000_0002_0000_0000;
pub const EDAF_BRIDGE1_CTL_BASE: Option<u64> = Some(0x0000_0000_1402_0000);
pub const EDAF_BRIDGE1_CMD_READ: Option<u32> = Some(0x0013_0401);
pub const EDAF_BRIDGE1_CMD_WRITE_ENABLE: Option<u32> = Some(0x0006_041e);
pub const EDAF_BRIDGE1_CMD_WRITE: Option<u32> = Some(0x0012_0402);
pub const EDAF_BRIDGE1_CMD_READ_STATUS: Option<u32> = Some(0x0005_0415);
pub const EDAF_BRIDGE1_CMD_ERASE_4K: Option<u32> = Some(0x0021_040e);
pub const EDAF_BRIDGE1_CMD_ERASE_32K: Option<u32> = Some(0x005c_040e);
pub const EDAF_BRIDGE1_CMD_ERASE_64K: Option<u32> = Some(0x00dc_040e);
