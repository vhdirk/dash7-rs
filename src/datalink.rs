use packed_struct::prelude::*;

use crate::{physical::{ChannelHeader, SubBand}, types::CompressedValue};

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum CsmaCaMode {
    Unc = 0,
    Aind = 1,
    Raind = 2,
    Rigd = 3,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct  SubProfile {
    subband_bitmap: u8,

    #[packed_field(element_size_bytes = "1")]
    scan_automation_period: CompressedValue,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct AccessProfile {
    #[packed_field(element_size_bytes = "1")]
    pub channel_header: ChannelHeader,

    #[packed_field(element_size_bytes = "2")]
    pub sub_profiles: [SubProfile; 4],

    #[packed_field(element_size_bytes = "7")]
    pub sub_bands: [SubBand; 8],
}
