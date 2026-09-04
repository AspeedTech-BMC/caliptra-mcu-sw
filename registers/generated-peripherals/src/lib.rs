// Licensed under the Apache-2.0 license.
#![no_std]

pub mod aspeed_dramc;
pub mod aspeed_edaf;
pub mod aspeed_hyperram;
pub mod aspeed_ipc;
pub mod aspeed_otp;
pub mod aspeed_soc;
pub mod aspeed_spi;
pub mod aspeed_wdt;
pub mod uart16550;

#[cfg(feature = "ast2705-register")]
pub mod ast2705;

#[cfg(feature = "ast1040-register")]
pub mod ast1040;
