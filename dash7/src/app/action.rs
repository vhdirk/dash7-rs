use std::borrow::Cow;
#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use alloc::fmt;

use deku::{
    no_std_io::{self, Seek, Write},
    prelude::*,
};

use crate::utils::{from_bytes, from_reader};

use super::operation::{
    ActionQuery, Chunk, CopyFile, Extension, FileData, FileId, FileProperties, Forward,
    IndirectForward, Logic, Nop, PermissionRequest, ReadFileData, RequestTag, ResponseTag,
    StatusOperand,
};

#[cfg(feature = "_wizzilab")]
use super::interface_final::*;

// ===============================================================================
// OpCodes
// ===============================================================================

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
#[deku(bits = 6, id_type = "u8")]
pub enum OpCode {
    // Nop
    #[default]
    #[deku(id = "0")]
    Nop,
    // Read
    #[deku(id = "1")]
    ReadFileData,
    #[deku(id = "2")]
    ReadFileProperties,

    // Write
    #[deku(id = "4")]
    WriteFileData,
    #[deku(id = "5")]
    WriteFileDataFlush,
    #[deku(id = "6")]
    WriteFileProperties,
    #[deku(id = "8")]
    ActionQuery,
    #[deku(id = "9")]
    BreakQuery,
    #[deku(id = "10")]
    PermissionRequest,
    #[deku(id = "11")]
    VerifyChecksum,

    // Management
    #[deku(id = "16")]
    ExistFile,
    #[deku(id = "17")]
    CreateNewFile,
    #[deku(id = "18")]
    DeleteFile,
    #[deku(id = "19")]
    RestoreFile,
    #[deku(id = "20")]
    FlushFile,
    #[deku(id = "23")]
    CopyFile,
    #[deku(id = "31")]
    ExecuteFile,

    // Response
    #[deku(id = "32")]
    ReturnFileData,
    #[deku(id = "33")]
    ReturnFileProperties,
    #[deku(id = "34")]
    Status,
    #[deku(id = "35")]
    ResponseTag,

    #[cfg(feature = "_wizzilab")]
    #[deku(id = "38")]
    TxStatus,

    // Special
    #[deku(id = "48")]
    Chunk,
    #[deku(id = "49")]
    Logic,
    #[deku(id = "50")]
    Forward,
    #[deku(id = "51")]
    IndirectForward,
    #[deku(id = "52")]
    RequestTag,
    #[deku(id = "63")]
    Extension,

    #[deku(id_pat = "_")]
    Other(u8)
}


impl OpCode {
    pub fn write<W: Write + Seek>(writer: &mut Writer<W>, opcode: OpCode) -> Result<(), DekuError> {
        opcode.to_writer(writer, ())
    }
}

// ===============================================================================
// Actions
// ===============================================================================

#[derive(Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
pub enum Action {
    /// Nop
    Nop(Nop),
    /// Read
    ReadFileData(ReadFileData),

    ReadFileProperties(FileId),

    // Write
    WriteFileData(FileData),
    WriteFileDataFlush(FileData),
    WriteFileProperties(FileProperties),

    /// Add a condition on the execution of the next group of action.
    ///
    /// If the condition is not met, the next group of actions should be skipped.
    ActionQuery(ActionQuery),

    /// Add a condition to continue the processing of this ALP command.
    ///
    /// If the condition is not met the all the next ALP action of this command should be ignored.
    BreakQuery(ActionQuery),
    PermissionRequest(PermissionRequest),

    /// Calculate checksum of file and compare with checksum in query
    // ALP_SPEC: Is the checksum calculated on the targeted data (offset, size) or the whole file?
    VerifyChecksum(ActionQuery),

    // Management
    ExistFile(FileId),
    CreateNewFile(FileProperties),
    DeleteFile(FileId),
    RestoreFile(FileId),
    FlushFile(FileId),
    CopyFile(CopyFile),
    ExecuteFile(FileId),

    // Response
    ReturnFileData(FileData),
    ReturnFileProperties(FileProperties),
    Status(StatusOperand),
    ResponseTag(ResponseTag),

