use deku::prelude::*;

use crate::app::command::Command;

#[derive(DekuRead, DekuWrite, Debug, Clone, Copy, PartialEq)]
#[deku(type = "u8")]
pub enum SerialMessageType {
    AlpData = 1,
    PingRequest = 2,
    PingResponse = 3,
    Logging = 4,
    Rebooted = 5,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "message_id: SerialMessageType, length: u32", id = "message_id")]
pub enum SerialMessage {
    #[deku(id = "SerialMessageType::AlpData")]
    AlpData(#[deku(ctx = "length")] Command),

    #[deku(id = "SerialMessageType::PingRequest")]
    PingRequest,

    #[deku(id = "SerialMessageType::PingResponse")]
    PingResponse,

    #[deku(id = "SerialMessageType::Rebooted")]
    Rebooted,

    #[deku(id = "SerialMessageType::Logging")]
    Logging(#[deku(bytes_read = "length")] Vec<u8>),
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(magic = b"\xC0")]
pub struct SerialFrameHeader {
    version: u8,
    counter: u8,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct SerialFrame {
    header: SerialFrameHeader,

    message_type: SerialMessageType,

    length: u8,
    crc: u16,

    #[deku(ctx = "(*message_type, Into::<u32>::into(*length))")]
    message: SerialMessage,
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::interface::InterfaceConfiguration;
    use crate::app::operand::{ActionHeader, FileData, FileOffset, RequestTag};
    use crate::file::File;
    use crate::link::AccessClass;
    use crate::network::{Addressee, NlsState};
    use crate::session::ResponseMode;
    use crate::{
        app::{action::Action, interface::Dash7InterfaceConfiguration, operand::Forward},
        session::{QoS, RetryMode},
        test_tools::test_item,
    };
    use hex_literal::hex;

    #[test]
    fn test_return_file_data_with_qos_unicast() {
        let command = Command::new(vec![
            Action::RequestTag(RequestTag { eop: true, id: 2 }),
            Action::Forward(Forward::new(
                false,
                InterfaceConfiguration::Dash7(Dash7InterfaceConfiguration {
                    qos: QoS {
                        record: false,
                        response_mode: ResponseMode::All,
                        stop_on_error: false,
                        retry_mode: RetryMode::No,
                    },
                    dormant_session_timeout: 0.into(),
                    addressee: Addressee::new(
                        crate::network::Address::Uid(2656824718681607041),
                        NlsState::None,
                        AccessClass::new(0, 0),
                    ),
                }),
            )),
            Action::ReturnFileData(FileData::new(
                ActionHeader::new(false, false),
                FileOffset::no_offset(64),
                File::Other(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
            )),
        ]);
        test_item(
            SerialFrame {
                header: SerialFrameHeader {
                    version: 0,
                    counter: 0,
                },

                message_type: SerialMessageType::AlpData,
                length: 30,
                crc: 41991,
                message: SerialMessage::AlpData(command),
            },
            &hex!("c00000011e07a4b40232d70100200024def001537e8b812040000a0102030405060708090a"),
        );
    }
}
