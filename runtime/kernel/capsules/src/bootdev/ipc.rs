// Licensed under the Apache-2.0 license

use super::hil::{BootDevType, ImageFlash};
use aspeed_ipc_protocol::net_boot::{
    ChunkAck, ImageChunkHeader, ImageDownloadRequest, ImageMetadataRequest, ImageMetadataResponse,
    NET_BOOT_CHUNK_ACK_FLAG_READY_FOR_NEXT, NET_BOOT_MSG_CHUNK_ACK, NET_BOOT_MSG_IMAGE_CHUNK,
    NET_BOOT_MSG_IMAGE_DOWNLOAD_REQ, NET_BOOT_MSG_IMAGE_METADATA_REQ,
    NET_BOOT_MSG_IMAGE_METADATA_RESP, NET_BOOT_STATUS_SUCCESS,
};
use caliptra_mcu_aspeed_ipc::AspeedIpcCtrl;
use kernel::ErrorCode;
use zerocopy::{FromBytes, IntoBytes};

impl ImageFlash for AspeedIpcCtrl {
    fn ready(&self) -> bool {
        self.is_rx_ready()
    }

    fn dev_type(&self) -> BootDevType {
        BootDevType::Ipc
    }

    fn load_image(&self, image_id: u32, dst_addr: u32) -> Result<u32, ErrorCode> {
        const NET_BOOT_MAX_POLLS: usize = 1_000_000_000;
        const HEADER_SIZE: usize = core::mem::size_of::<ImageChunkHeader>();

        // 1. Query metadata to learn image_size.
        let req = ImageMetadataRequest {
            message_type: NET_BOOT_MSG_IMAGE_METADATA_REQ,
            reserved: [0; 3],
            firmware_id: image_id,
        };
        let mut resp_buf = [0u8; core::mem::size_of::<ImageMetadataResponse>()];
        self.call(req.as_bytes(), &mut resp_buf, NET_BOOT_MAX_POLLS)
            .map_err(|_| ErrorCode::FAIL)?;
        let resp =
            ImageMetadataResponse::read_from_bytes(&resp_buf).map_err(|_| ErrorCode::FAIL)?;
        if resp.message_type != NET_BOOT_MSG_IMAGE_METADATA_RESP
            || resp.status != NET_BOOT_STATUS_SUCCESS
        {
            return Err(ErrorCode::FAIL);
        }
        let image_size = resp.image_size;

        // 2. Start the transfer so the peer begins pushing chunks.
        let download_req = ImageDownloadRequest {
            message_type: NET_BOOT_MSG_IMAGE_DOWNLOAD_REQ,
            reserved: [0; 3],
            firmware_id: image_id,
        };
        self.send(download_req.as_bytes())
            .map_err(|_| ErrorCode::FAIL)?;

        caliptra_mcu_romtime::println!("[mcu-runtime-ipc] Start to download image");

        // 3. Receive/copy/ack chunks until image_size bytes are copied.
        let mut copied: u32 = 0;
        while copied < image_size {
            let mut header_buf = [0u8; HEADER_SIZE];
            self.recv(&mut header_buf, NET_BOOT_MAX_POLLS)
                .map_err(|_| ErrorCode::FAIL)?;
            let header =
                ImageChunkHeader::read_from_bytes(&header_buf).map_err(|_| ErrorCode::FAIL)?;
            if header.message_type != NET_BOOT_MSG_IMAGE_CHUNK
                || header.status != NET_BOOT_STATUS_SUCCESS
            {
                caliptra_mcu_romtime::println!(
                    "[mcu-runtime-ipc] message_type: {:?}",
                    header.message_type
                );
                caliptra_mcu_romtime::println!("[mcu-runtime-ipc] status: {:?}", header.status);
                return Err(ErrorCode::FAIL);
            }

            let chunk_size = header.chunk_size;
            // caliptra_mcu_romtime::println!("[mcu-runtime-ipc] chunk_size: {:?}", chunk_size);
            if chunk_size == 0
                || copied
                    .checked_add(chunk_size)
                    .is_none_or(|c| c > image_size)
                || (HEADER_SIZE as u64 + chunk_size as u64) > self.shm_size
            {
                caliptra_mcu_romtime::println!("[mcu-runtime-ipc] chunk_size invalid");
                return Err(ErrorCode::FAIL);
            }

            self.read_at_to_addr(
                HEADER_SIZE,
                dst_addr as usize + copied as usize,
                chunk_size as usize,
            )
            .map_err(|_| ErrorCode::FAIL)?;

            // caliptra_mcu_romtime::println!("[mcu-runtime-ipc] ChunkAck");
            copied += chunk_size;

            let ack = ChunkAck {
                message_type: NET_BOOT_MSG_CHUNK_ACK,
                reserved0: [0; 3],
                firmware_id: image_id,
                sequence_number: header.sequence_number,
                reserved1: 0,
                flags: NET_BOOT_CHUNK_ACK_FLAG_READY_FOR_NEXT,
            };
            self.send(ack.as_bytes()).map_err(|_| ErrorCode::FAIL)?;
        }

        Ok(image_size)
    }

    fn map_to_axi(&self, addr: u64) -> u64 {
        addr
    }

    fn map_from_axi(&self, addr: u64) -> u64 {
        addr
    }
}
