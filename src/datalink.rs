use deku::prelude::*;

use crate::{
    // physical::{ChannelHeader, SubBand},
    varint::VarInt,
};

#[deku_derive(DekuRead, DekuWrite)]
#[deku(bits=6, type="u8")]
pub enum CsmaCaMode {
    Unc = 0,
    Aind = 1,
    Raind = 2,
    Rigd = 3,
}

#[deku_derive(DekuRead, DekuWrite)]
pub struct SubProfile {
    pub subband_bitmap: u8,
    pub scan_automation_period: VarInt,
}

// pub struct AccessProfile {
//     pub channel_header: ChannelHeader,
//     pub sub_profiles: [SubProfile; 4],
//     pub sub_bands: [SubBand; 8],
// }
