// Licensed under the Apache-2.0 license.
#![no_std]

pub mod aspeed_otp;
pub mod aspeed_soc;
pub mod aspeed_wdt;
pub mod uart16550;

#[cfg(feature = "ast2705-register")]
pub mod ast2705;
