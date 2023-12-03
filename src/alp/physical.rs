use deku::prelude::*;

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 2, endian = "big", type = "u8")]
pub enum Bandwidth {
    KHz200 = 0x00,
    KHz25 = 0x01,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 3, endian = "big", type = "u8")]
pub enum ChannelBand {
    NotImpl = 0x00,
    Band433 = 0x02,
    Band868 = 0x03,
    Band915 = 0x04,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 2, endian = "big", type = "u8")]
pub enum ChannelClass {
    LoRate = 0,
    Lora = 1, // TODO not part of spec
    NormalRate = 2,
    HiRate = 3,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 2, endian = "big", type = "u8")]
pub enum ChannelCoding {
    Pn9 = 0,
    Rfu = 1,
    FecPn9 = 2,
    Cw = 3,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelHeader {
    #[deku(pad_bits_before = "1")]
    pub channel_band: ChannelBand,
    pub channel_class: ChannelClass,
    pub channel_coding: ChannelCoding,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelId {
    pub header: ChannelHeader,
    #[deku(bits = 16)]
    pub index: u16,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelStatusIdentifier {
    // TODO update to D7AP v1.1
    pub channel_band: ChannelBand,
    pub bandwidth: Bandwidth,

    #[deku(bits = 11, endian = "big")]
    pub index: u16,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(endian = "big")]
pub struct SubBand {
    #[deku(bits = 16)]
    pub channel_index_start: u16,
    #[deku(bits = 16)]
    pub channel_index_end: u16,
    #[deku(bits = 8)]
    pub eirp: u8,
    #[deku(bits = 8)]
    pub clear_channel_assessment: u8,
    #[deku(bits = 8)]
    pub duty: u8,
}
