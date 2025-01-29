use std::borrow::Cow;
#[cfg(feature = "std")]
use std::fmt::{self, Display};

#[cfg(not(feature = "std"))]
use alloc::fmt;

use deku::{
    no_std_io::{self, Cursor, Seek},
    prelude::*,
};

use crate::{
    file::{FileCtx, OtherFile},
    session::InterfaceStatus,
    utils::{from_bytes, from_reader},
};

use super::{
    operand::{RequestTag, ResponseTag, ResponseTagHeader, Status}, operation::{Operation}
};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Command<F = OtherFile>
where
    F: for<'a> DekuReader<'a, FileCtx> + DekuWriter<FileCtx>,
{
    // we cannot process an indirect forward without knowing the interface type, which is stored in the interface file
    // as identified by the indirectforward itself
    // As such, we HAVE to bail here
    // Hopefully this will be addressed in SPEC 1.3
    // Always stop reading when length is reached
    // #[deku(bytes_read = "length", until = "|action: &Action| { action.deku_id().unwrap() == OpCode::INDIRECT_FORWARD }")]
    pub actions: Vec<Operation<F>>,
}

impl<'a, F> DekuReader<'a, u32> for Command<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    fn from_reader_with_ctx<R>(reader: &mut Reader<R>, length: u32) -> Result<Self, DekuError>
    where
        R: no_std_io::Read + no_std_io::Seek,
        Self: Sized,
    {
        let mut command = Self { actions: vec![] };

        let valid_length = |r: &mut Reader<R>, l: u32| -> Result<bool, DekuError> {
            Ok(match l {
                0 => true,
                _ => {
                    r.stream_position()
                        .map_err(|err| DekuError::Io(err.kind()))?
                        < (l as u64)
                }
            })
        };

        while valid_length(reader, length)? && !reader.end() {
            if let Some(Operation::IndirectForward(_)) = command.actions.last() {
                return Ok(command);
            }
            let action = Operation::from_reader_with_ctx(reader, ())?;
            command.actions.push(action);
        }
        return Ok(command);
    }
}

impl<F> Command<F>
where
    F: for<'a> DekuReader<'a, FileCtx> + DekuWriter<FileCtx> + fmt::Debug,
{
    pub fn from_reader<'a, R>(input: (&'a mut R, usize)) -> Result<(usize, Self), DekuError>
    where
        R: no_std_io::Read + no_std_io::Seek,
    {
        from_reader(input, 0)
    }

    pub fn from_bytes(input: (&'_ [u8], usize)) -> Result<((&'_ [u8], usize), Self), DekuError> {
        from_bytes(input, 0)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, DekuError> {
        let mut out_buf = Vec::new();
        let mut cursor = Cursor::new(&mut out_buf);
        let mut writer = Writer::new(&mut cursor);
        DekuWriter::to_writer(self, &mut writer, 0)?;
        writer.finalize()?;
        Ok(out_buf)
    }
}

/// Stub implementation so we can implement DekuContainerWrite
impl<F> DekuWriter<u32> for Command<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    fn to_writer<W>(&self, writer: &mut Writer<W>, _: u32) -> Result<(), DekuError>
    where
        W: no_std_io::Write + no_std_io::Seek,
    {
        for action in self.actions.iter() {
            action.to_writer(writer, ())?;
        }
        Ok(())
    }
}

impl<F> Command<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    pub fn new(actions: Vec<Operation<F>>) -> Self {
        // TODO: validate actions
        Self { actions }
    }

    pub fn interface_status(&self) -> Option<&InterfaceStatus> {
        for action in self.actions.iter() {
            if let Operation::Status(status) = &action {
                if let Status::Interface(iface_status) = &status.status {
                    return Some(&iface_status.status);
                }
            }
        }
        None
    }

    // TODO: a generator would be great here
    pub fn actions_without_interface_status(&self) -> Vec<&Operation<F>> {
        let mut actions = vec![];
        for action in self.actions.iter() {
            if let Operation::Status(status) = &action {
                if let Status::Interface(_) = &status.status {
                    continue;
                }
            }
            actions.push(action);
        }
        actions
    }

    pub fn request_tag(&self) -> Option<&RequestTag> {
        for action in self.actions.iter() {
            if let Operation::RequestTag(operation) = action {
                return Some(operation);
            }
        }
        None
    }

    pub fn request_id(&self) -> Option<u8> {
        self.request_tag().map(|t| t.id)
    }

    pub fn response_tag(&self) -> Option<&ResponseTag> {
        for action in self.actions.iter() {
            if let Operation::ResponseTag(operation) = action {
                return Some(operation);
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
            if let Operation::ResponseTag(ResponseTag {
                header: ResponseTagHeader { end_of_packet, .. },
                ..
            }) = action
            {
                return *end_of_packet;
            }
        }
        false
    }
}

