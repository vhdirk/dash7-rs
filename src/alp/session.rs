use deku::prelude::*;

use crate::alp::{network::Addressee, physical::Channel, varint::VarInt};

/// The Response Modes define the condition for termination on success of a Request
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
#[deku(bits = 3, type = "u8")]
pub enum ResponseMode {
    /// A Request is acknowledged if the DLL CSMA-CA routine succeeds. No
    /// responses are expected.
    ///
    /// Eg. The request is successful if your packet was successfully sent on the radio.
    #[default]
    #[deku(id = "0")]
    No,

    /// If the addressee is broadcast, a Request is acknowledged if as many as
    /// possible D7ATP responses to this Request are received (may be zero).
    ///
    /// If the addressee is unicast, a Request is acknowledged if the addressee provides a
    /// D7ATP response. All responses received during the D7ATP Receive Period
    /// are vectored to upper layer.
    ///
    /// Eg. Succeeds if everyone addressed responds to the radio packet.
    #[deku(id = "1")]
    All,

    /// A Request is acknowledged if at least one D7ATP response to this Request is
    /// received.
    ///
    /// Eg. Succeeds if you receive one response to the radio packet.
    #[deku(id = "2")]
    Any,

    /// A Request is acknowledged if the DLL CSMA-CA routine succeeds REPEAT
    /// times. No responses are expected. The parameters REPEAT is defined in the
    /// SEL configuration file.
    #[deku(id = "4")]
    NoRepeat,

    /// A Request is acknowledged if the DLL CSMA-CA routine succeeds. It is un-
    /// acknowledged when a response does not acknowledge the Request. The
    /// procedure behaves as RESP_ALL, but Responders provide responses only
    /// when their D7ATP ACK Templates is not void or if the upper layer provides a
    /// response.
    ///
    /// Eg. Succeeds only if the responder gives back an ALP packet in response (which is more
    /// restrictive than succeeding upon successful radio ACK).
    #[deku(id = "5")]
    OnError,

    /// A Request is acknowledged if at least one D7ATP response to this Request is
    /// received. The procedure behaves as RESP_ANY, but the Addressee is
    /// managed dynamically. It is set to broadcast after failure to receive an
    /// acknowledgement. On acknowledgement success, it is set to the
    /// Addressee of one of the responders that acknowledged the Request (preferred
    /// addressee). The preferred addressee selection is implementation dependent.
    #[deku(id = "6")]
    Preferred,
}

/// The Retry Modes define the pattern for re-flushing a FIFO that terminates on error.
///
/// In other words, what is the retry policy when sending your payload.
#[cfg(feature = "spec_v1_2")]
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
#[deku(bits = 3, type = "u8")]
pub enum RetryMode {
    #[default]
    #[deku(id = "0")]
    No,
}

#[cfg(feature = "wizzilab_v5_3")]
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
#[deku(bits = 3, type = "u8")]
pub enum RetryMode {
    #[default]
    #[deku(id = "0")]
    No,
    #[deku(id = "1")]
    OneshotRetry,
    #[deku(id = "2")]
    FifoFast,
    #[deku(id = "3")]
    FifoSlow,
    #[deku(id = "4")]
    SingleFast,
    #[deku(id = "5")]
    SingleSlow,
    #[deku(id = "6")]
    OneshotSticky,
    #[deku(id = "7")]
    Rfu7,
}

/// QoS of the request
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct QoS {
    #[deku(bits = 1)]
    pub stop_on_error: bool,
    #[deku(bits = 1)]
    pub record: bool,
    pub retry_mode: RetryMode,
    pub response_mode: ResponseMode,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Dash7InterfaceConfiguration {
    pub qos: QoS,
    pub dormant_session_timeout: VarInt,
    pub addressee: Addressee,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct LoRaWANInterfaceConfiguration {
    #[deku(pad_bits_before = "5", bits = 1)]
    pub adr_enabled: bool,
    #[deku(bits = 1)]
    pub request_ack: bool,

    #[deku(pad_bits_before = "1")]
    pub application_port: u8,
    pub data_rate: u8,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct LoRaWANOTAAInterfaceConfiguration {
    pub base: LoRaWANInterfaceConfiguration,

    #[deku(count = "8")]
    pub device_eui: Vec<u8>,

    #[deku(count = "8")]
    pub app_eui: Vec<u8>,

    #[deku(count = "16")]
    pub app_key: Vec<u8>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct LoRaWANABPInterfaceConfiguration {
    pub base: LoRaWANInterfaceConfiguration,

    #[deku(count = "16")]
    pub network_session_key: Vec<u8>,

    #[deku(count = "16")]
    pub app_session_key: Vec<u8>,

    pub device_address: u32,

    pub network_id: u32,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceStatus {
    /// PHY layer channel
    pub channel: Channel,
    /// PHY layer RX level in -dBm
    pub rx_level: u8,
    /// PHY layer link budget in dB
    pub link_budget: u8,

    pub target_rx_level: u8,
    #[deku(bits = 1)]
    pub nls: bool,
    #[deku(bits = 1)]
    pub missed: bool,
    #[deku(bits = 1)]
    pub retry: bool,
    #[deku(bits = 1)]
    pub unicast: bool,

    /// Value of the D7ATP Dialog ID
    #[deku(pad_bits_before = "4")]
    pub fifo_token: u8,

    /// Value of the D7ATP Transaction ID
    pub sequence_number: u8,

    /// Response delay (request to response time) in TiT
    pub response_timeout: VarInt,

    /// Address of source
    pub addressee: Addressee,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_tools::test_item;
    use hex_literal::hex;

    #[test]
    fn test_qos() {
        test_item(QoS::default(), &[0]);

        test_item(
            QoS {
                retry_mode: RetryMode::No,
                response_mode: ResponseMode::Any,
                record: true,
                stop_on_error: true,
            },
            &[0b11000010],
        );

        test_item(
            QoS {
                retry_mode: RetryMode::No,
                response_mode: ResponseMode::NoRepeat,
                record: false,
                stop_on_error: false,
            },
            &hex!("04"),
        )
    }
}
