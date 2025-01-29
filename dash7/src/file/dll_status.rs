use deku::prelude::*;

use crate::physical::ChannelHeader;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Record)]
pub struct DllStatusFile {
    pub last_rx_packet_level: u8,
    pub last_rx_packet_link_budget: u8,
    pub noise_floor: u8,
    pub channel_header: ChannelHeader,

    #[deku(endian = "big")]
    pub channel_index: u16,
    #[deku(endian = "big")]
    pub scan_timeout_ratio: u16,
    #[deku(endian = "big")]
    pub scan_count: u32,
    #[deku(endian = "big")]
    pub scan_timeout_count: u32,
}
