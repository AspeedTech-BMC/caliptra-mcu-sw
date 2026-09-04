// Licensed under the Apache-2.0 license

#![no_std]

use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;
use registers_peripherals::aspeed_edaf::{
    EdafBridgeCfg, EdafBridgeGlobal, EdafBridgeGlobalRegisters, EdafBridgeMisc,
    EdafBridgeRegisters, EdafBridgeRoute, EspiCh3Ctrl, EspiChannel3Registers,
};
use registers_peripherals::ast2705::aspeed_scu1::{HostConf, SCU1Registers};

fn route_name(route: EdafBridgeRoute) -> &'static str {
    match route {
        EdafBridgeRoute::Espi0 => "bridge0/espi0",
        EdafBridgeRoute::Espi1 => "bridge1/espi1",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EdafBridgeConfig {
    pub enabled: bool,
    pub route: EdafBridgeRoute,
    pub ddr_mode: bool,
    pub erase_with_one: bool,
    pub mem_base: u64,
    pub ctl_base: Option<u64>,
    pub commands: EdafBridgeCommands,
}

impl EdafBridgeConfig {
    pub const fn disabled(route: EdafBridgeRoute) -> Self {
        Self {
            enabled: false,
            route,
            ddr_mode: false,
            erase_with_one: false,
            mem_base: 0,
            ctl_base: None,
            commands: EdafBridgeCommands::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EdafBridgeCommands {
    pub read: Option<u32>,
    pub write_enable: Option<u32>,
    pub write: Option<u32>,
    pub read_status: Option<u32>,
    pub erase_4k: Option<u32>,
    pub erase_32k: Option<u32>,
    pub erase_64k: Option<u32>,
}

impl EdafBridgeCommands {
    pub const fn default() -> Self {
        Self {
            read: None,
            write_enable: None,
            write: None,
            read_status: None,
            erase_4k: None,
            erase_32k: None,
            erase_64k: None,
        }
    }
}

pub struct AspeedEdafBridge {
    regs: StaticRef<EdafBridgeRegisters>,
    global: StaticRef<EdafBridgeGlobalRegisters>,
    espi: StaticRef<EspiChannel3Registers>,
    scu1: StaticRef<SCU1Registers>,
    config: EdafBridgeConfig,
}

impl AspeedEdafBridge {
    pub const fn new(
        regs: StaticRef<EdafBridgeRegisters>,
        global: StaticRef<EdafBridgeGlobalRegisters>,
        espi: StaticRef<EspiChannel3Registers>,
        scu1: StaticRef<SCU1Registers>,
        config: EdafBridgeConfig,
    ) -> Self {
        Self {
            regs,
            global,
            espi,
            scu1,
            config,
        }
    }

    pub fn init(&self) -> Result<(), ErrorCode> {
        let route = route_name(self.config.route);
        caliptra_mcu_romtime::println!(
            "[edaf] {} init start: enable={}, ddr={}, erase_with_one={}, mem_base={:#x}, ctl_base={:?}",
            route,
            self.config.enabled,
            self.config.ddr_mode,
            self.config.erase_with_one,
            self.config.mem_base,
            self.config.ctl_base,
        );

        if !self.config.enabled {
            return Ok(());
        }

        self.enable_scu1_edaf()?;

        if self.config.ddr_mode {
            self.regs.cfg.modify(EdafBridgeCfg::CmdEn::Disabled);
        } else {
            self.regs.cfg.modify(EdafBridgeCfg::CmdEn::Enabled);
            self.configure_spi_commands();
        }

        if self.config.erase_with_one {
            self.enable_erase_with_one();
        }

        self.set_mbase(self.config.mem_base);

        if let Some(ctl_base) = self.config.ctl_base {
            self.set_cbase(ctl_base);
        }

        self.set_espi_channel3_ready();

        Ok(())
    }

    fn enable_scu1_edaf(&self) -> Result<(), ErrorCode> {
        match self.config.route {
            EdafBridgeRoute::Espi0 => {
                if self.scu1.host_conf1.is_set(HostConf::LpcEn) {
                    return Err(ErrorCode::BUSY);
                }
                self.scu1.host_conf1.modify(HostConf::EdafEn::Enabled);
            }
            EdafBridgeRoute::Espi1 => {
                if self.scu1.host_conf2.is_set(HostConf::LpcEn) {
                    return Err(ErrorCode::BUSY);
                }
                self.scu1.host_conf2.modify(HostConf::EdafEn::Enabled);
            }
        }

        Ok(())
    }

    fn configure_spi_commands(&self) {
        if let Some(cmd) = self.config.commands.read {
            self.regs.cmd_read.set(cmd);
        }
        if let Some(cmd) = self.config.commands.write_enable {
            self.regs.cmd_write_enable.set(cmd);
        }
        if let Some(cmd) = self.config.commands.write {
            self.regs.cmd_write.set(cmd);
        }
        if let Some(cmd) = self.config.commands.read_status {
            self.regs.cmd_read_status.set(cmd);
        }
        if let Some(cmd) = self.config.commands.erase_4k {
            self.regs.cmd_erase_4k.set(cmd);
        }
        if let Some(cmd) = self.config.commands.erase_32k {
            self.regs.cmd_erase_32k.set(cmd);
        }
        if let Some(cmd) = self.config.commands.erase_64k {
            self.regs.cmd_erase_64k.set(cmd);
        }
    }

    fn enable_erase_with_one(&self) {
        match self.config.route {
            EdafBridgeRoute::Espi0 => self
                .global
                .global
                .modify(EdafBridgeGlobal::Bridge0EraseWithOne::Enabled),
            EdafBridgeRoute::Espi1 => self
                .global
                .global
                .modify(EdafBridgeGlobal::Bridge1EraseWithOne::Enabled),
        }
    }

    fn set_espi_channel3_ready(&self) {
        self.espi.ch3_ctrl.modify(EspiCh3Ctrl::SwReady::Ready);
    }

    fn set_mbase(&self, addr: u64) {
        self.regs.mbase.set(addr as u32);
        self.regs
            .misc
            .modify(EdafBridgeMisc::MbaseHigh.val((addr >> 32) as u32));
    }

    fn set_cbase(&self, addr: u64) {
        self.regs.cbase.set(addr as u32);
        self.regs
            .misc
            .modify(EdafBridgeMisc::CbaseHigh.val((addr >> 32) as u32));
    }
}
