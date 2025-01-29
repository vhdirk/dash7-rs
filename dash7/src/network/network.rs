use deku::prelude::*;

use crate::{file::FileCtx, link::AccessClass, transport::TransportFrame};

use super::{Address, AddressType, NlsMethod};

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct NetworkFrameControl {
    #[deku(bits = 1)]
    pub has_no_origin_access_id: bool,
    #[deku(bits = 1)]
    pub has_hopping: bool,

    pub origin_address_type: AddressType,

    #[deku(pad_bits_before = "1")]
    pub nls_method: NlsMethod,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct HoppingControl {
    /// Hopping counter for no-hop and one-hop routing.
    #[deku(bits = 1, pad_bits_before = "1")]
    pub hop_counter: bool,

    #[deku(pad_bits_after = "4")]
    pub destination_address_type: AddressType,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(ctx = "command_length: u32", ctx_default = "0")]
pub struct NetworkFrame<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    pub control: NetworkFrameControl,

    #[deku(cond = "control.has_hopping")]
    pub hopping_control: Option<HoppingControl>,

    pub origin_access_class: AccessClass,

    #[deku(ctx = "control.origin_address_type")]
    pub origin_access_adress: Address,

    #[deku(ctx = "command_length")]
    pub frame: TransportFrame<F>,
}
