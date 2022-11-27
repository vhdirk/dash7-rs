use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier, Debug, PartialEq)]
#[bits = 2]
pub enum Bandwidth {
    KHz200 = 0x00,
    KHz25= 0x01,
}

#[derive(BitfieldSpecifier, Debug, PartialEq)]
#[bits = 3]
pub enum ChannelBand {
    NotImpl = 0x00,
    Band433 = 0x02,
    Band868 = 0x03,
    Band915 = 0x04,
}

#[derive(BitfieldSpecifier, Debug, PartialEq)]
#[bits = 2]
pub enum ChannelClass {
    LoRate = 0,
    Lora = 1, // TODO not part of spec
    NormalRate = 2,
    HiRate = 3,
}

#[derive(BitfieldSpecifier, Debug, PartialEq)]
#[bits = 2]
pub enum ChannelCoding {
    Pn9 = 0,
    Rfu = 1,
    FecPn9 = 2,
    Cw = 3,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, PartialEq)]
pub struct ChannelHeader {
    #[skip] __: B1,
    pub channel_band: ChannelBand,
    pub channel_class: ChannelClass,
    pub channel_coding: ChannelCoding
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, PartialEq)]
pub struct ChannelId {
   pub header: ChannelHeader,

   pub index: u16,
}


#[bitfield]
#[derive(BitfieldSpecifier, Debug, PartialEq)]
pub struct ChannelStatusIdentifier {
  // TODO update to D7AP v1.1
    pub channel_band: ChannelBand,
    pub bandwidth: Bandwidth,
    pub index: B11,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, PartialEq)]
pub struct SubBand {
    pub channel_index_start: u16,
    pub channel_index_end: u16,
    pub eirp: u8,
    pub clear_channel_assessment: u8,
    pub duty: u8,
}