    #[cfg(feature = "_wizzilab")]
    TxStatus(TxStatusOperation),

    // Special
    Chunk(Chunk),
    Logic(Logic),
    Forward(Forward),
    IndirectForward(IndirectForward),
    RequestTag(RequestTag),
    Extension(Extension),
}

// macro_rules! read_action {
//     ($action: ident, $operation: ty, $reader: ident) => {{
//         let header = <$operation as Operation>::Header::from_reader_with_ctx($reader, ())?;

//         // now skip the opcode
//         $reader.skip_bits(6)?;

//         <$operation as DekuReader<'_, _>>::from_reader_with_ctx($reader, header)
//             .map(|action| Self::$action(action))?
//     }};
// }

macro_rules! read_action {
    ($action: ident, $operation: ty, $reader: ident, $opcode: ident) => {{
        <$operation as DekuReader<'_, _>>::from_reader_with_ctx($reader, $opcode)
            .map(|action| Self::$action(action))?
    }};
}

impl<'a> DekuReader<'a, ()> for Action {
    fn from_reader_with_ctx<R>(reader: &mut Reader<R>, _: ()) -> Result<Self, DekuError>
    where
        R: no_std_io::Read + no_std_io::Seek,
    {
        // skip the 2 preamble bits
        reader.skip_bits(2)?;

        // read the opcode
        let code = <OpCode as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?;

        // seek back to the beginning (1 byte)
        let _ = reader.seek_relative(-1); // TODO process err

        // // Read the preamble and pass it on as context values
        // let preamble = (<u8 as DekuReader<'_, _>>::from_reader_with_ctx(
        //     reader,
        //     (Endian::Big, BitSize(2)),
        // )?,)
        //     .0;

        // let mut cursor = no_std_io::Cursor::new([preamble]);

        // let mut preamble_reader = Reader::new(&mut cursor);
        // preamble_reader.skip_bits(6)?;

        // now we have to revert for those 2 bits?
        let value = match code {
            OpCode::Nop => read_action!(Nop, Nop, reader, code),
            OpCode::ReadFileData => {
                read_action!(ReadFileData, ReadFileData, reader, code)
            }
            OpCode::ReadFileProperties => {
                read_action!(ReadFileProperties, FileId, reader, code)
            }
            OpCode::WriteFileData => read_action!(WriteFileData, FileData, reader, code),
            OpCode::WriteFileDataFlush => {
                read_action!(WriteFileDataFlush, FileData, reader, code)
            }
            OpCode::WriteFileProperties => {
                read_action!(WriteFileProperties, FileProperties, reader, code)
            }
            OpCode::ActionQuery => read_action!(ActionQuery, ActionQuery, reader, code),
            OpCode::BreakQuery => read_action!(BreakQuery, ActionQuery, reader, code),
            OpCode::PermissionRequest => {
                read_action!(PermissionRequest, PermissionRequest, reader, code)
            }
            OpCode::VerifyChecksum => {
                read_action!(VerifyChecksum, ActionQuery, reader, code)
            }
            OpCode::ExistFile => read_action!(ExistFile, FileId, reader, code),
            OpCode::CreateNewFile => {
                read_action!(CreateNewFile, FileProperties, reader, code)
            }
            OpCode::DeleteFile => read_action!(DeleteFile, FileId, reader, code),
            OpCode::RestoreFile => read_action!(RestoreFile, FileId, reader, code),
            OpCode::FlushFile => read_action!(FlushFile, FileId, reader, code),
            OpCode::CopyFile => read_action!(CopyFile, CopyFile, reader, code),
            OpCode::ExecuteFile => read_action!(ExecuteFile, FileId, reader, code),
            OpCode::ReturnFileData => {
                read_action!(ReturnFileData, FileData, reader, code)
            }
            OpCode::ReturnFileProperties => {
                read_action!(ReturnFileProperties, FileProperties, reader, code)
            }
            OpCode::ResponseTag => read_action!(ResponseTag, ResponseTag, reader, code),

            #[cfg(feature = "_wizzilab")]
            OpCode::TxStatus => read_action!(TxStatus, TxStatusOperation, reader, code),
            OpCode::Chunk => read_action!(Chunk, Chunk, reader, code),
            OpCode::Logic => read_action!(Logic, Logic, reader, code),
            OpCode::RequestTag => read_action!(RequestTag, RequestTag, reader, code),
            OpCode::Status => read_action!(Status, StatusOperand, reader, code),
            OpCode::Forward => read_action!(Forward, Forward, reader, code),
            OpCode::IndirectForward => {
                read_action!(IndirectForward, IndirectForward, reader, code)
            }
            OpCode::Extension => read_action!(Extension, Extension, reader, code),

            // TODO: Other!
            _ => return Err(DekuError::InvalidParam("opcode".into())),
        };
        Ok(value)
    }
}

