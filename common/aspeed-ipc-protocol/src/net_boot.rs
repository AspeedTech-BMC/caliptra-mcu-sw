// Licensed under the Apache-2.0 license
//

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

// ---- message type tags (byte 0 of every message below) ----
pub const NET_BOOT_MSG_INITIATE_BOOT_REQ: u8 = 0x01;
pub const NET_BOOT_MSG_INITIATE_BOOT_RESP: u8 = 0x81;
pub const NET_BOOT_MSG_IMAGE_METADATA_REQ: u8 = 0x02;
pub const NET_BOOT_MSG_IMAGE_METADATA_RESP: u8 = 0x82;
pub const NET_BOOT_MSG_IMAGE_DOWNLOAD_REQ: u8 = 0x03;
pub const NET_BOOT_MSG_IMAGE_CHUNK: u8 = 0x83;
pub const NET_BOOT_MSG_CHUNK_ACK: u8 = 0x04;
pub const NET_BOOT_MSG_FINALIZE: u8 = 0x05;
pub const NET_BOOT_MSG_FINALIZE_ACK: u8 = 0x85;

// ---- status codes ----
// `InitiateBootResponse` has its own 3-way status semantics, distinct from
// the Success/Failed pair shared by the other responses below.
pub const NET_BOOT_INITIATE_STATUS_STARTED: u8 = 0x00;
pub const NET_BOOT_INITIATE_STATUS_IN_PROGRESS: u8 = 0x01;
pub const NET_BOOT_INITIATE_STATUS_FAILED: u8 = 0xFF;

/// Shared by `ImageMetadataResponse`, `ImageChunkHeader`, `Finalize`, and
/// `FinalizeAck`.
pub const NET_BOOT_STATUS_SUCCESS: u8 = 0x00;
pub const NET_BOOT_STATUS_FAILED: u8 = 0xFF;

// See the module doc comment: this width (and these exact values) is
// transcribed from the spec's byte-offset table, not its prose.
pub const NET_BOOT_FIRMWARE_ID_CALIPTRA_FMC_RT: u32 = 0;
pub const NET_BOOT_FIRMWARE_ID_SOC_MANIFEST: u32 = 1;
pub const NET_BOOT_FIRMWARE_ID_MCU_RT: u32 = 2;

// ---- flag bits ----
pub const NET_BOOT_CHUNK_ACK_FLAG_READY_FOR_NEXT: u32 = 1 << 0;
pub const NET_BOOT_CHUNK_ACK_FLAG_ERROR: u32 = 1 << 1;

pub const NET_BOOT_IMAGE_METADATA_FLAG_COMPRESSED: u32 = 1 << 0;

pub const NET_BOOT_FINALIZE_ACK_FLAG_CLEAR_TOC: u32 = 1 << 0;
pub const NET_BOOT_FINALIZE_ACK_FLAG_RESET_CONNECTION: u32 = 1 << 1;

/// NetMCU network-interface init parameters, embedded in
/// `InitiateBootRequest`. Every field is a plain byte (not `u16`/`u32`), so
/// this type -- and anything that embeds it -- has alignment 1 and can
/// never pick up `#[repr(C)]` tail padding, which zerocopy's `IntoBytes`
/// derive rejects.
///
/// Bit-packed fields are private; use the accessor methods below so
/// reserved bits can't be set to something other than 0 by accident.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct VendorSpecific {
    /// [3:0] MAC Interface, [7:4] MDIO Interface.
    mac_mdio_iface: u8,
    /// [4:0] PHY Address, [5] MDI Nway, [6] Fixed Link, [7] reserved (0).
    phy_cfg: u8,
    /// [5:0] RGMII TX delay, [7:6] reserved (0).
    rgmii_tx_delay: u8,
    /// [5:0] RGMII RX delay, [7:6] reserved (0).
    rgmii_rx_delay: u8,
    pub mac_address: [u8; 6],
    /// [3:0] PHY link timeout (seconds), [7:4] reserved (0).
    phy_link_timeout: u8,
    /// [3:0] TFTP timeout (seconds), [7:4] TFTP retry count.
    tftp_timeout_retry: u8,
    /// [3:0] ARP/NS timeout (seconds), [7:4] ARP/NS retry count.
    arp_ns_timeout_retry: u8,
    /// [3:0] DHCPv4/v6 timeout (seconds), [7:4] DHCPv4/v6 retry count.
    dhcp_timeout_retry: u8,
}

