use deku::prelude::*;

use crate::{
    app::operand::Length,
    network::{self, Address, AddressType},
};

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct BackgroundFrameControl {
    address_type: AddressType,

    #[deku(bits = 6)]
    tag_id: u8,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct BackgroundFrame {
    subnet: u8,
    control: BackgroundFrameControl,
    payload: u16,
    crc16: u16,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct ForegroundFrameControl {
    address_type: AddressType,

    #[deku(bits = 6)]
    eirp_index: u8,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct ForegroundFrame {
    length: Length,
    subnet: u8,
    control: ForegroundFrameControl,

    #[deku(ctx = "control.address_type")]
    target_address: Address,

    #[deku(ctx = "Into::<u32>::into(*length)")]
    frame: network::Frame,
    crc16: u16,
}