impl TryFrom<&'_ [u8]> for Action {
    type Error = DekuError;
    fn try_from(input: &'_ [u8]) -> Result<Self, Self::Error> {
        let (rest, res) = <Self as DekuContainerRead>::from_bytes((input, 0))?;
        if !rest.0.is_empty() {
            return Err(DekuError::Parse({
                let res = fmt::format(format_args!("Too much data"));
                Cow::Owned(res)
            }));
        }
        Ok(res)
    }
}

impl DekuContainerRead<'_> for Action {
    fn from_reader<'a, R>(input: (&'a mut R, usize)) -> Result<(usize, Self), DekuError>
    where
        R: no_std_io::Read + no_std_io::Seek,
    {
        from_reader(input, ())
    }

    fn from_bytes(input: (&'_ [u8], usize)) -> Result<((&'_ [u8], usize), Self), DekuError> {
        from_bytes(input, ())
    }
}

impl DekuEnumExt<'_, OpCode> for Action {
    fn deku_id(&self) -> Result<OpCode, DekuError> {
        match self {
            Action::Nop(_) => Ok(OpCode::Nop),
            Action::ReadFileData(_) => Ok(OpCode::ReadFileData),
            Action::ReadFileProperties(_) => Ok(OpCode::ReadFileProperties),
            Action::WriteFileData(_) => Ok(OpCode::WriteFileData),
            Action::WriteFileDataFlush(_) => Ok(OpCode::WriteFileDataFlush),
            Action::WriteFileProperties(_) => Ok(OpCode::WriteFileProperties),
            Action::ActionQuery(_) => Ok(OpCode::ActionQuery),
            Action::BreakQuery(_) => Ok(OpCode::BreakQuery),
            Action::PermissionRequest(_) => Ok(OpCode::PermissionRequest),
            Action::VerifyChecksum(_) => Ok(OpCode::VerifyChecksum),
            Action::ExistFile(_) => Ok(OpCode::ExistFile),
            Action::CreateNewFile(_) => Ok(OpCode::CreateNewFile),
            Action::DeleteFile(_) => Ok(OpCode::DeleteFile),
            Action::RestoreFile(_) => Ok(OpCode::RestoreFile),
            Action::FlushFile(_) => Ok(OpCode::FlushFile),
            Action::CopyFile(_) => Ok(OpCode::CopyFile),
            Action::ExecuteFile(_) => Ok(OpCode::ExecuteFile),
            Action::ReturnFileData(_) => Ok(OpCode::ReturnFileData),
            Action::ReturnFileProperties(_) => Ok(OpCode::ReturnFileProperties),
            Action::ResponseTag(_) => Ok(OpCode::ResponseTag),
            #[cfg(feature = "_wizzilab")]
            Action::TxStatus(_) => Ok(OpCode::TxStatus),
            Action::Chunk(_) => Ok(OpCode::Chunk),
            Action::Logic(_) => Ok(OpCode::Logic),
            Action::Status(_) => Ok(OpCode::Status),
            Action::Forward(_) => Ok(OpCode::Forward),
            Action::IndirectForward(_) => Ok(OpCode::IndirectForward),
            Action::RequestTag(_) => Ok(OpCode::RequestTag),
            Action::Extension(_) => Ok(OpCode::Extension),
        }
    }
}