impl VendorSpecific {
    /// All fields zero; populate with the setters below.
    pub const fn zeroed() -> Self {
        VendorSpecific {
            mac_mdio_iface: 0,
            phy_cfg: 0,
            rgmii_tx_delay: 0,
            rgmii_rx_delay: 0,
            mac_address: [0; 6],
            phy_link_timeout: 0,
            tftp_timeout_retry: 0,
            arp_ns_timeout_retry: 0,
            dhcp_timeout_retry: 0,
        }
    }

    /// 0x0=MAC0_RGMII, 0x1=MAC1_RGMII, 0x2-0xf=Reserved.
    pub fn mac_interface(&self) -> u8 {
        self.mac_mdio_iface & 0x0F
    }
    pub fn set_mac_interface(&mut self, v: u8) {
        self.mac_mdio_iface = (self.mac_mdio_iface & 0xF0) | (v & 0x0F);
    }

    /// 0x0=MDIO0, 0x1=MDIO1, 0x2=MDIO2, 0x3-0xf=Reserved.
    pub fn mdio_interface(&self) -> u8 {
        (self.mac_mdio_iface >> 4) & 0x0F
    }
    pub fn set_mdio_interface(&mut self, v: u8) {
        self.mac_mdio_iface = (self.mac_mdio_iface & 0x0F) | ((v & 0x0F) << 4);
    }

    /// Clause 22 (C22) or Clause 45 (C45) PHY address.
    pub fn phy_address(&self) -> u8 {
        self.phy_cfg & 0x1F
    }
    pub fn set_phy_address(&mut self, v: u8) {
        self.phy_cfg = (self.phy_cfg & !0x1F) | (v & 0x1F);
    }

    /// `false` = forced 100M, `true` = auto-negotiation.
    pub fn mdi_nway_autoneg(&self) -> bool {
        self.phy_cfg & (1 << 5) != 0
    }
    pub fn set_mdi_nway_autoneg(&mut self, v: bool) {
        self.phy_cfg = (self.phy_cfg & !(1 << 5)) | ((v as u8) << 5);
    }

    /// `false` = polling dedicated PHY, `true` = forced link.
    pub fn fixed_link(&self) -> bool {
        self.phy_cfg & (1 << 6) != 0
    }
    pub fn set_fixed_link(&mut self, v: bool) {
        self.phy_cfg = (self.phy_cfg & !(1 << 6)) | ((v as u8) << 6);
    }

    /// RGMII transmit clock delay for 10/100/1000 Mbps operation.
    pub fn rgmii_tx_delay(&self) -> u8 {
        self.rgmii_tx_delay & 0x3F
    }
    pub fn set_rgmii_tx_delay(&mut self, v: u8) {
        self.rgmii_tx_delay = v & 0x3F;
    }

    /// RGMII receive clock delay for 10/100/1000 Mbps operation.
    pub fn rgmii_rx_delay(&self) -> u8 {
        self.rgmii_rx_delay & 0x3F
    }
    pub fn set_rgmii_rx_delay(&mut self, v: u8) {
        self.rgmii_rx_delay = v & 0x3F;
    }

    /// PHY link timeout in seconds. 0=5 sec, 1-15=1-15 sec.
    pub fn phy_link_timeout_sec(&self) -> u8 {
        self.phy_link_timeout & 0x0F
    }
    pub fn set_phy_link_timeout_sec(&mut self, v: u8) {
        self.phy_link_timeout = (self.phy_link_timeout & 0xF0) | (v & 0x0F);
    }

    /// TFTP timeout in seconds. 0=1 sec, 1-15=1-15 sec.
    pub fn tftp_timeout_sec(&self) -> u8 {
        self.tftp_timeout_retry & 0x0F
    }
    pub fn set_tftp_timeout_sec(&mut self, v: u8) {
        self.tftp_timeout_retry = (self.tftp_timeout_retry & 0xF0) | (v & 0x0F);
    }

