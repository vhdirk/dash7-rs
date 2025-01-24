use std::sync::Arc;
use deku::prelude::*;

mod access_profile;
mod dll_config;
mod dll_status;
mod engineering_mode;
mod factory_settings;
mod firmware_version;
mod interface_configuration;
mod phy_status;
mod security_key;

pub use access_profile::AccessProfileFile;
pub use dll_config::DllConfig;
pub use dll_status::DllStatus;
pub use engineering_mode::{EngineeringMode, EngineeringModeMethod};
pub use factory_settings::FactorySettings;
pub use firmware_version::FirmwareVersion;
pub use interface_configuration::InterfaceConfiguration;
pub use phy_status::PhyStatus;
pub use security_key::SecurityKey;

use crate::{
    network::{Address, AddressType},
    utils::from_bytes,
};

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
#[deku(id_type = "u8", bits = "8")]
pub enum FileId {
    #[deku(id = "0x00")]
    UId,
    #[deku(id = "0x01")]
    FactorySettings,
    #[deku(id = "0x02")]
    FirmwareVersion,
    #[deku(id = "0x03")]
    DeviceCapacity,
    #[deku(id = "0x04")]
    DeviceStatus,
    #[deku(id = "0x05")]
    EngineeringMode,
    #[deku(id = "0x06")]
    VId,
    #[deku(id = "0x08")]
    PhyConfig,
    #[deku(id = "0x09")]
    PhyStatus,
    #[deku(id = "0x0A")]
    DllConfig,
    #[deku(id = "0x0B")]
    DllStatus,
    #[deku(id = "0x0C")]
    NetworkRouting,
    #[deku(id = "0x0D")]
    NetworkSecurity,
    #[deku(id = "0x0E")]
    NetworkSecurityKey,
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
    Rfu,
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
    D7AalpRfu,
    #[deku(id = "0x20")]
    AccessProfile00,
    #[deku(id = "0x21")]
    AccessProfile01,
    #[deku(id = "0x22")]
    AccessProfile02,
    #[deku(id = "0x23")]
    AccessProfile03,
    #[deku(id = "0x24")]
    AccessProfile04,
    #[deku(id = "0x25")]
    AccessProfile05,
    #[deku(id = "0x26")]
    AccessProfile06,
    #[deku(id = "0x27")]
    AccessProfile07,
    #[deku(id = "0x28")]
    AccessProfile08,
    #[deku(id = "0x29")]
    AccessProfile09,
    #[deku(id = "0x2A")]
    AccessProfile10,
    #[deku(id = "0x2B")]
    AccessProfile11,
    #[deku(id = "0x2C")]
    AccessProfile12,
    #[deku(id = "0x2D")]
    AccessProfile13,
    #[deku(id = "0x2E")]
    AccessProfile14,
    #[deku(id_pat = "_")]
    Other(u8),
}

impl TryFrom<u8> for FileId {
    type Error = DekuError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self::from_bytes((&vec![value], 0))?.1)
    }
}

impl Into<u8> for FileId {
    fn into(self) -> u8 {
        self.deku_id().unwrap()
    }
}

pub trait SystemFile {
    const ID: u8;
    const SIZE: u32;
}

/// File IDs 0x00-0x17 and 0x20-0x2F are reserved by the DASH7 spec.
/// File IDs 0x18-0x1F Reserved for D7AALP.
/// File IDs 0x20+I with I in [0, 14] are reserved for Access Profiles.
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
#[deku(
    ctx = "file_id: FileId, length: u32",
    id = "file_id",
    ctx_default = "FileId::Other(0xFF), 0"
)]
pub enum File {
    #[deku(id = "FileId::AccessProfile00")]
    AccessProfile00(AccessProfileFile<0>),
    // #[deku(id = "FileId::AccessProfile01")]
    // AccessProfile01(AccessProfile<1>),
    // #[deku(id = "FileId::AccessProfile02")]
    // AccessProfile02(AccessProfile<2>),
    // #[deku(id = "FileId::AccessProfile03")]
    // AccessProfile03(AccessProfile<3>),
    // #[deku(id = "FileId::AccessProfile04")]
    // AccessProfile04(AccessProfile<4>),
    // #[deku(id = "FileId::AccessProfile05")]
    // AccessProfile05(AccessProfile<5>),
    // #[deku(id = "FileId::AccessProfile06")]
    // AccessProfile06(AccessProfile<6>),
    // #[deku(id = "FileId::AccessProfile07")]
    // AccessProfile07(AccessProfile<7>),
    // #[deku(id = "FileId::AccessProfile08")]
    // AccessProfile08(AccessProfile<8>),
    // #[deku(id = "FileId::AccessProfile09")]
    // AccessProfile09(AccessProfile<9>),
    // #[deku(id = "FileId::AccessProfile10")]
    // AccessProfile10(AccessProfile<10>),
    // #[deku(id = "FileId::AccessProfile11")]
    // AccessProfile11(AccessProfile<11>),
    // #[deku(id = "FileId::AccessProfile12")]
    // AccessProfile12(AccessProfile<12>),
    // #[deku(id = "FileId::AccessProfile13")]
    // AccessProfile13(AccessProfile<13>),
    // #[deku(id = "FileId::AccessProfile14")]
    // AccessProfile14(AccessProfile<14>),

    #[deku(id = "FileId::UId")]
    UId(#[deku(ctx = "AddressType::UId")] Address),

    #[deku(id = "FileId::FactorySettings")]
    FactorySettings(FactorySettings),

    #[deku(id = "FileId::FirmwareVersion")]
    FirmwareVersion(FirmwareVersion),

    #[deku(id = "FileId::EngineeringMode")]
    EngineeringMode(EngineeringMode),

    #[deku(id = "FileId::VId")]
    VId(#[deku(ctx = "AddressType::VId")] Address),

    #[deku(id = "FileId::PhyStatus")]
    PhyStatus(PhyStatus),

    #[deku(id = "FileId::DllConfig")]
    DllConfig(DllConfig),

    #[deku(id = "FileId::DllStatus")]
    DllStatus(DllStatus),

    #[deku(id = "FileId::NetworkSecurityKey")]
    NwlSecurityKey(Arc<SecurityKey>),

    #[deku(id_pat = "_")]
    Other {
        id: u8,
        #[deku(count = "length")]
        buffer: Vec<u8>,
    },
}

impl Default for File {
    fn default() -> Self {
        Self::Other{id: 0xFF, buffer: vec![]}
    }
}

impl File {
    pub fn from_bytes<'a>(
        input: (&'a [u8], usize),
        file_id: FileId,
        length: u32,
    ) -> Result<((&'a [u8], usize), Self), DekuError> {
        from_bytes(input, (file_id, length))
    }

    // fn to_bytes(&self) -> Result<Vec<u8>, DekuError> {
    //     let output = self.to_bits()?;
    //     Ok(output.into_vec())
    // }

    // fn to_bits(&self) -> Result<BitVec<u8, Msb0>, DekuError> {
    //     let mut output: BitVec<u8, Msb0> = BitVec::new();
    //     self.write(&mut output, u32::MAX)?;
    //     Ok(output)
    // }
}
