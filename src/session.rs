
use deku::prelude::*;

use crate::{network::Addressee, physical::ChannelId, varint::VarInt};

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 3, type="u8")]
pub enum ResponseMode {
    #[deku(id = "0")] No,
    #[deku(id = "1")] All,
    #[deku(id = "2")] Any,
    #[deku(id = "4")] NoRepeat,
    #[deku(id = "5")] OnError,
    #[deku(id = "6")] Preferred,
}

/// The Retry Modes define the pattern for re-flushing a FIFO that terminates on error.
///
/// In other words, what is the retry policy when sending your payload.
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 3, type="u8")]
pub enum RetryMode {
    No = 0,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct QoS {
    #[deku(bits=1)]
    pub stop_on_error: bool,
    #[deku(bits=1)]
    pub record: bool,
    pub retry_mode: RetryMode,
    pub response_mode: ResponseMode,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct Dash7InterfaceConfiguration {
    pub qos: QoS,
    pub dormant_session_timeout: VarInt,
    pub addressee: Addressee,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    pub channel_id: ChannelId,
    pub rx_level: u8,
    pub link_budget: u8,
    pub target_rx_level: u8,
    #[deku(bits=1)]
    pub nls: bool,
    #[deku(bits=1)]
    pub missed: bool,
    #[deku(bits=1)]
    pub retry: bool,
    #[deku(bits=1)]
    pub unicast: bool,

    #[deku(pad_bits_before = "4")]
    pub fifo_token: u8,
    pub sequence_number: u8,
    pub response_timeout: VarInt,

    pub addressee: Addressee,
}
