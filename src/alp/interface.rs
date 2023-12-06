use deku::prelude::*;

use super::{
    network::{Address, AddressType, NlsMethod, Addressee},
    session::{QoS, Dash7InterfaceConfiguration, LoRaWANABPInterfaceConfiguration, LoRaWANOTAAInterfaceConfiguration},
};

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

    #[deku(update = "self.address.deku_id().unwrap()")]
    address_type: AddressType,

    /// Use VID instead of UID when possible
    #[deku(bits = 1)]
    pub use_vid: bool,

    /// Security method
    pub nls_method: NlsMethod,

    /// Access class of the targeted listening device
    pub access_class: u8,

    /// Address of the target.
    #[deku(ctx = "*address_type")]
    pub address: Address,
}

impl InterfaceConfiguration {
    pub fn new(
        qos: QoS,
        to: u8,
        te: u8,
        group_condition: GroupCondition,
        use_vid: bool,
        nls_method: NlsMethod,
        access_class: u8,
        address: Address,
    ) -> Self {
        Self {
            qos,
            to,
            te,
            group_condition,
            address_type: address.deku_id().unwrap(),
            use_vid,
            nls_method,
            access_class,
            address,
        }
    }
}


#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[deku(bits = 8, type = "u8")]
pub enum InterfaceType {

    #[deku(id = "0x00")]
    Host,

    #[deku(id = "0x01")]
    Serial,

    #[deku(id = "0x02")]
    LoRaWanABP,

    #[deku(id = "0x03")]
    LoRaWanOTAA,

    #[deku(id = "0xD7")]
    Dash7,

    #[deku(id_pat = "_")]
    Other
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(type = "InterfaceType")]
pub enum InterfaceConfigurationOverload {

    #[deku(id = "InterfaceType::Host")]
    Host,

    #[deku(id = "InterfaceType::Serial")]
    Serial,

    #[deku(id = "InterfaceType::LoRaWanABP")]
    LoRaWanABP(LoRaWANABPInterfaceConfiguration),

    #[deku(id = "InterfaceType::LoRaWanOTAA")]
    LoRaWanOTAA(LoRaWANOTAAInterfaceConfiguration),

    #[deku(id = "InterfaceType::Dash7")]
    Dash7(Dash7InterfaceConfiguration),

    #[deku(id_pat = "_")]
    Other,
}


#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(ctx = "overloaded: bool", id = "overloaded")]
pub enum IndirectInterface {
    #[deku(id = "true")]
    Overloaded(InterfaceConfigurationOverload),
    #[deku(id = "false")]
    NonOverloaded(InterfaceType),
}





#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;
    use crate::test_tools::test_item;

    // #[test]
    // fn test_overloaded_indirect_interface() {
    //     test_item(
    //         OverloadedIndirectInterface::new(
    //             4,
    //             NlsMethod::AesCcm32,
    //             0xFF,
    //             Address::Vid(0xABCD),
    //         ),
    //         &hex!("04 37 FF ABCD"),
    //     )
    // }
}
