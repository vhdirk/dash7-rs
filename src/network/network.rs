use deku::prelude::*;

use crate::transport;

use super::{Address, AddressType, NlsMethod};

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct Control {
    #[deku(bits = 1)]
    has_no_origin_access_id: bool,
    #[deku(bits = 1)]
    has_hopping: bool,

    origin_address_type: AddressType,

    #[deku(pad_bits_before = "1")]
    nls_method: NlsMethod,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct HoppingControl {
    /// Hopping counter for no-hop and one-hop routing.
    #[deku(bits = 1, pad_bits_before = "1")]
    hop_counter: bool,

    #[deku(pad_bits_after = "4")]
    destination_address_type: AddressType,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(ctx = "command_length: u32", ctx_default = "u32::MAX")]
pub struct Frame {
    control: Control,

    #[deku(cond = "control.has_hopping")]
    hopping_control: Option<HoppingControl>,

    origin_access_class: u8,

    #[deku(ctx = "control.origin_address_type")]
    origin_access_adress: Address,

    #[deku(ctx = "command_length")]
    frame: transport::Frame,
}
