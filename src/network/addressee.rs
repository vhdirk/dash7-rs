use deku::prelude::*;

use crate::link::AccessClass;

#[cfg(feature = "_wizzilab")]
use crate::transport::GroupCondition;

use super::{Address, AddressType, NlsMethod, NlsState};

#[cfg(not(feature = "_wizzilab"))]
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct Addressee {
    #[deku(update = "self.address.deku_id().unwrap()", pad_bits_before = "2")]
    address_type: AddressType,

    #[deku(update = "self.nls_state.deku_id().unwrap()")]
    nls_method: NlsMethod,

    pub access_class: AccessClass,

    #[deku(ctx = "*address_type")]
    pub address: Address,

    #[deku(ctx = "*nls_method")]
    pub nls_state: NlsState,
}

#[cfg(feature = "_wizzilab")]
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct Addressee {
    /// Group condition
    pub group_condition: GroupCondition,

    #[deku(update = "self.address.deku_id().unwrap()")]
    address_type: AddressType,

    /// Use VID instead of UID when possible
    /// Only for wizzilab
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
    #[cfg(not(feature = "_wizzilab"))]
    pub fn new(address: Address, nls_state: NlsState, access_class: AccessClass) -> Self {
        Self {
            address_type: address.deku_id().unwrap(),
            nls_method: nls_state.deku_id().unwrap(),
            access_class,
            address,
            nls_state,
        }
    }

    #[cfg(feature = "_wizzilab")]
    pub fn new(
        use_vid: bool,
        group_condition: GroupCondition,
        address: Address,
        nls_state: NlsState,
        access_class: AccessClass,
    ) -> Self {
        Self {
            use_vid,
            group_condition,
            address_type: address.deku_id().unwrap(),
            nls_method: nls_state.deku_id().unwrap(),
            access_class,
            address,
            nls_state,
        }
    }
}
