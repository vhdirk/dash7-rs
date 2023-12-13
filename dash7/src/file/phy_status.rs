use deku::prelude::*;

use crate::physical::ChannelStatus;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct PhyStatus {
    #[deku(endian = "big")]
    pub up_time: u32,
    #[deku(endian = "big")]
    pub rx_time: u32,
    #[deku(endian = "big")]
    pub tx_time: u32,
    #[deku(endian = "big")]
    pub tx_duty_cycle: u16,

    #[deku(update = "self.channel_status.len()")]
    channel_status_list_length: u8,

    #[deku(count = "*channel_status_list_length")]
    pub channel_status: Vec<ChannelStatus>,
}

impl PhyStatus {
    pub fn new(
        up_time: u32,
        rx_time: u32,
        tx_time: u32,
        tx_duty_cycle: u16,
        channel_status: Vec<ChannelStatus>,
    ) -> Self {
        Self {
            up_time,
            rx_time,
            tx_time,
            tx_duty_cycle,
            channel_status_list_length: channel_status.len() as u8,
            channel_status,
        }
    }
}
