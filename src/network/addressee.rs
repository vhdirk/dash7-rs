use deku::prelude::*;

use crate::{link::AccessClass, transport::GroupCondition};

use super::{Address, AddressType, NlsMethod, NlsState};

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct Addressee {
    /// Group condition
    /// Not used in sub-iot
    pub group_condition: GroupCondition,

    #[deku(update = "self.address.deku_id().unwrap()")]
    address_type: AddressType,

    /// Use VID instead of UID when possible
    /// Not used in sub-iot
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
