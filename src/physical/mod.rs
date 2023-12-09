use deku::prelude::*;

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(bits = 2, endian = "big", type = "u8")]
pub enum Bandwidth {
    #[default]
    KHz200 = 0x00,
    KHz25 = 0x01,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(bits = 3, endian = "big", type = "u8")]
pub enum ChannelBand {
    #[default]
    NotImpl = 0x00,
    Band433 = 0x02,
    Band868 = 0x03,
    Band915 = 0x04,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(bits = 2, endian = "big", type = "u8")]
pub enum ChannelClass {
    #[default]
    LoRate = 0,
    Lora = 1, // TODO not part of spec
    NormalRate = 2,
    HiRate = 3,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(bits = 2, endian = "big", type = "u8")]
pub enum ChannelCoding {
    #[default]
    Pn9 = 0,
    Rfu = 1,
    FecPn9 = 2,
    Cw = 3,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct ChannelHeader {
    #[deku(pad_bits_before = "1")]
    pub channel_band: ChannelBand,
    pub channel_class: ChannelClass,
    pub channel_coding: ChannelCoding,
}

impl ChannelHeader {
    pub fn new(
        channel_band: ChannelBand,
        channel_class: ChannelClass,
        channel_coding: ChannelCoding,
    ) -> Self {
        Self {
            channel_band,
            channel_class,
            channel_coding,
        }
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct Channel {
    pub header: ChannelHeader,
    #[deku(endian = "big")]
    pub index: u16,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct ChannelStatusIdentifier {
    // TODO update to D7AP v1.1
    pub channel_band: ChannelBand,
    pub bandwidth: Bandwidth,

    #[deku(bits = 11, endian = "big")]
    pub index: u16,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(endian = "big")]
pub struct SubBand {
    pub channel_index_start: u16,
    pub channel_index_end: u16,
    pub eirp: i8,
    pub clear_channel_assessment: u8,
    pub duty: u8,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct ChannelStatus {
    pub identifier: ChannelStatusIdentifier,
    pub noise_floor: u8,
}
