use modular_bitfield::prelude::*;

use crate::{types::CompressedValue, physical::ChannelId, network::Addressee};

#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
#[bits = 3]
pub enum ResponseMode {
    No = 0,
    All = 1,
    Any = 2,
    NoRpt = 4,
    OnError = 5,
    Preferred = 6,
}

/// The Retry Modes define the pattern for re-flushing a FIFO that terminates on error.
///
/// In other words, what is the retry policy when sending your payload.

#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
#[bits = 3]
pub enum RetryMode {
    No = 0,
}

#[bitfield]
#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
pub struct QoS {
    pub stop_on_error: bool,
    pub record: bool,
    pub retry_mode: RetryMode,
    pub response_mode: ResponseMode,
}

#[bitfield]
#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
pub struct Dash7InterfaceConfiguration {
    pub qos: QoS,
    pub dormant_session_timeout: CompressedValue,
    pub addressee: Addressee,
}

#[bitfield]
#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
pub struct Status {
    pub channel_id: ChannelId,
    pub rx_level: u8,
    pub link_budget: u8,
    pub target_rx_level: u8,
    pub nls: bool,
    pub missed: bool,
    pub retry: bool,
    pub unicast: bool,
    #[skip] __: B4,
    pub fifo_token: u8,
    pub sequence_number: u8,
    pub response_timeout: CompressedValue,

    // TODO: does not fit
    pub addressee: B48,
}





