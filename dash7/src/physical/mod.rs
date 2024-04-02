use deku::prelude::*;

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(bits = 2, endian = "big", type = "u8")]
pub enum Bandwidth {
    #[default]
    KHz200 = 0x00,
    KHz25 = 0x01,
}

/// D7A channel bands indexes
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(bits = 3, endian = "big", type = "u8")]
pub enum ChannelBand {
    #[default]
    Rfu0 = 0x00,
    Rfu1 = 0x01,
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
#[deku(bits = 4, type = "u8")]
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

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(bits = 4, type = "u8")]
pub enum NoiseFloor {
    /// Noise floor (in dBm). Use the default channel CCA threshold (5.4).
    /// Eccao is set to 0 dB.
    #[deku(id = "0x00")]
    Fixed(u8),

    /// Forget factor (seconds). Slow RSSI Variation with 6dB offset. Noise
    /// Floor is computed based on regular RSSI measurements with a forget
    /// factor defined by the associated parameter. Measurement above the
    /// default channel CCA threshold are discarded. The regular RSSI
    /// measurements period shall be at most 1/8 of the forget factor duration.
    /// Eccao is set to 6 dB.
    #[deku(id = "0x01")]
    ForgetFactor(u8),

    /// User defined
    #[deku(id_pat = "_")]
    Other,
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
