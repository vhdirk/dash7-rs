use deku::prelude::*;

use crate::{
    file::{FileCtx, OtherFile}, network::{Address, AddressType, NetworkFrame}, types::Length
};

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct BackgroundFrameControl {
    address_type: AddressType,

    #[deku(bits = 6)]
    tag_id: u8,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct BackgroundFrame {
    subnet: u8,
    control: BackgroundFrameControl,
    payload: u16,
    crc16: u16,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct ForegroundFrameControl {
    address_type: AddressType,

    #[deku(bits = 6)]
    eirp_index: u8,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct ForegroundFrame<F = OtherFile>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
 {
    length: Length,
    subnet: u8,
    control: ForegroundFrameControl,

    #[deku(ctx = "control.address_type")]
    target_address: Address,

    #[deku(ctx = "Into::<u32>::into(*length)")]
    frame: NetworkFrame<F>,
    crc16: u16,
}
