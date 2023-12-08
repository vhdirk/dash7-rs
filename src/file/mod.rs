use deku::prelude::*;

mod access_profile;
mod dll_config;
mod dll_status;
mod engineering_mode;
mod firmware_version;
mod phy_status;
mod security_key;
mod factory_settings;
mod interface_configuration;

pub use access_profile::AccessProfile;
pub use dll_config::DllConfig;
pub use dll_status::DllStatus;
pub use engineering_mode::{EngineeringMode, EngineeringModeMethod};
pub use firmware_version::FirmwareVersion;
pub use phy_status::PhyStatus;
pub use security_key::SecurityKey;
pub use factory_settings::FactorySettings;
pub use interface_configuration::InterfaceConfiguration;

use crate::network::{AddressType, Address};

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(type="u8")]
pub enum SystemFileId{
    #[deku(id="0x00")] Uid,
    #[deku(id="0x01")] FactorySettings,
    #[deku(id="0x02")] FirmwareVersion,
    #[deku(id="0x03")] DeviceCapacity,
    #[deku(id="0x04")] DeviceStatus,
    #[deku(id="0x05")] EngineeringMode,
    #[deku(id="0x06")] Vid,
    #[deku(id="0x08")] PhyConfig,
    #[deku(id="0x09")] PhyStatus,
    #[deku(id="0x0A")] DllConfig,
    #[deku(id="0x0B")] DllStatus,
    #[deku(id="0x0C")] NwlRouting,
    #[deku(id="0x0D")] NwlSecurity,
    #[deku(id="0x0E")] NwlSecurityKey,
    #[deku(id="0x0F")] NwlSsr,
    #[deku(id="0x10")] NwlStatus,
    #[deku(id="0x11")] TrlStatus,
    #[deku(id="0x12")] SelConfig,
    #[deku(id="0x13")] FofStatus,
    #[deku(id_pat="0x07 | 0x14..=0x16")] Rfu(u8),
    #[deku(id="0x17")] LocationData,
    #[deku(id_pat="0x18..=0x1F")] D7AalpRfu(u8),
    #[deku(id_pat="0x20..=0x2E")] AccessProfile(u8)
}


#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx="file_id: SystemFileId, length: u32", id="file_id")]
pub enum SystemFile {
    // TODO: how to pass specifier to nested struct
    // #[deku(id_pat="SystemFileId::AccessProfile(specifier)")]
    // AccessProfile(#[deku(ctx="*specifier")]AccessProfile),

    #[deku(id="SystemFileId::Uid")]
    Uid(#[deku(ctx="AddressType::Uid")]Address),

    #[deku(id="SystemFileId::FactorySettings")]
    FactorySettings(FactorySettings),

    #[deku(id="SystemFileId::FirmwareVersion")]
    FirmwareVersion(FirmwareVersion),

    #[deku(id="SystemFileId::EngineeringMode")]
    EngineeringMode(EngineeringMode),

    #[deku(id="SystemFileId::Vid")]
    Vid(#[deku(ctx="AddressType::Vid")]Address),

    #[deku(id="SystemFileId::PhyStatus")]
    PhyStatus(PhyStatus),

    #[deku(id="SystemFileId::DllConfig")]
    DllConfig(DllConfig),

    #[deku(id="SystemFileId::DllStatus")]
    DllStatus(DllStatus),

    #[deku(id="SystemFileId::NwlSecurityKey")]
    NwlSecurityKey(SecurityKey),

    #[deku(id_pat="_")]
    Other(#[deku(count="length")] Vec<u8>)
}