impl<F> TryFrom<&'_ [u8]> for Command<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx> + fmt::Debug,
{
    type Error = DekuError;
    fn try_from(input: &'_ [u8]) -> Result<Self, Self::Error> {
        let (rest, res) = Self::from_bytes((input, 0))?;
        if !rest.0.is_empty() {
            return Err(DekuError::Parse({
                let res = fmt::format(format_args!("Too much data"));
                Cow::Owned(res)
            }));
        }
        Ok(res)
    }
}

impl<F> TryFrom<Command<F>> for Vec<u8>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx> + fmt::Debug,
{
    type Error = DekuError;
    fn try_from(input: Command<F>) -> Result<Self, Self::Error> {
        input.to_bytes()
    }
}

#[cfg(feature = "std")]
impl<F> Display for Command<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag_str = self
            .tag_id()
            .map_or("".to_string(), |t| format!("with tag {} ", t));
        f.write_str(&format!("Command::<OtherFile> {}", &tag_str))?;

        let status = if let Some(operation) = self.response_tag() {
            if operation.header.end_of_packet {
                if operation.header.error {
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
    use std::sync::Arc;

    use hex_literal::hex;

    use super::*;

    #[cfg(feature = "_wizzilab")]
    use crate::transport::GroupCondition;
    use crate::{
        app::{
            operation::OpCode,
            interface::InterfaceConfiguration,
            operand::{
                ActionHeader, FileDataOperand, Forward, Nop, ReadFileData, RequestTagHeader,
                ResponseTagHeader, Status,
            },
        },
        file::{FileData, SystemFile, File},
        link::AccessClass,
        network::{Address, Addressee, NlsState},
        physical::{Channel, ChannelBand, ChannelClass, ChannelCoding, ChannelHeader},
        session::{Dash7InterfaceStatus, InterfaceStatus},
        test_tools::test_item,
    };

    #[test]
    fn test_command() {
        let cmd = Command::<OtherFile> {
            actions: vec![
                Operation::RequestTag(RequestTag {
                    id: 66,
                    header: RequestTagHeader {
                        end_of_packet: true,
                    },
                    opcode: OpCode::RequestTag,
                }),
                Operation::ReadFileData(ReadFileData {
                    header: ActionHeader {
                        response: true,
                        group: false,
                    },
                    file_id: 0,
                    offset: 0u32.into(),

                    length: 8u32.into(),
                    opcode: OpCode::ReadFileData,
                }),
                Operation::ReadFileData(ReadFileData {
                    header: ActionHeader {
                        response: false,
                        group: true,
                    },
                    file_id: 4,
                    offset: 2u32.into(),
                    length: 3u32.into(),
                    opcode: OpCode::ReadFileData,
                }),
                Operation::Nop(Nop {
                    header: ActionHeader {
                        response: true,
                        group: true,
                    },
                    opcode: OpCode::Nop,
                }),
            ],
        };
        let data = &hex!("B4 42 41 00 00 08 81 04 02 03 C0");

        test_item(cmd, data);
    }

    #[test]
    fn test_command_request_id() {
        assert_eq!(
            Command::<OtherFile> {
                actions: vec![
                    Operation::RequestTag(RequestTag {
                        header: RequestTagHeader {
                            end_of_packet: true
                        },
                        id: 66,
                        opcode: OpCode::RequestTag
                    }),
                    Operation::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: true
                        },
                        opcode: OpCode::Nop
                    })
                ]
            }
            .request_id(),
            Some(66)
        );
        assert_eq!(
            Command::<OtherFile> {
                actions: vec![
                    Operation::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        },
                        opcode: OpCode::Nop
                    }),
                    Operation::RequestTag(RequestTag {
                        header: RequestTagHeader {
                            end_of_packet: true
                        },
                        id: 44,
                        opcode: OpCode::RequestTag
                    }),
                ]
            }
            .request_id(),
            Some(44)
        );
        assert_eq!(
            Command::<OtherFile> {
                actions: vec![
                    Operation::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        },
                        opcode: OpCode::Nop
                    }),
                    Operation::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        },
                        opcode: OpCode::Nop
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
            Command::<OtherFile> {
                actions: vec![
                    Operation::ResponseTag(ResponseTag {
                        header: ResponseTagHeader {
                            end_of_packet: true,
                            error: true,
                        },
                        id: 66,
                        opcode: OpCode::ResponseTag
                    }),
                    Operation::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: true
                        },
                        opcode: OpCode::Nop
                    })
                ]
            }
            .response_id(),
            Some(66)
        );
        assert_eq!(
            Command::<OtherFile> {
                actions: vec![
                    Operation::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        },
                        opcode: OpCode::Nop
                    }),
                    Operation::ResponseTag(ResponseTag {
                        header: ResponseTagHeader {
                            end_of_packet: true,
                            error: true,
                        },
                        id: 44,
                        opcode: OpCode::ResponseTag
                    }),
                ]
            }
            .response_id(),
            Some(44)
        );
        assert_eq!(
            Command::<OtherFile> {
                actions: vec![
                    Operation::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        },
                        opcode: OpCode::Nop
                    }),
                    Operation::Nop(Nop {
                        header: ActionHeader {
                            group: true,
                            response: false
                        },
                        opcode: OpCode::Nop
                    })
                ]
            }
            .response_id(),
            None
        );
    }

    #[test]
    fn test_command_is_last_response() {
        assert!(Command::<SystemFile> {
            actions: vec![
                Operation::ResponseTag(ResponseTag {
                    header: ResponseTagHeader {
                        end_of_packet: true,
                        error: true,
                    },
                    id: 66,
                    opcode: OpCode::ResponseTag
                }),
                Operation::Nop(Nop {
                    header: ActionHeader {
                        group: true,
                        response: true
                    },
                    opcode: OpCode::Nop
                })
            ]
        }
        .is_last_response());

        assert!(!Command::<OtherFile> {
            actions: vec![
                Operation::ResponseTag(ResponseTag {
                    header: ResponseTagHeader {
                        end_of_packet: false,
                        error: false,
                    },
                    id: 66,
                    opcode: OpCode::ResponseTag
                }),
                Operation::Nop(Nop {
                    header: ActionHeader {
                        group: true,
                        response: true
                    },
                    opcode: OpCode::Nop
                })
            ]
        }
        .is_last_response());

        assert!(!Command::<OtherFile> {
            actions: vec![
                Operation::ResponseTag(ResponseTag {
                    header: ResponseTagHeader {
                        end_of_packet: false,
                        error: true,
                    },
                    id: 44,
                    opcode: OpCode::ResponseTag
                }),
                Operation::ResponseTag(ResponseTag {
                    header: ResponseTagHeader {
                        end_of_packet: true,
                        error: true
                    },
                    id: 44,
                    opcode: OpCode::ResponseTag
                }),
            ]
        }
        .is_last_response());

        assert!(!Command::<OtherFile> {
            actions: vec![
                Operation::Nop(Nop {
                    header: ActionHeader {
                        group: true,
                        response: false
                    },
                    opcode: OpCode::Nop
                }),
                Operation::Nop(Nop {
                    header: ActionHeader {
                        group: true,
                        response: false
                    },
                    opcode: OpCode::Nop
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

        let item = Command::<OtherFile> {
            actions: vec![
                Operation::Status(
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
                Operation::ReturnFileData(FileData::new(
                    ActionHeader::default(),
                    FileData {
                        id: 0x51,
                        offset: 0u32.into(),
                        data: "Hello world".into(),
                    },
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

        let item = Command::<OtherFile> {
            actions: vec![
                Operation::RequestTag(RequestTag {
                    header: RequestTagHeader {
                        end_of_packet: true,
                    },
                    id: 25,
                }),
                Operation::Status(
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
                Operation::ReturnFileData(FileData::new(
                    ActionHeader::default(),
                    FileOffset::no_offset(0x51),
                    FileData::{
                        id: 0x51,
                        offset: 0u32.into(),
                        "Hello world".into(),
                    }
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

        let item = Command::<OtherFile> {
            actions: vec![
                Operation::Status(
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
                                Address::UId(4123107267735781422u64),
                                NlsState::None,
                                AccessClass::new(0, 1),
                            ),
                        })
                        .into(),
                    )
                    .into(),
                ),
                Operation::Forward(Forward::new(false, InterfaceConfiguration::Serial)),
                Operation::WriteFileData(FileDataOperand::new(
                    ActionHeader {
                        group: false,
                        response: true,
                    },
                    FileData {
                        id: 53,
                        offset: 0u32.into(),
                        file: File::User(OtherFile::new(
                            hex!(
                                r#"
                       00 F4 01 00 00 44 48 00 09 00 00 00 00 00 00 30 00 00 44
                       48 00 09 00 00 30 00 00 00 00 02 00 44 48 00 09 00 00 70 00 00 00 30 02 00"#
                            )
                            .to_vec(),
                        )),
                    },
                    OpCode::WriteFileData,
                )),
            ],
        };

        test_item(item, data);
    }
}
