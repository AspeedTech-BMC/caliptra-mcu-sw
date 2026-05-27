// Licensed under the Apache-2.0 license

//! FLSH image-package access over the boot-device syscall.
//!
//! Mirror of `flash_client.rs` for the bootdev read path: `BootDev::read`
//! has the same shape as `SpiFlash::read`, and loading an image is the
//! same buffer-read + DMA-syscall composition — the bootdev capsule only
//! abstracts which boot device serves the reads.

use caliptra_mcu_flash_image::{FlashHeader, ImageHeader};
use caliptra_mcu_libsyscall_caliptra::bootdev::BootDev;
use caliptra_mcu_libsyscall_caliptra::dma::{
    AXIAddr, DMAMapping, DMASource, DMATransaction, DMA as DMASyscall,
};
use caliptra_mcu_libtock_platform::ErrorCode;
use zerocopy::FromBytes;

/// This is the size of the buffer used for DMA transfers.
const MAX_DMA_TRANSFER_SIZE: usize = 4 * 1024;

const FLASH_HEADER_OFFSET: usize = 0;

/// Read the FLSH package header into `header`.
pub async fn bootdev_read_header(
    boot_dev: &BootDev,
    header: &mut [u8; core::mem::size_of::<FlashHeader>()],
) -> Result<(), ErrorCode> {
    boot_dev
        .read(
            FLASH_HEADER_OFFSET,
            core::mem::size_of::<FlashHeader>(),
            header,
        )
        .await?;
    Ok(())
}

/// Walk the image headers following the package header and return
/// `(offset, size)` of the image matching `component_id`.
pub async fn bootdev_read_toc(
    boot_dev: &BootDev,
    header: &[u8; core::mem::size_of::<FlashHeader>()],
    component_id: u32,
) -> Result<(u32, u32), ErrorCode> {
    let (header, _) = FlashHeader::ref_from_prefix(header).map_err(|_| ErrorCode::Fail)?;
    for index in 0..header.image_count as usize {
        let flash_offset =
            core::mem::size_of::<FlashHeader>() + index * core::mem::size_of::<ImageHeader>();
        let buffer = &mut [0u8; core::mem::size_of::<ImageHeader>()];
        boot_dev
            .read(flash_offset, core::mem::size_of::<ImageHeader>(), buffer)
            .await?;
        let (image_header, _) =
            ImageHeader::ref_from_prefix(buffer).map_err(|_| ErrorCode::Fail)?;
        if image_header.identifier == component_id {
            return Ok((image_header.offset, image_header.size));
        }
    }

    Err(ErrorCode::Fail)
}

/// Load the image payload (`offset`/`img_size` from the TOC) to
/// `load_address`: chunked bootdev reads into a local buffer, each chunk
/// moved to the destination by the DMA syscall (mirrors
/// `flash_client::flash_load_image`).
///
/// The image's start offset (baked into the FLSH package at build time) is
/// arbitrary, but the physical flash driver's page size matches the bootdev
/// chunk size, so a read landing on a chunk-size boundary costs exactly one
/// page fetch instead of straddling two. The first `boot_dev.read()` below
/// fixes up just the leading, short misaligned remainder; every read after
/// that inherits the alignment since each step advances by exactly
/// `chunk_size`.
pub async fn bootdev_load_image(
    boot_dev: &BootDev,
    load_address: AXIAddr,
    offset: usize,
    img_size: usize,
    dma_mapping: &impl DMAMapping,
) -> Result<(), ErrorCode> {
    let dma_syscall: DMASyscall = DMASyscall::new();
    let chunk_size = boot_dev.get_chunk_size()?;
    let mut remaining_size = img_size;
    let mut current_offset = offset;
    let mut current_address = load_address;

    while remaining_size > 0 {
        // Recomputed from the true current_offset every iteration (not
        // tracked incrementally): this stays correct even if a previous
        // iteration's transfer_size was cut short of a chunk boundary by
        // the .min(MAX_DMA_TRANSFER_SIZE) clamp below, since it re-derives
        // the real misalignment instead of assuming the clamp always
        // reaches the boundary.
        let misalign = current_offset % chunk_size;
        let transfer_size = (chunk_size - misalign).min(remaining_size.min(MAX_DMA_TRANSFER_SIZE));

        let mut buffer = [0; MAX_DMA_TRANSFER_SIZE];
        boot_dev
            .read(current_offset, transfer_size, &mut buffer[..transfer_size])
            .await?;

        let source_address = dma_mapping.mcu_sram_to_mcu_axi(buffer.as_ptr() as u32)?;
        let transaction = DMATransaction {
            byte_count: transfer_size,
            source: DMASource::Address(source_address),
            dest_addr: current_address,
        };
        dma_syscall.xfer(&transaction).await?;

        remaining_size -= transfer_size;
        current_offset += transfer_size;
        current_address += transfer_size as u64;
    }

    Ok(())
}