impl DekuUpdate for Action {
    fn update(&mut self) -> Result<(), DekuError> {
        Ok(())
    }
}

impl DekuWriter<()> for Action {
    fn to_writer<W>(&self, writer: &mut Writer<W>, _: ()) -> Result<(), DekuError>
    where
        W: no_std_io::Write + no_std_io::Seek,
    {
        let opcode = self.deku_id()?;

        match self {
            Action::Nop(action) => action.to_writer(writer, opcode)?,
            Action::ReadFileData(action) => action.to_writer(writer, opcode)?,
            Action::ReadFileProperties(action) => action.to_writer(writer, opcode)?,
            Action::WriteFileData(action) => action.to_writer(writer, opcode)?,
            Action::WriteFileDataFlush(action) => action.to_writer(writer, opcode)?,
            Action::WriteFileProperties(action) => action.to_writer(writer, opcode)?,
            Action::ActionQuery(action) => action.to_writer(writer, opcode)?,
            Action::BreakQuery(action) => action.to_writer(writer, opcode)?,
            Action::PermissionRequest(action) => action.to_writer(writer, opcode)?,
            Action::VerifyChecksum(action) => action.to_writer(writer, opcode)?,
            Action::ExistFile(action) => action.to_writer(writer, opcode)?,
            Action::CreateNewFile(action) => action.to_writer(writer, opcode)?,
            Action::DeleteFile(action) => action.to_writer(writer, opcode)?,
            Action::RestoreFile(action) => action.to_writer(writer, opcode)?,
            Action::FlushFile(action) => action.to_writer(writer, opcode)?,
            Action::CopyFile(action) => action.to_writer(writer, opcode)?,
            Action::ExecuteFile(action) => action.to_writer(writer, opcode)?,
            Action::ReturnFileData(action) => action.to_writer(writer, opcode)?,
            Action::ReturnFileProperties(action) => action.to_writer(writer, opcode)?,
            Action::ResponseTag(action) => action.to_writer(writer, opcode)?,
            #[cfg(feature = "_wizzilab")]
            Action::TxStatus(action) => action.to_writer(writer, opcode)?,
            Action::Chunk(action) => action.to_writer(writer, opcode)?,
            Action::Logic(action) => action.to_writer(writer, opcode)?,
            Action::Status(action) => action.to_writer(writer, opcode)?,
            Action::Forward(action) => action.to_writer(writer, opcode)?,
            Action::IndirectForward(action) => action.to_writer(writer, opcode)?,
            Action::RequestTag(action) => action.to_writer(writer, opcode)?,
            Action::Extension(action) => action.to_writer(writer, opcode)?,
        }

        Ok(())
    }
}

