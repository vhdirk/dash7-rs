#[cfg(feature = "std")]
use std::fmt::{self, Display};

#[cfg(not(feature = "std"))]
use alloc::fmt;

use deku::{
    bitvec::{BitSlice, BitVec, BitView, Msb0},
    prelude::*,
};

use crate::{session::InterfaceStatus, utils::pad_rest};

use super::{
    action::Action,
    operand::{
        RequestTag, ResponseTag, Status,
        StatusOperand,
    },
};

#[derive(DekuRead, DekuWrite, Clone, Debug, PartialEq, Default)]
#[deku(ctx = "length: u32")]
pub struct Command {
    // we cannot process an indirect forward without knowing the interface type, which is stored in the interface file
    // as identified by the indirectforward itself
    // As such, we HAVE to bail here
    // Hopefully this will be addressed in SPEC 1.3
    #[deku(until = "|action: &Action| { action.deku_id().unwrap() == OpCode::IndirectForward }")]
    // Always stop reading when length is reached
    #[deku(bytes_read = "length")]
    pub actions: Vec<Action>,
}

/// Stub implementation so we can implement DekuContainerRead
impl<'a> DekuRead<'a, ()> for Command {
    fn read(_: &'a BitSlice<u8, Msb0>, _: ()) -> Result<(&'a BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        unreachable!("This should not have been called")
    }
}

/// Stub implementation so we can implement DekuContainerWrite
impl DekuWrite<()> for Command {
    fn write(&self, _: &mut BitVec<u8, Msb0>, _: ()) -> Result<(), DekuError> {
        unreachable!("This should not have been called")
    }
}

impl Command {
    pub fn new(actions: Vec<Action>) -> Self {
        // TODO: validate actions
        Self { actions }
    }

    pub fn interface_status(&self) -> Option<&InterfaceStatus> {
        for action in self.actions.iter() {
            if let Action::Status(StatusOperand { status, .. }) = &action {
                if let Status::Interface(iface_status) = status {
                    return Some(&iface_status.status);
                }
            }
        }
        None
    }

    // TODO: a generator would be great here
    pub fn actions_without_interface_status(&self) -> Vec<&Action> {
        let mut actions = vec![];
        for action in self.actions.iter() {
            if let Action::Status(StatusOperand { status, .. }) = &action {
                if let Status::Interface(_) = status {
                    continue;
                }
            }
            actions.push(action);
        }
        actions
    }

    pub fn request_tag(&self) -> Option<&RequestTag> {
        for action in self.actions.iter() {
            if let Action::RequestTag(operand) = action {
                return Some(operand);
            }
        }
        None
    }

    pub fn request_id(&self) -> Option<u8> {
        self.request_tag().map(|t| t.id)
    }

    pub fn response_tag(&self) -> Option<&ResponseTag> {
        for action in self.actions.iter() {
            if let Action::ResponseTag(operand) = action {
                return Some(operand);
            }
        }
        None
    }

    pub fn response_id(&self) -> Option<u8> {
        self.response_tag().map(|t| t.id)
    }

    pub fn tag_id(&self) -> Option<u8> {
        self.request_id().or(self.response_id())
    }

    pub fn is_last_response(&self) -> bool {
        for action in self.actions.iter() {
            if let Action::ResponseTag(ResponseTag { eop, .. }) = action {
                return *eop;
            }
        }
        false
    }
}

impl<'a> DekuContainerRead<'a> for Command {
    fn from_bytes(input: (&'a [u8], usize)) -> Result<((&'a [u8], usize), Self), DekuError> {
        let input_bits = input.0.view_bits::<Msb0>();
        let size = (input_bits.len() - input.1) as u32 / u8::BITS;
        let (rest, value) = Self::read(&input_bits[input.1..], size)?;

        Ok((pad_rest(input_bits, rest), value))
    }
}

/// Stub implementation so we can implement DekuContainerWrite
impl DekuContainerWrite for Command {
    fn to_bytes(&self) -> Result<Vec<u8>, DekuError> {
        let output = self.to_bits()?;
        Ok(output.into_vec())
    }

