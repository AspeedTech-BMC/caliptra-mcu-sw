// Licensed under the Apache-2.0 license.
//
// File: aspeed_ipc.rs
//
// Provides IPC control for AST2705.

#![no_std]

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;
use registers_peripherals::aspeed_ipc::bits::{Ch_Ctrl, Ctrl};
use registers_peripherals::aspeed_ipc::rx_regs::IpcRxCtrl;
use registers_peripherals::aspeed_ipc::tx_regs::IpcTxCtrl;
use registers_peripherals::aspeed_ipc::ASPEED_IPC_DATA_SIZE_PER_CH;

#[derive(Debug, Eq, PartialEq)]
pub enum AspeedIpcApiError {
    TXisBusy,
    RXNotReady,
    RXEmpty,
}

pub struct AspeedIpcCtrl {
    tx_regs: StaticRef<IpcTxCtrl>,
    rx_regs: StaticRef<IpcRxCtrl>,
    ch: tock_registers::fields::Field<u32, Ch_Ctrl::Register>,
    data_offset: usize,
    target_dram_base: u64,
}

impl AspeedIpcCtrl {
    pub const fn new(tx_addr: u32, rx_addr: u32, ch_id: u32, target_dram_base: u64) -> Self {
        let ch = match ch_id {
            0 => Ch_Ctrl::Ch0,
            1 => Ch_Ctrl::Ch1,
            2 => Ch_Ctrl::Ch2,
            3 => Ch_Ctrl::Ch3,
            _ => Ch_Ctrl::Ch0, // Default to Ch0 for invalid values
        };
        let data_offset = (ch_id * ASPEED_IPC_DATA_SIZE_PER_CH / 4) as usize;
        AspeedIpcCtrl {
            tx_regs: unsafe { StaticRef::new(tx_addr as *const _) },
            rx_regs: unsafe { StaticRef::new(rx_addr as *const _) },
            ch,
            data_offset,
            target_dram_base,
        }
    }

    pub fn init(&self) {
        //caliptra_mcu_romtime::println!("[mcu-ipc] Initializing ASPEED IPC controller...");

        self.rx_regs.ctrl.modify(Ctrl::Enable::SET);
        /* Enable ch0 */
        self.rx_regs.enable.modify(self.ch.val(1));
    }

    pub fn is_rx_ready(&self) -> bool {
        self.tx_regs.enable.is_set(self.ch)
    }

    pub fn target_dram(&self, dram_offset: u64) -> u64 {
        self.target_dram_base + dram_offset
    }

    pub fn send(&self, cmd: u32, image_id: u32, dst_addr: u32) -> Result<(), AspeedIpcApiError> {
        if !self.is_rx_ready() {
            //caliptra_mcu_romtime::println!("TX not enabled for channel");
            return Err(AspeedIpcApiError::RXNotReady);
        }

        /* Wait until RX is ready */
        while self.tx_regs.status.is_set(self.ch) {}

        /* Write messages to TX data register */
        self.tx_regs.data[self.data_offset].set(cmd);
        self.tx_regs.data[self.data_offset + 1].set(image_id);
        self.tx_regs.data[self.data_offset + 2].set(dst_addr);

        /* Trigger TX */
        self.tx_regs.trig.set(1 << self.ch.shift);
        while self.tx_regs.status.is_set(self.ch) {}

        Ok(())
    }

    pub fn receive(
        &self,
        status: &mut u32,
        image_size: &mut u32,
        svn: &mut u32,
        load: &mut u32,
    ) -> Result<(), AspeedIpcApiError> {
        /* Wait until RX is ready */
        if !self.rx_regs.status.is_set(self.ch) {
            return Err(AspeedIpcApiError::RXEmpty);
        }

        /* Read message from RX data register */
        *status = self.rx_regs.data[self.data_offset].get();
        *image_size = self.rx_regs.data[self.data_offset + 1].get();
        *svn = self.rx_regs.data[self.data_offset + 2].get();
        *load = self.rx_regs.data[self.data_offset + 3].get();

        /* Clear RX status */
        self.rx_regs.status.modify(self.ch.val(1));
        Ok(())
    }
}
