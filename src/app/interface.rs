use deku::prelude::*;

use crate::{
    network::Addressee,
    session::{InterfaceType, QoS},
    types::VarInt,
};

/// Section 9.2.1
///
/// Parameters to handle the sending of a request.
// ALP SPEC: Add link to D7a section
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct Dash7InterfaceConfiguration {
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
    pub dormant_session_timeout: VarInt,

    /// Response Execution Delay in Compressed Format, unit is in milliseconds.
    ///
    /// Time given to the target to process the request.
    #[cfg(not(feature = "_subiot"))]
    pub te: VarInt,

    /// Address of the target.
    pub addressee: Addressee,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct LoRaWANInterfaceConfiguration {
    #[deku(pad_bits_before = "5", bits = 1)]
    pub adr_enabled: bool,
    #[deku(bits = 1)]
    pub request_ack: bool,

    #[deku(pad_bits_before = "1")]
    pub application_port: u8,
    pub data_rate: u8,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct LoRaWANOTAAInterfaceConfiguration {
    pub base: LoRaWANInterfaceConfiguration,

    #[deku(count = "8")]
    pub device_eui: Vec<u8>,

    #[deku(count = "8")]
    pub app_eui: Vec<u8>,

    #[deku(count = "16")]
    pub app_key: Vec<u8>,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct LoRaWANABPInterfaceConfiguration {
    pub base: LoRaWANInterfaceConfiguration,

    #[deku(count = "16")]
    pub network_session_key: Vec<u8>,

    #[deku(count = "16")]
    pub app_session_key: Vec<u8>,

    pub device_address: u32,

    pub network_id: u32,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "interface_type: InterfaceType", id = "interface_type")]
pub enum InterfaceConfiguration {
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
    Unknown,
}

#[cfg(test)]
mod test {
    use crate::{
        link::AccessClass,
        network::{Address, NlsState},
        session::{ResponseMode, RetryMode},
        test_tools::test_item,
        transport::GroupCondition,
    };

    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_dash7_interface_configuration() {
        test_item(
            Dash7InterfaceConfiguration {
                qos: QoS {
                    retry_mode: RetryMode::No,
                    response_mode: ResponseMode::Any,
                    stop_on_error: false,
                    record: false,
                },
                dormant_session_timeout: 0x20.into(),

                #[cfg(not(feature = "_subiot"))]
                te: 0x34.into(),

                addressee: Addressee::new(
                    false,
                    GroupCondition::Any,
                    Address::Vid(0xABCD),
                    NlsState::AesCcm32([1, 2, 3, 4, 5]),
                    AccessClass::new(0x0F, 0x0F),
                ),
            },
            #[cfg(not(feature = "_subiot"))]
            &hex!("02 28 2D 37 FF ABCD 0102030405"),
            #[cfg(feature = "_subiot")]
            &hex!("02 28 37 FF ABCD 0102030405"),
        )
    }

    #[test]
    fn test_dash7_interface_configuration_with_address_nbid() {
        test_item(
            Dash7InterfaceConfiguration {
                qos: QoS {
                    retry_mode: RetryMode::No,
                    response_mode: ResponseMode::Any,
                    stop_on_error: false,
                    record: false,
                },
                dormant_session_timeout: 0x20.into(),

                #[cfg(not(feature = "_subiot"))]
                te: 0x34.into(),

                addressee: Addressee::new(
                    true,
                    GroupCondition::NotEqual,
                    Address::NbId(0x15.into()),
                    NlsState::None,
                    AccessClass::default(),
                ),
            },
            #[cfg(not(feature = "_subiot"))]
            &hex!("02 28 2D 48 00 15"),
            #[cfg(feature = "_subiot")]
            &hex!("02 28 48 00 15"),
        )
    }
    #[test]
    fn test_dash7_interface_configuration_with_address_noid() {
        test_item(
            Dash7InterfaceConfiguration {
                qos: QoS {
                    retry_mode: RetryMode::No,
                    response_mode: ResponseMode::Any,
                    stop_on_error: false,
                    record: false,
                },
                dormant_session_timeout: 0x20.into(),

                #[cfg(not(feature = "_subiot"))]
                te: 0x34.into(),

                addressee: Addressee::new(
                    false,
                    GroupCondition::Equal,
                    Address::NoId,
                    NlsState::AesCbcMac128([0x0A, 0x0B, 0x0C, 0x0D, 0x0E]),
                    AccessClass::new(0x02, 0x04),
                ),
            },
            #[cfg(not(feature = "_subiot"))]
            &hex!("02 28 2D 92 24 0A 0B 0C 0D 0E"),
            #[cfg(feature = "_subiot")]
            &hex!("02 28 92 24 0A 0B 0C 0D 0E"),
        )
    }

    #[test]
    fn test_dash7_interface_configuration_with_address_uid() {
        test_item(
            Dash7InterfaceConfiguration {
                qos: QoS {
                    retry_mode: RetryMode::No,
                    response_mode: ResponseMode::Any,
                    stop_on_error: false,
                    record: false,
                },
                dormant_session_timeout: 0x20.into(),

                #[cfg(not(feature = "_subiot"))]
                te: 0x34.into(),

                addressee: Addressee::new(
                    true,
                    GroupCondition::GreaterThan,
                    Address::Uid(0x0001020304050607),
                    NlsState::AesCcm64([0xA1, 0xA2, 0xA3, 0xA4, 0xA5]),
                    AccessClass::new(0x04, 0x08),
                ),
            },
            #[cfg(not(feature = "_subiot"))]
            &hex!("02 28 2D EE 48 0001020304050607 A1A2A3A4A5"),
            #[cfg(feature = "_subiot")]
            &hex!("02 28 EE 48 0001020304050607 A1A2A3A4A5"),
        )
    }

    #[test]
    fn test_dash7_interface_configuration_with_address_vid() {
        test_item(
            Dash7InterfaceConfiguration {
                qos: QoS {
                    retry_mode: RetryMode::No,
                    response_mode: ResponseMode::Any,
                    stop_on_error: false,
                    record: false,
                },
                dormant_session_timeout: 0x20.into(),

                #[cfg(not(feature = "_subiot"))]
                te: 0x34.into(),

                addressee: Addressee::new(
                    false,
                    GroupCondition::Any,
                    Address::Vid(0xABCD),
                    NlsState::AesCcm32([0xA1, 0xA2, 0xA3, 0xA4, 0xA5]),
                    AccessClass::new(0x0F, 0x0F),
                ),
            },
            #[cfg(not(feature = "_subiot"))]
            &hex!("02 28 2D 37 FF AB CD A1A2A3A4A5"),
            #[cfg(feature = "_subiot")]
            &hex!("02 28 37 FF AB CD A1A2A3A4A5"),
        )
    }
}
