use deku::prelude::*;

use crate::link::AccessClass;

#[cfg(feature = "_wizzilab")]
use crate::transport::GroupCondition;

use super::{Address, AddressType, NlsMethod, NlsState};

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct Addressee {
    /// Group condition
    #[cfg(feature = "_wizzilab")]
    pub group_condition: GroupCondition,

    #[cfg_attr(not(feature = "_wizzilab"), deku(pad_bits_before = "2"))]
    #[deku(update = "self.address.deku_id().unwrap()")]
    address_type: AddressType,

    /// Use VID instead of UID when possible
    #[cfg(feature = "_wizzilab")]
    #[deku(bits = 1)]
    pub use_vid: bool,

    #[deku(update = "self.nls_state.deku_id().unwrap()")]
    nls_method: NlsMethod,

    pub access_class: AccessClass,

    #[deku(ctx = "*address_type")]
    pub address: Address,

    #[deku(ctx = "*nls_method")]
    pub nls_state: NlsState,
}

impl Addressee {
    pub fn new(
        #[cfg(feature = "_wizzilab")] use_vid: bool,
        #[cfg(feature = "_wizzilab")] group_condition: GroupCondition,
        address: Address,
        nls_state: NlsState,
        access_class: AccessClass,
    ) -> Self {
        Self {
            #[cfg(feature = "_wizzilab")]
            use_vid,
            #[cfg(feature = "_wizzilab")]
            group_condition,
            address_type: address.deku_id().unwrap(),
            nls_method: nls_state.deku_id().unwrap(),
            access_class,
            address,
            nls_state,
        }
    }
}
