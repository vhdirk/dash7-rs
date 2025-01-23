use deku::prelude::*;

use crate::physical::{ChannelHeader, SubBand};
use crate::types::VarInt;
use crate::utils::{read_array, write_array};

mod frame;
pub use frame::{BackgroundFrame, BackgroundFrameControl, ForegroundFrame, ForegroundFrameControl};

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Object)]
pub struct SubProfile {
    pub subband_bitmap: u8,
    pub scan_automation_period: VarInt,
}

/// The Access Specifier is the Index of the D7A file containing the Access
/// Profile. Each bit of the Access Mask corresponds to one of the subprofiles,
/// bit 0 corresponding to subprofile 0 and so on. The subprofiles having their
/// Access Mask bits set to 1 and having non-void (not null) subband bitmaps are
/// selected. As a result, only subprofiles performing scan automation (6.7) are
/// selectable.
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Object)]
pub struct AccessClass {
    #[deku(bits = 4)]
    pub specifier: u8,
    #[deku(bits = 4)]
    pub mask: u8,
}

#[uniffi::export]
impl AccessClass {
    #[uniffi::constructor]
    pub fn new(specifier: u8, mask: u8) -> Self {
        Self { specifier, mask }
    }

    #[uniffi::constructor]
    pub fn unavailable() -> Self {
        Self {
            specifier: 0x0F,
            mask: 0x0F,
        }
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Object)]
pub struct AccessProfile {
    pub channel_header: ChannelHeader,

    #[deku(
        reader = "read_array::<_, SubProfile, 4>(deku::reader)",
        writer = "write_array::<_, SubProfile, 4>(deku::writer, &self.sub_profiles)"
    )]
    pub sub_profiles: [SubProfile; 4],

    #[deku(
        reader = "read_array::<_, SubBand, 8>(deku::reader)",
        writer = "write_array::<_, SubBand, 8>(deku::writer, &self.sub_bands)"
    )]
    pub sub_bands: [SubBand; 8],
}
