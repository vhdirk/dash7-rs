use packed_struct::prelude::*;

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum Bandwidth {
    KHz200 = 0x00,
    KHz25= 0x01,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum ChannelBand {
    NotImpl = 0x00,
    Band433 = 0x02,
    Band868 = 0x03,
    Band915 = 0x04,
}


#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum ChannelClass {
    LoRate = 0,
    Lora = 1, // TODO not part of spec
    NormalRate = 2,
    HiRate = 3,
}
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum ChannelCoding {
    Pn9 = 0,
    Rfu = 1,
    FecPn9 = 2,
    Cw = 3,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct ChannelHeader {
    #[packed_field(bits = "1..=3", ty = "enum")]
    pub channel_band: ChannelBand,

    #[packed_field(bits = "4..=5", ty = "enum")]
    pub channel_class: ChannelClass,

    #[packed_field(bits = "6..=7", ty = "enum")]
    pub channel_coding: ChannelCoding,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct ChannelId {
    #[packed_field(element_size_bytes="1")]
   pub header: ChannelHeader,

    #[packed_field(endian="msb")]
   pub index: u16,
}


#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct ChannelStatusIdentifier {
  // TODO update to D7AP v1.1

    #[packed_field(bits = "0..=2", ty = "enum")]
    pub channel_band: ChannelBand,

    #[packed_field(bits = "3", ty = "enum")]
    pub bandwidth: Bandwidth,

    #[packed_field(bits = "5..=15",endian="msb")]
    pub index: Integer<u16, packed_bits::Bits<11>>,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct SubBand {
    #[packed_field(endian="msb")]
    pub channel_index_start: u16,

    #[packed_field(endian="msb")]
    pub channel_index_end: u16,
    pub eirp: i8,
    pub clear_channel_assessment: u8,
    pub duty: u8,
}