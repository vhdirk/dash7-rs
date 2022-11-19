use packed_struct::prelude::*;

use crate::{types::CompressedValue, physical::ChannelId};

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum ResponseMode {
    No = 0,
    All = 1,
    Any = 2,
    NoRpt = 4,
    OnError = 5,
    Preferred = 6,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum RetryMode {
    No = 0,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct QoS {

    #[packed_field(bits = "0")]
    pub stop_on_error: bool,

    #[packed_field(bits = "1")]
    pub record: bool,

    #[packed_field(bits = "2..=4", ty = "enum")]
    pub retry_mode: RetryMode,

    #[packed_field(bits = "5..=7", ty = "enum")]
    pub response_mode: ResponseMode,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct Dash7InterfaceConfiguration {
    #[packed_field(element_size_bytes = "1")]
    pub qos: QoS,

    #[packed_field(element_size_bytes = "1")]
    pub dormant_session_timeout: CompressedValue,

    #[packed_field(element_size_bytes = "7")]
    pub addressee: Addressee,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct Status {
    #[packed_field(element_size_bytes = "3")]
    pub channel_id: ChannelId,

    pub rx_level: u8,
    pub link_budget: u8,
    pub target_rx_level: u8,

    #[packed_field(bits = "0")]
    pub nls: bool,

    #[packed_field(bits = "1")]
    pub missed: bool,

    #[packed_field(bits = "2")]
    pub retry: bool,

    #[packed_field(bits = "3")]
    pub unicast: bool,

    pub fifo_token: u8,
    pub sequence_number: u8,

    #[packed_field(element_size_bytes = "1")]
    pub response_timeout: CompressedValue,
    pub addressee: Addressee,
}