impl TryFrom<Action> for Vec<u8> {
    type Error = DekuError;
    fn try_from(input: Action) -> Result<Self, Self::Error> {
        DekuContainerWrite::to_bytes(&input)
    }
}
impl DekuContainerWrite for Action {}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;
    #[cfg(feature = "_wizzilab")]
    use crate::transport::GroupCondition;
    use crate::{
        app::{
            interface::{IndirectInterface, InterfaceConfiguration},
            operation::{
                ActionHeader, ActionStatus, ChunkStep, FileOffset, LogicOp, Permission,
                PermissionLevel, RequestTagHeader, ResponseTagHeader, Status, StatusCode,
            },
            query::{NonVoid, Query},
        },
        data::{self, FileHeader, FilePermissions, UserPermissions},
        file::File,
        link::AccessClass,
        network::{Address, Addressee, NlsState},
        physical::{Channel, ChannelBand, ChannelClass, ChannelCoding, ChannelHeader},
        session::{Dash7InterfaceStatus, InterfaceStatus},
        test_tools::{test_item, WithPadding},
    };

    #[test]
    fn test_header() {
        test_item(
            WithPadding::<ActionHeader, 0, 6>(ActionHeader {
                group: true,
                response: false,
            }),
            &[0b1000_0000],
        )
    }

    #[test]
    fn test_nop() {
        test_item(
            Action::Nop(Nop {
                header: ActionHeader {
                    group: false,
                    response: true,
                },
                opcode: OpCode::Nop,
            }),
            &[0b0100_0000],
        )
    }

    #[test]
    fn test_read_file_data() {
        test_item(
            Action::ReadFileData(ReadFileData {
                header: ActionHeader {
                    group: false,
                    response: true,
                },
                offset: FileOffset {
                    file_id: 1,
                    offset: 2u32.into(),
                },
                length: 3u32.into(),
                opcode: OpCode::ReadFileData,
            }),
            &hex!("41 01 02 03"),
        )
    }

    #[test]
    fn test_read_file_properties() {
        test_item(
            Action::ReadFileProperties(FileId {
                header: ActionHeader {
                    group: false,
                    response: false,
                },
                file_id: 9,
                opcode: OpCode::ReadFileProperties,
            }),
            &hex!("02 09"),
        )
    }

    #[test]
    fn test_write_file_data() {
        let data = hex!("01 02 03").to_vec();
        test_item(
            Action::WriteFileData(FileData::new(
                ActionHeader {
                    group: true,
                    response: false,
                },
                FileOffset {
                    file_id: 0xF9,
                    offset: 5u32.into(),
                },
                File::Other{id: 0xF9, buffer: data},
                OpCode::WriteFileData,
            )),
            &hex!("84 F9 05 03 010203"),
        )
    }

    #[test]
    fn test_return_file_properties() {
        test_item(
            Action::ReturnFileProperties(FileProperties {
                header: ActionHeader {
                    group: false,
                    response: false,
                },
                file_id: 9,
                file_header: FileHeader {
                    permissions: FilePermissions {
                        encrypted: true,
                        executable: false,
                        user: UserPermissions {
                            read: true,
                            write: true,
                            executable: true,
                        },
                        guest: UserPermissions {
                            read: false,
                            write: false,
                            executable: false,
                        },
                    },
                    properties: data::FileProperties {
                        enabled: false,
                        condition: data::ActionCondition::Read,
                        storage_class: data::StorageClass::Permanent,
                    },
                    alp_command_file_id: 1,
                    interface_file_id: 2,
                    file_size: 0xDEAD_BEEF,
                    allocated_size: 0xBAAD_FACE,
                },
                opcode: OpCode::ReturnFileProperties,
            }),
            &hex!("21 09  B8 13 01 02 DEADBEEF BAADFACE"),
        )
    }

    #[test]
    fn test_write_file_properties() {
        test_item(
            Action::WriteFileProperties(FileProperties {
                header: ActionHeader {
                    group: true,
                    response: false,
                },
                file_id: 9,
                file_header: FileHeader {
                    permissions: FilePermissions {
                        encrypted: true,
                        executable: false,
                        user: UserPermissions {
                            read: true,
                            write: true,
                            executable: true,
                        },
                        guest: UserPermissions {
                            read: false,
                            write: false,
                            executable: false,
                        },
                    },
                    properties: data::FileProperties {
                        enabled: false,
                        condition: data::ActionCondition::Read,
                        storage_class: data::StorageClass::Permanent,
                    },
                    alp_command_file_id: 1,
                    interface_file_id: 2,
                    file_size: 0xDEAD_BEEF,
                    allocated_size: 0xBAAD_FACE,
                },
                opcode: OpCode::WriteFileProperties,
            }),
            &hex!("86 09 B8 13 01 02 DEADBEEF BAADFACE"),
        )
    }

    #[test]
    fn test_permission_request() {
        test_item(
            Action::PermissionRequest(PermissionRequest {
                header: ActionHeader {
                    group: false,
                    response: false,
                },
                level: PermissionLevel::Root,
                permission: Permission::Dash7(hex!("0102030405060708")),
                opcode: OpCode::PermissionRequest,
            }),
            &hex!("0A 01 42 0102030405060708"),
        )
    }

    #[test]
    fn test_exist_file() {
        test_item(
            Action::ExistFile(FileId {
                header: ActionHeader {
                    group: false,
                    response: false,
                },
                file_id: 9,
                opcode: OpCode::ExistFile,
            }),
            &hex!("10 09"),
        )
    }

    #[test]
    fn test_create_new_file() {
        test_item(
            Action::CreateNewFile(FileProperties {
                header: ActionHeader {
                    group: true,
                    response: false,
                },
                file_id: 3,
                file_header: FileHeader {
                    permissions: FilePermissions {
                        encrypted: true,
                        executable: false,
                        user: UserPermissions {
                            read: true,
                            write: true,
                            executable: true,
                        },
                        guest: UserPermissions {
                            read: false,
                            write: false,
                            executable: false,
                        },
                    },
                    properties: data::FileProperties {
                        enabled: false,
                        condition: data::ActionCondition::Read,
                        storage_class: data::StorageClass::Permanent,
                    },
                    alp_command_file_id: 1,
                    interface_file_id: 2,
                    file_size: 0xDEAD_BEEF,
                    allocated_size: 0xBAAD_FACE,
                },
                opcode: OpCode::CreateNewFile,
            }),
            &hex!("91 03 B8 13 01 02 DEADBEEF BAADFACE"),
        )
    }

    #[test]
    fn test_delete_file() {
        test_item(
            Action::DeleteFile(FileId {
                header: ActionHeader {
                    group: false,
                    response: true,
                },
                file_id: 9,
                opcode: OpCode::DeleteFile,
            }),
            &hex!("52 09"),
        )
    }

    #[test]
    fn test_restore_file() {
        test_item(
            Action::RestoreFile(FileId {
                header: ActionHeader {
                    group: true,
                    response: true,
                },
                file_id: 9,
                opcode: OpCode::RestoreFile,
            }),
            &hex!("D3 09"),
        )
    }

    #[test]
    fn test_flush_file() {
        test_item(
            Action::FlushFile(FileId {
                header: ActionHeader {
                    group: false,
                    response: false,
                },
                file_id: 9,
                opcode: OpCode::FlushFile,
            }),
            &hex!("14 09"),
        )
    }

    #[test]
    fn test_copy_file() {
        test_item(
            Action::CopyFile(CopyFile {
                header: ActionHeader {
                    group: false,
                    response: false,
                },
                src_file_id: 0x42,
                dst_file_id: 0x24,
                opcode: OpCode::CopyFile,
            }),
            &hex!("17 42 24"),
        )
    }

    #[test]
    fn test_execute_file() {
        test_item(
            Action::ExecuteFile(FileId {
                header: ActionHeader {
                    group: false,
                    response: false,
                },
                file_id: 9,
                opcode: OpCode::ExecuteFile,
            }),
            &hex!("1F 09"),
        )
    }

    #[test]
    fn test_return_file_data() {
        let data = hex!("01 02 03").to_vec();

        test_item(
            Action::ReturnFileData(FileData::new(
                ActionHeader {
                    group: false,
                    response: false,
                },
                FileOffset {
                    file_id: 0xF9,
                    offset: 5u32.into(),
                },
                File::Other{id: 0xF9, buffer: data},
                OpCode::ReturnFileData,
            )),
            &hex!("20 F9 05 03 010203"),
        )
    }

    #[test]
    fn test_action_query() {
        test_item(
            Action::ActionQuery(ActionQuery {
                header: ActionHeader {
                    group: true,
                    response: true,
                },
                query: Query::NonVoid(NonVoid {
                    length: 4u32.into(),
                    file: FileOffset {
                        file_id: 5,
                        offset: 6u32.into(),
                    },
                }),
                opcode: OpCode::ActionQuery,
            }),
            &hex!("C8 00 04 05 06"),
        )
    }

    #[test]
    fn test_break_query() {
        test_item(
            Action::BreakQuery(ActionQuery {
                header: ActionHeader {
                    group: true,
                    response: true,
                },
                query: Query::NonVoid(NonVoid {
                    length: 4u32.into(),
                    file: FileOffset {
                        file_id: 5,
                        offset: 6u32.into(),
                    },
                }),
                opcode: OpCode::BreakQuery,
            }),
            &hex!("C9 00 04  05 06"),
        )
    }

    #[test]
    fn test_verify_checksum() {
        test_item(
            Action::VerifyChecksum(ActionQuery {
                header: ActionHeader {
                    group: false,
                    response: false,
                },
                query: Query::NonVoid(NonVoid {
                    length: 4u32.into(),
                    file: FileOffset {
                        file_id: 5,
                        offset: 6u32.into(),
                    },
                }),
                opcode: OpCode::VerifyChecksum,
            }),
            &hex!("0B 00 04  05 06"),
        )
    }

    #[test]
    fn test_forward() {
        test_item(
            Action::Forward(Forward::new(true, InterfaceConfiguration::Host)),
            &hex!("72 00"),
        )
    }

    #[test]
    fn test_forward_serial() {
        test_item(
            Action::Forward(Forward::new(false, InterfaceConfiguration::Serial)),
            &hex!("32 01"),
        )
    }

    #[test]
    fn test_indirect_forward_dash7_serialization() {
        use crate::link::AccessClass;

        let item = Action::IndirectForward(IndirectForward::new(
            true,
            Some(IndirectInterface::Dash7(Addressee::new(
                #[cfg(feature = "_wizzilab")]
                false,
                #[cfg(feature = "_wizzilab")]
                GroupCondition::Any,
                Address::VId(0xABCD),
                NlsState::AesCcm32([1, 2, 3, 4, 5]),
                AccessClass::unavailable(),
            ))),
        ));

        let data = &hex!("F3 D7 37 FF ABCD 01 02 03 04 05");

        test_item(item, data);
    }

    #[test]
    fn test_indirect_forward_dash7_deserialization() {
        let input = &hex!("F3 D7 37 FF ABCD 01 02 03 04 05");

        let expected = Action::IndirectForward(IndirectForward::new(
            true,
            Some(IndirectInterface::Dash7(Addressee::new(
                #[cfg(feature = "_wizzilab")]
                false,
                #[cfg(feature = "_wizzilab")]
                GroupCondition::Any,
                Address::VId(0xABCD),
                NlsState::AesCcm32([1, 2, 3, 4, 5]),
                AccessClass::unavailable(),
            ))),
        ));

        test_item(expected, input);
    }

    #[test]
    fn test_request_tag() {
        test_item(
            Action::RequestTag(RequestTag {
                header: RequestTagHeader {
                    end_of_packet: true,
                },
                id: 8,
                opcode: OpCode::RequestTag,
            }),
            &hex!("B4 08"),
        )
    }

    #[test]
    fn test_logic() {
        test_item(
            Action::Logic(Logic {
                logic: LogicOp::Nand,
                opcode: OpCode::Logic,
            }),
            &[0b1111_0001],
        )
    }

    #[test]
    fn test_chunk() {
        test_item(Action::Chunk(ChunkStep::End.into()), &[0b1011_0000])
    }

    #[test]
    fn test_response_tag() {
        test_item(
            Action::ResponseTag(ResponseTag {
                header: ResponseTagHeader {
                    end_of_packet: true,
                    error: false,
                },
                id: 8,
                opcode: OpCode::ResponseTag,
            }),
            &hex!("A3 08"),
        )
    }

    #[test]
    fn test_status() {
        test_item(
            Action::Status(
                Status::Action(ActionStatus {
                    action_id: 2,
                    status: StatusCode::UnknownOperation,
                })
                .into(),
            ),
            &hex!("22 02 F6"),
        )
    }

    #[test]
    fn test_extension() {
        test_item(
            Action::Extension(Extension {
                header: ActionHeader {
                    group: true,
                    response: true,
                },
                opcode: OpCode::Extension,
            }),
            &[0xFF],
        )
    }

    #[test]
    fn test_interface_status() {
        let data = &hex!("62 D7 14 32 00 32 2D 3E 50 80 00 00 58 20 01 39 38 38 37 00 39 00 2E");

        let item = Action::Status(
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
        );

        test_item(item, data);
    }
}
