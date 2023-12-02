use deku::prelude::*;

use crate::{
    physical::{ChannelHeader, SubBand},
    varint::VarInt,
};

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits=6, type="u8")]
pub enum CsmaCaMode {
    Unc = 0,
    Aind = 1,
    Raind = 2,
    Rigd = 3,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct SubProfile {
    pub subband_bitmap: u8,
    pub scan_automation_period: VarInt,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct AccessProfile {
    pub channel_header: ChannelHeader,
    #[deku(count = "4")]
    pub sub_profiles: Vec<SubProfile>,
    #[deku(count = "8")]
    pub sub_bands: Vec<SubBand>,
}
