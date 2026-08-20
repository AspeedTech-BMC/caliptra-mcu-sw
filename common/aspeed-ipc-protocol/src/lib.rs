// Licensed under the Apache-2.0 license
//
// Wire format for the ASPEED SS-MCU shared-memory IPC protocol: `net_boot`
// (SS-MCU <-> NetMCU).
//
// This crate only defines the wire layout — message types, offsets, sizes
// (see `net_boot`). It does not know the shared-memory base address or how
// the doorbell hardware works: that transport (doorbell registers + the
// shared buffer + the send/poll/call logic and its memory fences) lives in
// `caliptra_mcu_romtime::aspeed_ipc::AspeedIpcCtrl`, with the shared-memory base address
// and size supplied by each platform's `config` crate.
//
// See `aspeed_ipc_shm.h` in this crate for the C-side mirror of this
// layout, for the NetMCU team.

#![no_std]

pub mod net_boot;

pub use net_boot::{
    ChunkAck, Finalize, FinalizeAck, ImageChunkHeader, ImageDownloadRequest, ImageMetadataRequest,
    ImageMetadataResponse, InitiateBootRequest, InitiateBootResponse, VendorSpecific,
};
