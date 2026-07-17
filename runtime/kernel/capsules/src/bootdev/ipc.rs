// Licensed under the Apache-2.0 license

use super::hil::{BootDevType, ImageFlash};
use caliptra_mcu_aspeed_ipc::AspeedIpcCtrl;
use kernel::ErrorCode;

const SS_AXI_DRAM_BASE: u64 = 0xc000_0000;
const SS_AXI_SRAM_OFFSET: u64 = 0x2000_0000;

impl ImageFlash for AspeedIpcCtrl {
    fn ready(&self) -> bool {
        self.is_rx_ready()
    }

    fn dev_type(&self) -> BootDevType {
        BootDevType::Ipc
    }

    fn load_image(&self, image_id: u32, dst_addr: u32) -> Result<u32, ErrorCode> {
        self.send(0, image_id, dst_addr)
            .map_err(|_| ErrorCode::FAIL)?;
        let mut status = 0u32;
        let mut img_size = 0u32;
        let mut svn = 0u32;
        let mut load = 0u32;
        loop {
            if self
                .receive(&mut status, &mut img_size, &mut svn, &mut load)
                .is_ok()
            {
                break;
            }
        }
        Ok(img_size)
    }

    fn map_to_axi(&self, addr: u64) -> u64 {
        if addr < SS_AXI_DRAM_BASE {
            // Addresses below the SRAM window would underflow; clamp
            // instead of wrapping to a bogus high address.
            addr.saturating_sub(SS_AXI_SRAM_OFFSET)
        } else {
            self.target_dram(addr - SS_AXI_DRAM_BASE)
        }
    }

    fn map_from_axi(&self, addr: u64) -> u64 {
        addr
    }
}
