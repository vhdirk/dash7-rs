use deku::prelude::*;

use crate::types::VarInt;
use crate::utils::{read_array, write_array};
use crate::physical::{ChannelHeader, SubBand};

mod frame;
pub use frame::{BackgroundFrameControl, BackgroundFrame, ForegroundFrameControl, ForegroundFrame};

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(bits = 6, type = "u8")]
pub enum CsmaCaMode {
    #[default]
    #[deku(id = "0")]
    Unc,
    #[deku(id = "1")]
    Aind,
    #[deku(id = "2")]
    Raind,
    #[deku(id = "3")]
    Rigd,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct SubProfile {
    pub subband_bitmap: u8,
    pub scan_automation_period: VarInt,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct AccessProfile {
    pub channel_header: ChannelHeader,

    #[deku(
        reader = "read_array::<SubProfile, 4>(deku::rest)",
        writer = "write_array::<SubProfile, 4>(deku::output, &self.sub_profiles)"
    )]
    pub sub_profiles: [SubProfile; 4],

    #[deku(
        reader = "read_array::<SubBand, 8>(deku::rest)",
        writer = "write_array::<SubBand, 8>(deku::output, &self.sub_bands)"
    )]
    pub sub_bands: [SubBand; 8],
}
