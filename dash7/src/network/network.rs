use deku::prelude::*;

use crate::{link::AccessClass, transport};

use super::{Address, AddressType, NlsMethod};

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Object)]
pub struct Control {
    #[deku(bits = 1)]
    pub has_no_origin_access_id: bool,
    #[deku(bits = 1)]
    pub has_hopping: bool,

    pub origin_address_type: AddressType,

    #[deku(pad_bits_before = "1")]
    pub nls_method: NlsMethod,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Object)]
pub struct HoppingControl {
    /// Hopping counter for no-hop and one-hop routing.
    #[deku(bits = 1, pad_bits_before = "1")]
    pub hop_counter: bool,

    #[deku(pad_bits_after = "4")]
    pub destination_address_type: AddressType,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Object)]
#[deku(ctx = "command_length: u32", ctx_default = "0")]
pub struct Frame {
    pub control: Control,

    #[deku(cond = "control.has_hopping")]
    pub hopping_control: Option<HoppingControl>,

    pub origin_access_class: AccessClass,

    #[deku(ctx = "control.origin_address_type")]
    pub origin_access_adress: Address,

    #[deku(ctx = "command_length")]
    pub frame: transport::Frame,
}
