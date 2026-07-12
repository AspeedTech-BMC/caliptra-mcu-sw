// Licensed under the Apache-2.0 license

//! Platform-neutral DMA drivers implementing the
//! `capsules_runtime::dma::hil::Dma` HAL.

#![cfg_attr(target_arch = "riscv32", no_std)]

pub mod nodma;
