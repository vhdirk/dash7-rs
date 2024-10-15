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

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpCode(#[deku(bits = 6)] pub u8);

impl OpCode {
    pub const NOP: OpCode = OpCode(0u8);

    // Read
    pub const READ_FILE_DATA: OpCode = OpCode(0x01u8);
    pub const READ_FILE_PROPERTIES: OpCode = OpCode(0x02u8);

    // Write
    pub const WRITE_FILE_DATA: OpCode = OpCode(0x04u8);
    pub const WRITE_FILE_DATA_FLUSH: OpCode = OpCode(0x05u8);
    pub const WRITE_FILE_PROPERTIES: OpCode = OpCode(0x06u8);
    pub const ACTION_QUERY: OpCode = OpCode(0x08u8);
    pub const BREAK_QUERY: OpCode = OpCode(0x09u8);
    pub const PERMISSION_REQUEST: OpCode = OpCode(0x0au8);
    pub const VERIFY_CHECKSUM: OpCode = OpCode(0x0bu8);

    // Management
    pub const EXIST_FILE: OpCode = OpCode(16u8);
    pub const CREATE_NEW_FILE: OpCode = OpCode(17u8);
    pub const DELETE_FILE: OpCode = OpCode(18u8);
    pub const RESTORE_FILE: OpCode = OpCode(19u8);
    pub const FLUSH_FILE: OpCode = OpCode(20u8);
    pub const COPY_FILE: OpCode = OpCode(23u8);
    pub const EXECUTE_FILE: OpCode = OpCode(31u8);

    // Response
    pub const RETURN_FILE_DATA: OpCode = OpCode(32u8);
    pub const RETURN_FILE_PROPERTIES: OpCode = OpCode(33u8);
    pub const STATUS: OpCode = OpCode(34u8);
    pub const RESPONSE_TAG: OpCode = OpCode(35u8);

    #[cfg(feature = "_wizzilab")]
    pub const TX_STATUS: OpCode = OpCode(38u8);

    // Special
    pub const CHUNK: OpCode = OpCode(48u8);
    pub const LOGIC: OpCode = OpCode(49u8);
    pub const FORWARD: OpCode = OpCode(50u8);
    pub const INDIRECT_FORWARD: OpCode = OpCode(51u8);
    pub const REQUEST_TAG: OpCode = OpCode(52u8);
    pub const EXTENSION: OpCode = OpCode(63u8);
}

impl OpCode {
    pub fn write<W: Write + Seek>(writer: &mut Writer<W>, opcode: OpCode) -> Result<(), DekuError> {
        opcode.to_writer(writer, ())
    }
}

// ===============================================================================
// Actions
// ===============================================================================

#[derive(Debug, Clone, PartialEq)]
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
            OpCode::NOP => read_action!(Nop, Nop, reader, code),
            OpCode::READ_FILE_DATA => {
                read_action!(ReadFileData, ReadFileData, reader, code)
            }
            OpCode::READ_FILE_PROPERTIES => {
                read_action!(ReadFileProperties, FileId, reader, code)
            }
            OpCode::WRITE_FILE_DATA => read_action!(WriteFileData, FileData, reader, code),
            OpCode::WRITE_FILE_DATA_FLUSH => {
                read_action!(WriteFileDataFlush, FileData, reader, code)
            }
            OpCode::WRITE_FILE_PROPERTIES => {
                read_action!(WriteFileProperties, FileProperties, reader, code)
            }
            OpCode::ACTION_QUERY => read_action!(ActionQuery, ActionQuery, reader, code),
            OpCode::BREAK_QUERY => read_action!(BreakQuery, ActionQuery, reader, code),
            OpCode::PERMISSION_REQUEST => {
                read_action!(PermissionRequest, PermissionRequest, reader, code)
            }
            OpCode::VERIFY_CHECKSUM => {
                read_action!(VerifyChecksum, ActionQuery, reader, code)
            }
            OpCode::EXIST_FILE => read_action!(ExistFile, FileId, reader, code),
            OpCode::CREATE_NEW_FILE => {
                read_action!(CreateNewFile, FileProperties, reader, code)
            }
            OpCode::DELETE_FILE => read_action!(DeleteFile, FileId, reader, code),
            OpCode::RESTORE_FILE => read_action!(RestoreFile, FileId, reader, code),
            OpCode::FLUSH_FILE => read_action!(FlushFile, FileId, reader, code),
            OpCode::COPY_FILE => read_action!(CopyFile, CopyFile, reader, code),
            OpCode::EXECUTE_FILE => read_action!(ExecuteFile, FileId, reader, code),
            OpCode::RETURN_FILE_DATA => {
                read_action!(ReturnFileData, FileData, reader, code)
            }
            OpCode::RETURN_FILE_PROPERTIES => {
                read_action!(ReturnFileProperties, FileProperties, reader, code)
            }
            OpCode::RESPONSE_TAG => read_action!(ResponseTag, ResponseTag, reader, code),

            #[cfg(feature = "_wizzilab")]
            OpCode::TX_STATUS => read_action!(TxStatus, TxStatusOperation, reader, code),
            OpCode::CHUNK => read_action!(Chunk, Chunk, reader, code),
            OpCode::LOGIC => read_action!(Logic, Logic, reader, code),
            OpCode::REQUEST_TAG => read_action!(RequestTag, RequestTag, reader, code),
            OpCode::STATUS => read_action!(Status, StatusOperand, reader, code),
            OpCode::FORWARD => read_action!(Forward, Forward, reader, code),
            OpCode::INDIRECT_FORWARD => {
                read_action!(IndirectForward, IndirectForward, reader, code)
            }
            OpCode::EXTENSION => read_action!(Extension, Extension, reader, code),
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
            Action::Nop(_) => Ok(OpCode::NOP),
            Action::ReadFileData(_) => Ok(OpCode::READ_FILE_DATA),
            Action::ReadFileProperties(_) => Ok(OpCode::READ_FILE_PROPERTIES),
            Action::WriteFileData(_) => Ok(OpCode::WRITE_FILE_DATA),
            Action::WriteFileDataFlush(_) => Ok(OpCode::WRITE_FILE_DATA_FLUSH),
            Action::WriteFileProperties(_) => Ok(OpCode::WRITE_FILE_PROPERTIES),
            Action::ActionQuery(_) => Ok(OpCode::ACTION_QUERY),
            Action::BreakQuery(_) => Ok(OpCode::BREAK_QUERY),
            Action::PermissionRequest(_) => Ok(OpCode::PERMISSION_REQUEST),
            Action::VerifyChecksum(_) => Ok(OpCode::VERIFY_CHECKSUM),
            Action::ExistFile(_) => Ok(OpCode::EXIST_FILE),
            Action::CreateNewFile(_) => Ok(OpCode::CREATE_NEW_FILE),
            Action::DeleteFile(_) => Ok(OpCode::DELETE_FILE),
            Action::RestoreFile(_) => Ok(OpCode::RESTORE_FILE),
            Action::FlushFile(_) => Ok(OpCode::FLUSH_FILE),
            Action::CopyFile(_) => Ok(OpCode::COPY_FILE),
            Action::ExecuteFile(_) => Ok(OpCode::EXECUTE_FILE),
            Action::ReturnFileData(_) => Ok(OpCode::RETURN_FILE_DATA),
            Action::ReturnFileProperties(_) => Ok(OpCode::RETURN_FILE_PROPERTIES),
            Action::ResponseTag(_) => Ok(OpCode::RESPONSE_TAG),
            #[cfg(feature = "_wizzilab")]
            Action::TxStatus(_) => Ok(OpCode::TX_STATUS),
            Action::Chunk(_) => Ok(OpCode::CHUNK),
            Action::Logic(_) => Ok(OpCode::LOGIC),
            Action::Status(_) => Ok(OpCode::STATUS),
            Action::Forward(_) => Ok(OpCode::FORWARD),
            Action::IndirectForward(_) => Ok(OpCode::INDIRECT_FORWARD),
            Action::RequestTag(_) => Ok(OpCode::REQUEST_TAG),
            Action::Extension(_) => Ok(OpCode::EXTENSION),
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
                opcode: OpCode::NOP,
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
                opcode: OpCode::READ_FILE_DATA,
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
                opcode: OpCode::READ_FILE_PROPERTIES,
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
                File::Other(data),
                OpCode::WRITE_FILE_DATA,
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
                opcode: OpCode::RETURN_FILE_PROPERTIES,
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
                opcode: OpCode::WRITE_FILE_PROPERTIES,
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
                opcode: OpCode::PERMISSION_REQUEST,
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
                opcode: OpCode::EXIST_FILE,
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
                opcode: OpCode::CREATE_NEW_FILE,
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
                opcode: OpCode::DELETE_FILE,
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
                opcode: OpCode::RESTORE_FILE,
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
                opcode: OpCode::FLUSH_FILE,
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
                opcode: OpCode::COPY_FILE,
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
                opcode: OpCode::EXECUTE_FILE,
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
                File::Other(data),
                OpCode::RETURN_FILE_DATA,
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
                opcode: OpCode::ACTION_QUERY,
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
                opcode: OpCode::BREAK_QUERY,
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
                opcode: OpCode::VERIFY_CHECKSUM,
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
                opcode: OpCode::REQUEST_TAG,
            }),
            &hex!("B4 08"),
        )
    }

    #[test]
    fn test_logic() {
        test_item(
            Action::Logic(Logic {
                logic: LogicOp::Nand,
                opcode: OpCode::LOGIC,
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
                opcode: OpCode::RESPONSE_TAG,
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
                    status: StatusCode::UNKNOWN_OPERATION,
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
                opcode: OpCode::EXTENSION,
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
