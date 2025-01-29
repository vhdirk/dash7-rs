use deku::{DekuRead, DekuWrite};

use super::{address::AddressFile, other::OtherFile, AccessProfileFile, DllConfigFile, DllStatusFile, EngineeringModeFile, FactorySettingsFile, FileCtx, PhyStatusFile, SecurityKeyFile};
use crate::network::{Address, AddressType};

/// File IDs 0x00-0x17 and 0x20-0x2F are reserved by the DASH7 spec.
/// File IDs 0x18-0x1F Reserved for D7AALP.
/// File IDs 0x20+I with I in [0, 14] are reserved for Access Profiles.
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
#[deku(ctx = "ctx: FileCtx", id = "ctx.id")]
pub enum SystemFile {
    #[deku(id = "0x00")]
    UId(#[deku(ctx = "AddressType::UId")] AddressFile),

    #[deku(id = "0x01")]
    FactorySettings(FactorySettingsFile),

    #[deku(id = "0x02")]
    FirmwareVersion,

    #[deku(id = "0x03")]
    DeviceCapacity,

    #[deku(id = "0x04")]
    DeviceStatus,

    #[deku(id = "0x05")]
    EngineeringMode(EngineeringModeFile),

    #[deku(id = "0x06")]
    VId(#[deku(ctx = "AddressType::VId")] AddressFile),

    #[deku(id = "0x08")]
    PhyConfig,

    #[deku(id = "0x09")]
    PhyStatus(PhyStatusFile),

    #[deku(id = "0x0A")]
    DllConfig(DllConfigFile),

    #[deku(id = "0x0B")]
    DllStatus(DllStatusFile),

    #[deku(id = "0x0C")]
    NetworkRouting,

    #[deku(id = "0x0D")]
    NetworkSecurity,

    #[deku(id = "0x0E")]
    NetworkSecurityKey(SecurityKeyFile),

    #[deku(id = "0x0F")]
    NetworkSsr,

    #[deku(id = "0x10")]
    NetworkStatus,

    #[deku(id = "0x11")]
    TrlStatus,

    #[deku(id = "0x12")]
    SelConfig,

    #[deku(id = "0x13")]
    FofStatus,

    #[deku(id_pat = "0x07 | 0x14..=0x16")]
    Rfu(u8),

    #[deku(id = "0x17")]
    LocationData,

    #[deku(id = "0x18")]
    RootKey,

    #[deku(id = "0x19")]
    UserKey,

    #[deku(id = "0x1B")]
    SensorDescription,

    #[deku(id = "0x1C")]
    Rtc,

    #[deku(id_pat = "0x1D..=0x1F")]
    D7AalpRfu(u8),

    #[deku(id_pat = "0x20..=0x2E")]
    AccessProfile(#[deku(ctx = "ctx.id - 0x20")] AccessProfileFile),
}


