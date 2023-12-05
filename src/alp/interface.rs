use deku::prelude::*;

use super::{session::QoS, network::{Address, NlsMethod, AddressType}};

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
