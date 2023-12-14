use deku::prelude::*;

use super::InterfaceType;
use crate::{network::Addressee, physical::Channel};

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "interface_id: InterfaceType, length: u32", id = "interface_id")]
pub enum InterfaceFinalStatus {
    #[deku(id = "InterfaceType::Dash7")]
    Dash7(InterfaceFinalStatusCode),

    #[deku(id_pat = "_")]
    Other(#[deku(count = "length")] Vec<u8>),
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(type = "u8")]
pub enum InterfaceFinalStatusCode {
    /// No error
    No = 0,
    /// Resource busy
    Busy = 0xFF,
    /// bad parameter
    BadParam = 0xFE,
    /// duty cycle limit overflow
    DutyCycle = 0xFD,
    /// cca timeout
    CcaTo = 0xFC,
    /// security frame counter overflow
    NlsKey = 0xFB,
    /// tx stream underflow
    TxUdf = 0xFA,
    /// rx stream overflow
    RxOvf = 0xF9,
    /// rx checksum
    RxCrc = 0xF8,
    /// abort
    Abort = 0xF7,
    /// no ack received
    NoAck = 0xF6,
    /// rx timeout
    RxTo = 0xF5,
    /// not supported band
    NotSupportedBand = 0xF4,
    /// not supported channel
    NotSupportedChannel = 0xF3,
    /// not supported modulation
    NotSupportedModulation = 0xF2,
    /// no channels in list
    VoidChannelList = 0xF1,
    /// not supported packet length
    NotSupportedLen = 0xF0,
    /// parameter overflow
    ParamOvf = 0xEF,
    /// vid used without nls
    VidWoNls = 0xEE,
    /// tx scheduling late
    TxSched = 0xED,
    /// rx scheduling late
    RxSched = 0xEC,
    /// parameter overflow
    BufferOvf = 0xEB,
    /// mode not supported
    NotSupportedMode = 0xEA,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "interface_id: InterfaceType, length: u32", id = "interface_id")]
pub enum InterfaceTxStatus {
    #[deku(id = "InterfaceType::Dash7")]
    Dash7(Dash7InterfaceTxStatus),

    #[deku(id_pat = "_")]
    Other(#[deku(count = "length")] Vec<u8>),
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct Dash7InterfaceTxStatus {
    /// PHY layer channel header
    pub channel: Channel,
    /// Target power in dBm
    pub target_rx_level: i8,
    /// D7A Error
    pub error: InterfaceFinalStatusCode,
    /// End transmission date using the local RTC time stamp
    #[deku(pad_bits_before = "24")]
    pub lts: u32,
    /// Addressee
    pub addressee: Addressee,
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use crate::{
        link::AccessClass,
        network::{Address, Addressee, NlsState},
        physical::{Channel, ChannelBand, ChannelClass, ChannelCoding, ChannelHeader},
        test_tools::test_item,
        transport::GroupCondition,
    };

    use super::*;

    #[test]
    fn test_interface_tx_status() {
        test_item(
            Dash7InterfaceTxStatus {
                channel: Channel {
                    header: ChannelHeader::new(
                        ChannelBand::Rfu0,
                        ChannelClass::LoRate,
                        ChannelCoding::Rfu,
                    ),
                    index: 0x0123,
                },
                target_rx_level: 2,
                error: InterfaceFinalStatusCode::Busy,
                lts: 0x0708_0000,
                addressee: Addressee::new(
                    false,
                    GroupCondition::Any,
                    Address::Vid(0x0011),
                    NlsState::AesCcm64([0; 5]),
                    AccessClass::new(0x0F, 0x0F),
                ),
            },
            &hex!("01 0123 02 FF 00 00 00 0000 0807 36 FF 0011 0000000000"),
        )
    }
}