    /// TFTP retry attempts. 0=3 attempts, 1-15=1-15 attempts.
    pub fn tftp_retry_count(&self) -> u8 {
        (self.tftp_timeout_retry >> 4) & 0x0F
    }
    pub fn set_tftp_retry_count(&mut self, v: u8) {
        self.tftp_timeout_retry = (self.tftp_timeout_retry & 0x0F) | ((v & 0x0F) << 4);
    }

    /// ARP/NS timeout in seconds. 0=1 sec, 1-15=1-15 sec.
    pub fn arp_ns_timeout_sec(&self) -> u8 {
        self.arp_ns_timeout_retry & 0x0F
    }
    pub fn set_arp_ns_timeout_sec(&mut self, v: u8) {
        self.arp_ns_timeout_retry = (self.arp_ns_timeout_retry & 0xF0) | (v & 0x0F);
    }

    /// ARP/NS retry attempts. 0=3 attempts, 1-15=1-15 attempts.
    pub fn arp_ns_retry_count(&self) -> u8 {
        (self.arp_ns_timeout_retry >> 4) & 0x0F
    }
    pub fn set_arp_ns_retry_count(&mut self, v: u8) {
        self.arp_ns_timeout_retry = (self.arp_ns_timeout_retry & 0x0F) | ((v & 0x0F) << 4);
    }

    /// DHCPv4/v6 timeout in seconds. 0=4 sec, 1-15=1-15 sec.
    pub fn dhcp_timeout_sec(&self) -> u8 {
        self.dhcp_timeout_retry & 0x0F
    }
    pub fn set_dhcp_timeout_sec(&mut self, v: u8) {
        self.dhcp_timeout_retry = (self.dhcp_timeout_retry & 0xF0) | (v & 0x0F);
    }

    /// DHCPv4/v6 retry attempts. 0=4 attempts, 1-15=1-15 attempts.
    pub fn dhcp_retry_count(&self) -> u8 {
        (self.dhcp_timeout_retry >> 4) & 0x0F
    }
    pub fn set_dhcp_retry_count(&mut self, v: u8) {
        self.dhcp_timeout_retry = (self.dhcp_timeout_retry & 0x0F) | ((v & 0x0F) << 4);
    }
}

/// `NET_BOOT_MSG_INITIATE_BOOT_REQ` (0x01): SS-MCU -> NetMCU. Initiate boot
/// source discovery.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct InitiateBootRequest {
    pub message_type: u8,
    pub reserved0: [u8; 3],
    /// Not used yet. `[u8; 4]` (not `u32`) keeps this struct's alignment at
    /// 1, since 26 isn't a multiple of 4 -- see the module doc comment.
    pub protocol_version: [u8; 4],
    /// Not used yet; reserved. Same `[u8; 4]` reasoning as `protocol_version`.
    pub flags: [u8; 4],
    pub vendor_specific: VendorSpecific,
}

impl InitiateBootRequest {
    /// All fields zero; populate with the setters below.
    pub const fn new() -> Self {
        InitiateBootRequest {
            message_type: NET_BOOT_MSG_INITIATE_BOOT_REQ,
            reserved0: [0; 3],
            protocol_version: [0; 4],
            flags: [0; 4],
            vendor_specific: VendorSpecific::zeroed(),
        }
    }
}

/// `NET_BOOT_MSG_INITIATE_BOOT_RESP` (0x81): NetMCU -> SS-MCU. Confirm
/// discovery and image availability.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct InitiateBootResponse {
    pub message_type: u8,
    /// See `NET_BOOT_INITIATE_STATUS_*`.
    pub status: u8,
    /// Meaningful only if `status == NET_BOOT_INITIATE_STATUS_FAILED`.
    pub error_code: u16,
}

impl InitiateBootResponse {
    /// All fields zero; populate with the setters below.
    pub const fn new() -> Self {
        InitiateBootResponse {
            message_type: 0,
            status: 0,
            error_code: 0,
        }
    }
}

/// `NET_BOOT_MSG_IMAGE_METADATA_REQ` (0x02): SS-MCU -> NetMCU. Query image
/// metadata.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct ImageMetadataRequest {
    pub message_type: u8,
    pub reserved: [u8; 3],
    /// See the module doc comment re: this field's width.
    pub firmware_id: u32,
}

