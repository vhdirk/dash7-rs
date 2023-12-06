use deku::prelude::*;

use super::{session::QoS, network::{Address, NlsMethod, AddressType}, operand::Length};

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
#[deku(bits = 2, type = "u8")]
pub enum GroupCondition {
    /// <, =, > (always true)
    #[default]
    #[deku(id = "0")]
    Any,
    /// <, >
    #[deku(id = "1")]
    NotEqual,
    /// =
    #[deku(id = "2")]
    Equal,
    /// >
    #[deku(id = "3")]
    GreaterThan,
}

/// Section 9.2.1
///
/// Parameters to handle the sending of a request.
// ALP SPEC: Add link to D7a section
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct InterfaceConfiguration {
    pub qos: QoS,
    /// Flush Start Timeout in Compressed Format, unit is in seconds
    ///
    /// Maximum time to send the packet. This means that the modem will wait for a "good opportunity"
    /// to send the packet until the timeout, after which it will just send the packet over the
    /// air.
    ///
    /// A good opportunity is, for example, if we are sending another packet to the same target,
    /// then we can aggregate the requests, to avoid advertising twice. Another example would be if
    /// the target sends us a packet, the modem can aggregate our request to the response of the
    /// request of the target.
    pub to: u8,
    /// Response Execution Delay in Compressed Format, unit is in milliseconds.
    ///
    /// Time given to the target to process the request.
    pub te: u8,

    /// Group condition
    pub group_condition: GroupCondition,

    #[deku(update="self.address.deku_id().unwrap()")]
    address_type: AddressType,

    /// Use VID instead of UID when possible
    #[deku(bits=1)]
    pub use_vid: bool,

    /// Security method
    pub nls_method: NlsMethod,

    /// Access class of the targeted listening device
    pub access_class: u8,

    /// Address of the target.
    #[deku(ctx = "*address_type")]
    pub address: Address,

}


/// Dash7 interface
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Clone, Debug, PartialEq)]
pub struct OverloadedIndirectInterface {
    /// File containing the `QoS`, `to` and `te` to use for the transmission (see
    /// dash7::InterfaceConfiguration
    pub interface_file_id: u8,

    #[deku(update="self.address.deku_id().unwrap()", pad_bits_before="2")]
    address_type: AddressType,

    pub nls_method: NlsMethod,
    pub access_class: u8,

    #[deku(ctx = "*address_type")]
    pub address: Address,
}

impl OverloadedIndirectInterface {
    pub fn new(interface_file_id: u8, nls_method: NlsMethod, access_class: u8, address: Address) -> Self {
        Self {
            interface_file_id,
            address_type: address.deku_id().unwrap(),
            nls_method,
            access_class,
            address
        }
    }
}


/// Non Dash7 interface
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Clone, Debug, PartialEq)]
// ALP SPEC: This seems undoable if we do not know the interface (per protocol specific support)
//  which is still a pretty legitimate policy on a low power protocol.
pub struct NonOverloadedIndirectInterface {
    pub interface_file_id: u8,
    // ALP SPEC: Where is this defined? Is this ID specific?

    #[deku(update = "self.data.len()")]
    length: Length,

    #[deku(count = "length", endian = "big")]
    pub data: Vec<u8>,
}

// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Clone, Debug, PartialEq)]
// pub enum IndirectInterface {
//     Overloaded(OverloadedIndirectInterface),
//     NonOverloaded(NonOverloadedIndirectInterface),
// }


#[cfg(test)]
mod test {
    use hex_literal::hex;

    use crate::test_tools::test_item;
    use super::*;

    #[test]
    fn test_overloaded_indirect_interface() {
        test_item(
            OverloadedIndirectInterface::new(
                4,
                NlsMethod::AesCcm32,
                0xFF,
                Address::Vid(0xABCD),
            ),
            &hex!("04 37 FF ABCD"),
            (&[], 0)
        )
    }

}