    fn to_bits(&self) -> Result<BitVec<u8, Msb0>, DekuError> {
        let mut output: BitVec<u8, Msb0> = BitVec::new();
        self.write(&mut output, u32::MAX)?;
        Ok(output)
    }
}

impl TryFrom<&'_ [u8]> for Command {
    type Error = DekuError;
    fn try_from(input: &'_ [u8]) -> Result<Self, Self::Error> {
        let (rest, res) = <Self as DekuContainerRead>::from_bytes((input, 0))?;
        if !rest.0.is_empty() {
            return Err(DekuError::Parse({
                let res = fmt::format(format_args!("Too much data"));
                res
            }));
        }
        Ok(res)
    }
}

impl TryFrom<Command> for Vec<u8> {
    type Error = DekuError;
    fn try_from(input: Command) -> Result<Self, Self::Error> {
        DekuContainerWrite::to_bytes(&input)
    }
}

#[cfg(feature = "std")]
impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag_str = self
            .tag_id()
            .map_or("".to_string(), |t| format!("with tag {} ", t));
        f.write_str(&format!("Command {}", &tag_str))?;

        let status = if let Some(operand) = self.response_tag() {
            if operand.eop {
                if operand.error {
                    "completed, with error"
                } else {
                    "completed, without error"
                }
            } else {
                "executing"
            }
        } else {
            "executing"
        };

        f.write_str(&format!("({})", status))?;

        let actions = self.actions_without_interface_status();
        if actions.len() > 0 {
            f.write_str("\n\tactions:\n")?;

            for action in actions.iter() {
                f.write_str(&format!("\t\t{:?}\n", action))?;
            }
        }

        if let Some(interface_status) = self.interface_status() {
            if actions.len() > 0 {
                f.write_str("\n")?;
            }

            f.write_str(&format!("\tinterface status: {}\n", interface_status))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use hex_literal::hex;

    use super::*;
    #[cfg(feature = "_wizzilab")]
    use crate::transport::GroupCondition;
    use crate::{
        app::{
            interface::InterfaceConfiguration,
            operand::{ActionHeader, FileData, FileOffset, Forward, Nop, ReadFileData, Status},
        },
        file::File,
        link::AccessClass,
        network::{Address, Addressee, NlsState},
        physical::{Channel, ChannelBand, ChannelClass, ChannelCoding, ChannelHeader},
        session::{Dash7InterfaceStatus, InterfaceStatus},
        test_tools::test_item,
    };

    #[test]
    fn test_command() {
        let cmd = Command {
            actions: vec![
                Action::RequestTag(RequestTag { id: 66, eop: true }),
                Action::ReadFileData(ReadFileData {
                    header: ActionHeader {
                        response: true,
                        group: false,
                    },
                    offset: FileOffset {
                        file_id: 0,
                        offset: 0u32.into(),
                    },
                    length: 8u32.into(),
                }),
                Action::ReadFileData(ReadFileData {
                    header: ActionHeader {
                        response: false,
                        group: true,
                    },
                    offset: FileOffset {
                        file_id: 4,
                        offset: 2u32.into(),
                    },
                    length: 3u32.into(),
                }),
                Action::Nop(Nop {
                    header: ActionHeader {
                        response: true,
                        group: true,
                    },
                }),
            ],
        };
        let data = &hex!("B4 42 41 00 00 08 81 04 02 03 C0");

        test_item(cmd, data);
    }

    #[test]
    fn test_command_request_id() {
        assert_eq!(
            Command {
                actions: vec![
                    Action::RequestTag(RequestTag { eop: true, id: 66 }),
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: true
                        }
                    })
                ]
            }
            .request_id(),
            Some(66)
        );
        assert_eq!(
            Command {
                actions: vec![
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        }
                    }),
                    Action::RequestTag(RequestTag { eop: true, id: 44 }),
                ]
            }
            .request_id(),
            Some(44)
        );
        assert_eq!(
            Command {
                actions: vec![
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        }
                    }),
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        }
                    })
                ]
            }
            .request_id(),
            None
        );
    }

    #[test]
    fn test_command_response_id() {
        assert_eq!(
            Command {
                actions: vec![
                    Action::ResponseTag(ResponseTag {
                        eop: true,
                        error: true,
                        id: 66
                    }),
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: true
                        }
                    })
                ]
            }
            .response_id(),
            Some(66)
        );
        assert_eq!(
            Command {
                actions: vec![
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        }
                    }),
                    Action::ResponseTag(ResponseTag {
                        eop: true,
                        error: true,
                        id: 44
                    }),
                ]
            }
            .response_id(),
            Some(44)
        );
        assert_eq!(
            Command {
                actions: vec![
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        }
                    }),
                    Action::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        }
                    })
                ]
            }
            .response_id(),
            None
        );
    }

    #[test]
    fn test_command_is_last_response() {
        assert!(Command {
            actions: vec![
                Action::ResponseTag(ResponseTag {
                    eop: true,
                    error: true,
                    id: 66
                }),
                Action::Nop(Nop {
                    header: ActionHeader {
                        group: true,
                        response: true
                    }
                })
            ]
        }
        .is_last_response());

        assert!(!Command {
            actions: vec![
                Action::ResponseTag(ResponseTag {
                    eop: false,
                    error: false,
                    id: 66
                }),
                Action::Nop(Nop {
                    header: ActionHeader {
                        group: true,
                        response: true
                    }
                })
            ]
        }
        .is_last_response());

        assert!(!Command {
            actions: vec![
                Action::ResponseTag(ResponseTag {
                    eop: false,
                    error: true,
                    id: 44
                }),
                Action::ResponseTag(ResponseTag {
                    eop: true,
                    error: true,
                    id: 44
                }),
            ]
        }
        .is_last_response());

        assert!(!Command {
            actions: vec![
                Action::Nop(Nop {
                    header: ActionHeader {
                        group: true,
                        response: false
                    }
                }),
                Action::Nop(Nop {
                    header: ActionHeader {
                        group: true,
                        response: false
                    }
                })
            ]
        }
        .is_last_response());
    }

    #[cfg(feature = "subiot_v0_0")]
    #[test]
    fn test_simple_received_return_file_data_command_subiot_v0_0() {
        let data = [
            0x62u8, // Interface Status action
            0xD7,   // D7ASP interface
            32,     // channel header
            0, 16,   // channel_id
            70,   // rxlevel (- dBm)
            80,   // link budget
            80,   // target rx level
            0,    // status
            200,  // fifo token
            0,    // seq
            20,   // response timeout
            0x10, // addressee ctrl (NOID)
            0,    // access class
            0x20, // action=32/ReturnFileData
            0x51, // File ID
            0x00, // offset
            0x0b, // length
            0x48, 0x65, 0x6c, 0x6c, 0x6f, // Hello
            0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, // World
        ];

        let item = Command {
            actions: vec![
                Action::Status(
                    Status::Interface(
                        InterfaceStatus::Dash7(Dash7InterfaceStatus {
                            channel: Channel {
                                header: ChannelHeader::new(
                                    ChannelBand::Band433,
                                    ChannelClass::LoRate,
                                    ChannelCoding::Pn9,
                                ),
                                index: 16,
                            },
                            rx_level: 70,
                            link_budget: 80,
                            target_rx_level: 80,
                            nls: false,
                            missed: false,
                            retry: false,
                            unicast: false,
                            fifo_token: 200,
                            sequence_number: 0,
                            response_timeout: 20u32.into(),
                            addressee: Addressee::default(),
                        })
                        .into(),
                    )
                    .into(),
                ),
                Action::ReturnFileData(FileData::new(
                    ActionHeader::default(),
                    FileOffset::no_offset(0x51),
                    File::Other("Hello world".into()),
                )),
            ],
        };

        test_item(item, &data);
    }

    #[cfg(feature = "subiot_v0_0")]
    #[test]
    fn test_simple_received_return_file_data_command_with_tag_request() {
        let data = [
            0xB4u8, // tag request with send response bit set
            25,     // tag ID
            0x62,   // Interface Status action
            0xD7,   // D7ASP interface
            32,     // channel header
            0, 16,   // channel_id
            70,   // rxlevel (- dBm)
            80,   // link budget
            80,   // target rx level
            0,    // status
            200,  // fifo token
            0,    // seq
            20,   // response timeout
            0x10, // addressee ctrl (NOID)
            0,    // access class
            0x20, // action=32/ReturnFileData
            0x51, // File ID
            0x00, // offset
            0x0b, // length
            0x48, 0x65, 0x6c, 0x6c, 0x6f, // Hello
            0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, // World
        ];

        let item = Command {
            actions: vec![
                Action::RequestTag(RequestTag { eop: true, id: 25 }),
                Action::Status(
                    Status::Interface(
                        InterfaceStatus::Dash7(Dash7InterfaceStatus {
                            channel: Channel {
                                header: ChannelHeader::new(
                                    ChannelBand::Band433,
                                    ChannelClass::LoRate,
                                    ChannelCoding::Pn9,
                                ),
                                index: 16,
                            },
                            rx_level: 70,
                            link_budget: 80,
                            target_rx_level: 80,
                            nls: false,
                            missed: false,
                            retry: false,
                            unicast: false,
                            fifo_token: 200,
                            sequence_number: 0,
                            response_timeout: 20u32.into(),
                            addressee: Addressee::default(),
                        })
                        .into(),
                    )
                    .into(),
                ),
                Action::ReturnFileData(FileData::new(
                    ActionHeader::default(),
                    FileOffset::no_offset(0x51),
                    File::Other("Hello world".into()),
                )),
            ],
        };

        test_item(item, &data);
    }

    #[test]
    fn test_command_with_interface_status() {
        // 44
        let data = &hex!(
            r#"
        62 D7 14 32 00 32 2D 3E 50 80 00 00 58 20 01 39 38 38 37 00 39 00 2E 32
        01 44 35 00 2C 00 F4 01 00 00 44 48 00 09 00 00 00 00 00 00 30 00 00 44
        48 00 09 00 00 30 00 00 00 00 02 00 44 48 00 09 00 00 70 00 00 00 30 02 00"#
        );

        let item = Command {
            actions: vec![
                Action::Status(
                    Status::Interface(
                        InterfaceStatus::Dash7(Dash7InterfaceStatus {
                            channel: Channel {
                                header: ChannelHeader::new(
                                    ChannelBand::Band868,
                                    ChannelClass::LoRate,
                                    ChannelCoding::FecPn9,
                                ),
                                index: 50,
                            },
                            rx_level: 45,
                            link_budget: 62,
                            target_rx_level: 80,
                            nls: true,
                            missed: false,
                            retry: false,
                            unicast: false,
                            fifo_token: 0,
                            sequence_number: 0,
                            response_timeout: 384.into(),
                            addressee: Addressee::new(
                                #[cfg(feature = "_wizzilab")]
                                false,
                                #[cfg(feature = "_wizzilab")]
                                GroupCondition::Any,
                                Address::Uid(4123107267735781422u64),
                                NlsState::None,
                                AccessClass::new(0, 1),
                            ),
                        })
                        .into(),
                    )
                    .into(),
                ),
                Action::Forward(Forward::new(false, InterfaceConfiguration::Serial)),
                Action::WriteFileData(FileData::new(
                    ActionHeader {
                        group: false,
                        response: true,
                    },
                    FileOffset {
                        file_id: 53,
                        offset: 0u32.into(),
                    },
                    File::Other(
                        hex!(
                            r#"
                       00 F4 01 00 00 44 48 00 09 00 00 00 00 00 00 30 00 00 44
                       48 00 09 00 00 30 00 00 00 00 02 00 44 48 00 09 00 00 70 00 00 00 30 02 00"#
                        )
                        .to_vec(),
                    ),
                )),
            ],
        };

        test_item(item, data);
    }
}