impl ImageMetadataRequest {
    /// All fields zero; populate with the setters below.
    pub const fn new() -> Self {
        ImageMetadataRequest {
            message_type: NET_BOOT_MSG_IMAGE_METADATA_REQ,
            firmware_id: 0,
            reserved: [0; 3],
        }
    }
}

/// `NET_BOOT_MSG_IMAGE_METADATA_RESP` (0x82): NetMCU -> SS-MCU. Return
/// image metadata and checksums.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct ImageMetadataResponse {
    pub message_type: u8,
    /// See `NET_BOOT_STATUS_*`.
    pub status: u8,
    /// Meaningful only if `status == NET_BOOT_STATUS_FAILED`.
    pub error_code: u16,
    pub image_size: u32,
    pub checksum: [u8; 32],
    /// Not used yet.
    pub version: u32,
    /// Not used yet. Bit 0 = `NET_BOOT_IMAGE_METADATA_FLAG_COMPRESSED`.
    pub flags: u32,
    pub reserved: u32,
}

impl ImageMetadataResponse {
    /// All fields zero; populate with the setters below.
    pub const fn new() -> Self {
        ImageMetadataResponse {
            message_type: 0,
            status: 0,
            error_code: 0,
            image_size: 0,
            checksum: [0; 32],
            version: 0,
            flags: 0,
            reserved: 0,
        }
    }
}

/// `NET_BOOT_MSG_IMAGE_DOWNLOAD_REQ` (0x03): SS-MCU -> NetMCU. Start image
/// transfer.
///
/// Same shape as `ImageMetadataRequest`, kept as a distinct type so a
/// "query" can't be passed where a "start transfer" is expected.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct ImageDownloadRequest {
    pub message_type: u8,
    pub reserved: [u8; 3],
    /// See the module doc comment re: this field's width.
    pub firmware_id: u32,
}

/// `NET_BOOT_MSG_IMAGE_CHUNK` (0x83): NetMCU -> SS-MCU. Header for one
/// image data chunk.
///
/// This is the header only -- see the module doc comment. The image bytes
/// themselves are NOT a field here; they're meant to live in a separate
/// staging-memory region sized `chunk_size`, addressed by a mechanism to be
/// designed in a follow-up task.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct ImageChunkHeader {
    pub message_type: u8,
    /// See `NET_BOOT_STATUS_*`.
    pub status: u8,
    /// Meaningful only if `status == NET_BOOT_STATUS_FAILED`.
    pub error_code: u16,
    /// For ordered delivery.
    pub sequence_number: u16,
    pub reserved: u16,
    /// Marked "not used" in the spec; kept for wire compatibility.
    pub offset: u32,
    /// Bytes of image data staged in the external staging buffer for this
    /// chunk.
    pub chunk_size: u32,
}

/// `NET_BOOT_MSG_CHUNK_ACK` (0x04): SS-MCU -> NetMCU. Acknowledge chunk and
/// flow control.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct ChunkAck {
    pub message_type: u8,
    pub reserved0: [u8; 3],
    pub firmware_id: u32,
    pub sequence_number: u16,
    pub reserved1: u16,
    pub flags: u32,
}

/// `NET_BOOT_MSG_FINALIZE` (0x05): SS-MCU -> NetMCU. Notify recovery
/// completion/error.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct Finalize {
    pub message_type: u8,
    /// See `NET_BOOT_STATUS_*`.
    pub status: u8,
    /// Meaningful only if `status == NET_BOOT_STATUS_FAILED`.
    pub error_code: u16,
    /// For future use.
    pub reserved: u32,
}

/// `NET_BOOT_MSG_FINALIZE_ACK` (0x85): NetMCU -> SS-MCU. Final
/// acknowledgment.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct FinalizeAck {
    pub message_type: u8,
    /// See `NET_BOOT_STATUS_*`.
    pub status: u8,
    /// Meaningful only if `status == NET_BOOT_STATUS_FAILED`.
    pub error_code: u16,
    /// Bit 0 = `NET_BOOT_FINALIZE_ACK_FLAG_CLEAR_TOC`, bit 1 =
    /// `NET_BOOT_FINALIZE_ACK_FLAG_RESET_CONNECTION`.
    pub cleanup_flags: u32,
    /// For future use.
    pub reserved: u32,
}
