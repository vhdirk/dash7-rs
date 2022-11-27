
use modular_bitfield::prelude::*;

use crate::{physical::{ChannelHeader, SubBand}, types::CompressedValue};

#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
#[bits = 6]
pub enum CsmaCaMode {
    Unc = 0,
    Aind = 1,
    Raind = 2,
    Rigd = 3,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, PartialEq)]
pub struct SubProfile {
    pub subband_bitmap: u8,
    pub scan_automation_period: CompressedValue,
}

pub struct AccessProfile {
    pub channel_header: ChannelHeader,
    pub sub_profiles: [SubProfile; 4],
    pub sub_bands: [SubBand; 8],
}
