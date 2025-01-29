use std::borrow::Cow;
#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use alloc::fmt;


use deku::{
    no_std_io::{self, Seek, Write},
    prelude::*,
};

use crate::{
    file::{OtherFile, FileCtx},
    utils::{from_bytes, from_reader},
};

use super::operation::{
    ActionQuery, Chunk, CopyFile, Extension, FileDataOperand, FileIdOperand, FilePropertiesOperand,
    Forward, IndirectForward, Logic, Nop, PermissionRequest, ReadFileData, RequestTag, ResponseTag,
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
    Other(u8),
}

impl OpCode {
    pub fn write<W: Write + Seek>(writer: &mut Writer<W>, opcode: OpCode) -> Result<(), DekuError> {
        opcode.to_writer(writer, ())
    }
}

// ===============================================================================
// Actions
// ===============================================================================

#[derive(Debug, Clone, PartialEq, strum::Display)]
pub enum Action<F = OtherFile>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    /// Nop
    Nop(Nop),
    /// Read
    ReadFileData(ReadFileData),

    ReadFileProperties(FileIdOperand),

    // Write
    WriteFileData(FileDataOperand<F>),
    WriteFileDataFlush(FileDataOperand<F>),
    WriteFileProperties(FilePropertiesOperand),

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
    ExistFile(FileIdOperand),
    CreateNewFile(FilePropertiesOperand),
    DeleteFile(FileIdOperand),
    RestoreFile(FileIdOperand),
    FlushFile(FileIdOperand),
    CopyFile(CopyFile),
    ExecuteFile(FileIdOperand),

    // Response
    ReturnFileData(FileDataOperand<F>),
    ReturnFileProperties(FilePropertiesOperand),
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

impl<'a, F> DekuReader<'a, ()> for Action<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    fn from_reader_with_ctx<R>(
        reader: &mut Reader<R>,
        _: (),
    ) -> Result<Self, DekuError>
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
                read_action!(ReadFileProperties, FileIdOperand, reader, code)
            }
            OpCode::WriteFileData => read_action!(WriteFileData, FileDataOperand<F>, reader, code),
            OpCode::WriteFileDataFlush => {
                read_action!(WriteFileDataFlush, FileDataOperand<F>, reader, code)
            }
            OpCode::WriteFileProperties => {
                read_action!(WriteFileProperties, FilePropertiesOperand, reader, code)
            }
            OpCode::ActionQuery => read_action!(ActionQuery, ActionQuery, reader, code),
            OpCode::BreakQuery => read_action!(BreakQuery, ActionQuery, reader, code),
            OpCode::PermissionRequest => {
                read_action!(PermissionRequest, PermissionRequest, reader, code)
            }
            OpCode::VerifyChecksum => {
                read_action!(VerifyChecksum, ActionQuery, reader, code)
            }
            OpCode::ExistFile => read_action!(ExistFile, FileIdOperand, reader, code),
            OpCode::CreateNewFile => {
                read_action!(CreateNewFile, FilePropertiesOperand, reader, code)
            }
            OpCode::DeleteFile => read_action!(DeleteFile, FileIdOperand, reader, code),
            OpCode::RestoreFile => read_action!(RestoreFile, FileIdOperand, reader, code),
            OpCode::FlushFile => read_action!(FlushFile, FileIdOperand, reader, code),
            OpCode::CopyFile => read_action!(CopyFile, CopyFile, reader, code),
            OpCode::ExecuteFile => read_action!(ExecuteFile, FileIdOperand, reader, code),
            OpCode::ReturnFileData => {
                read_action!(ReturnFileData, FileDataOperand<F>, reader, code)
            }
            OpCode::ReturnFileProperties => {
                read_action!(ReturnFileProperties, FilePropertiesOperand, reader, code)
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


impl<F> TryFrom<&'_ [u8]> for Action<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx> + fmt::Debug,
{
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

impl<F> DekuContainerRead<'_> for Action<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx> + fmt::Debug,
{
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

impl<F> DekuEnumExt<'_, OpCode> for Action<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    fn deku_id(&self) -> Result<OpCode, DekuError> {
        match self {
            Self::Nop(_) => Ok(OpCode::Nop),
            Self::ReadFileData(_) => Ok(OpCode::ReadFileData),
            Self::ReadFileProperties(_) => Ok(OpCode::ReadFileProperties),
            Self::WriteFileData(_) => Ok(OpCode::WriteFileData),
            Self::WriteFileDataFlush(_) => Ok(OpCode::WriteFileDataFlush),
            Self::WriteFileProperties(_) => Ok(OpCode::WriteFileProperties),
            Self::ActionQuery(_) => Ok(OpCode::ActionQuery),
            Self::BreakQuery(_) => Ok(OpCode::BreakQuery),
            Self::PermissionRequest(_) => Ok(OpCode::PermissionRequest),
            Self::VerifyChecksum(_) => Ok(OpCode::VerifyChecksum),
            Self::ExistFile(_) => Ok(OpCode::ExistFile),
            Self::CreateNewFile(_) => Ok(OpCode::CreateNewFile),
            Self::DeleteFile(_) => Ok(OpCode::DeleteFile),
            Self::RestoreFile(_) => Ok(OpCode::RestoreFile),
            Self::FlushFile(_) => Ok(OpCode::FlushFile),
            Self::CopyFile(_) => Ok(OpCode::CopyFile),
            Self::ExecuteFile(_) => Ok(OpCode::ExecuteFile),
            Self::ReturnFileData(_) => Ok(OpCode::ReturnFileData),
            Self::ReturnFileProperties(_) => Ok(OpCode::ReturnFileProperties),
            Self::ResponseTag(_) => Ok(OpCode::ResponseTag),
            #[cfg(feature = "_wizzilab")]
            Self::TxStatus(_) => Ok(OpCode::TxStatus),
            Self::Chunk(_) => Ok(OpCode::Chunk),
            Self::Logic(_) => Ok(OpCode::Logic),
            Self::Status(_) => Ok(OpCode::Status),
            Self::Forward(_) => Ok(OpCode::Forward),
            Self::IndirectForward(_) => Ok(OpCode::IndirectForward),
            Self::RequestTag(_) => Ok(OpCode::RequestTag),
            Self::Extension(_) => Ok(OpCode::Extension),
        }
    }
}

impl<F> DekuWriter<()> for Action<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    fn to_writer<W>(&self, writer: &mut Writer<W>, _: ()) -> Result<(), DekuError>
    where
        W: no_std_io::Write + no_std_io::Seek,
    {
        let opcode = self.deku_id()?;

        match self {
            Self::Nop(action) => action.to_writer(writer, opcode)?,
            Self::ReadFileData(action) => action.to_writer(writer, opcode)?,
            Self::ReadFileProperties(action) => action.to_writer(writer, opcode)?,
            Self::WriteFileData(action) => action.to_writer(writer, opcode)?,
            Self::WriteFileDataFlush(action) => action.to_writer(writer, opcode)?,
            Self::WriteFileProperties(action) => action.to_writer(writer, opcode)?,
            Self::ActionQuery(action) => action.to_writer(writer, opcode)?,
            Self::BreakQuery(action) => action.to_writer(writer, opcode)?,
            Self::PermissionRequest(action) => action.to_writer(writer, opcode)?,
            Self::VerifyChecksum(action) => action.to_writer(writer, opcode)?,
            Self::ExistFile(action) => action.to_writer(writer, opcode)?,
            Self::CreateNewFile(action) => action.to_writer(writer, opcode)?,
            Self::DeleteFile(action) => action.to_writer(writer, opcode)?,
            Self::RestoreFile(action) => action.to_writer(writer, opcode)?,
            Self::FlushFile(action) => action.to_writer(writer, opcode)?,
            Self::CopyFile(action) => action.to_writer(writer, opcode)?,
            Self::ExecuteFile(action) => action.to_writer(writer, opcode)?,
            Self::ReturnFileData(action) => action.to_writer(writer, opcode)?,
            Self::ReturnFileProperties(action) => action.to_writer(writer, opcode)?,
            Self::ResponseTag(action) => action.to_writer(writer, opcode)?,
            #[cfg(feature = "_wizzilab")]
            Self::TxStatus(action) => action.to_writer(writer, opcode)?,
            Self::Chunk(action) => action.to_writer(writer, opcode)?,
            Self::Logic(action) => action.to_writer(writer, opcode)?,
            Self::Status(action) => action.to_writer(writer, opcode)?,
            Self::Forward(action) => action.to_writer(writer, opcode)?,
            Self::IndirectForward(action) => action.to_writer(writer, opcode)?,
            Self::RequestTag(action) => action.to_writer(writer, opcode)?,
            Self::Extension(action) => action.to_writer(writer, opcode)?,
        }

        Ok(())
    }
}

impl<F> TryFrom<Action<F>> for Vec<u8>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    type Error = DekuError;
    fn try_from(input: Action<F>) -> Result<Self, Self::Error> {
        DekuContainerWrite::to_bytes(&input)
    }
}
impl<F> DekuContainerWrite for Action<F> where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>
{
}

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
        file::{FileData, File},
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
            Action::<OtherFile>::Nop(Nop {
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
            Action::<OtherFile>::ReadFileData(ReadFileData {
                header: ActionHeader {
                    group: false,
                    response: true,
                },

                file_id: 1,
                offset: 2u32.into(),

                length: 3u32.into(),
                opcode: OpCode::ReadFileData,
            }),
            &hex!("41 01 02 03"),
        )
    }

    #[test]
    fn test_read_file_properties() {
        test_item(
            Action::<OtherFile>::ReadFileProperties(FileIdOperand {
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
            Action::<OtherFile>::WriteFileData(FileDataOperand::new(
                ActionHeader {
                    group: true,
                    response: false,
                },
                FileData {
                    id: 0xF9,
                    offset: 5u32.into(),
                    file: File::User(
                        OtherFile{data}
                    ),
                },
                OpCode::WriteFileData,
            )),
            &hex!("84 F9 05 03 010203"),
        )
    }

    #[test]
    fn test_return_file_properties() {
        test_item(
            Action::<OtherFile>::ReturnFileProperties(FilePropertiesOperand {
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
            Action::<OtherFile>::WriteFileProperties(FilePropertiesOperand {
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
            Action::<OtherFile>::PermissionRequest(PermissionRequest {
                header: ActionHeader {
                    group: false,
                    response: false,
                },
                level: PermissionLevel::Root,
                permission: Permission::Dash7(0x01_02_03_04_05_06_07_08),
                opcode: OpCode::PermissionRequest,
            }),
            &hex!("0A 01 42 0102030405060708"),
        )
    }

    #[test]
    fn test_exist_file() {
        test_item(
            Action::<OtherFile>::ExistFile(FileIdOperand {
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
            Action::<OtherFile>::CreateNewFile(FilePropertiesOperand {
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
            Action::<OtherFile>::DeleteFile(FileIdOperand {
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
            Action::<OtherFile>::RestoreFile(FileIdOperand {
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
            Action::<OtherFile>::FlushFile(FileIdOperand {
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
            Action::<OtherFile>::CopyFile(CopyFile {
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
            Action::<OtherFile>::ExecuteFile(FileIdOperand {
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
            Action::<OtherFile>::ReturnFileData(FileDataOperand::new(
                ActionHeader {
                    group: false,
                    response: false,
                },
                FileData {
                    offset: 5u32.into(),
                    id: 0xF9,
                    file: File::User(OtherFile {
                        data,
                    })
                },
                OpCode::ReturnFileData,
            )),
            &hex!("20 F9 05 03 010203"),
        )
    }

    #[test]
    fn test_action_query() {
        test_item(
            Action::<OtherFile>::ActionQuery(ActionQuery {
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
            Action::<OtherFile>::BreakQuery(ActionQuery {
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
            Action::<OtherFile>::VerifyChecksum(ActionQuery {
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
            Action::<OtherFile>::Forward(Forward::new(true, InterfaceConfiguration::Host)),
            &hex!("72 00"),
        )
    }

    #[test]
    fn test_forward_serial() {
        test_item(
            Action::<OtherFile>::Forward(Forward::new(false, InterfaceConfiguration::Serial)),
            &hex!("32 01"),
        )
    }

    #[test]
    fn test_indirect_forward_dash7_serialization() {
        use crate::link::AccessClass;

        let item = Action::<OtherFile>::IndirectForward(IndirectForward::new(
            true,
            Some(IndirectInterface::Dash7(Addressee::new(
                #[cfg(feature = "_wizzilab")]
                false,
                #[cfg(feature = "_wizzilab")]
                GroupCondition::Any,
                Address::VId(0xABCD),
                NlsState::AesCcm32(0x01_02_03_04_05),
                AccessClass::unavailable(),
            ))),
        ));

        let data = &hex!("F3 D7 37 FF ABCD 01 02 03 04 05");

        test_item(item, data);
    }

    #[test]
    fn test_indirect_forward_dash7_deserialization() {
        let input = &hex!("F3 D7 37 FF ABCD 01 02 03 04 05");

        let expected = Action::<OtherFile>::IndirectForward(IndirectForward::new(
            true,
            Some(IndirectInterface::Dash7(Addressee::new(
                #[cfg(feature = "_wizzilab")]
                false,
                #[cfg(feature = "_wizzilab")]
                GroupCondition::Any,
                Address::VId(0xABCD),
                NlsState::AesCcm32(0x01_02_03_04_05),
                AccessClass::unavailable(),
            ))),
        ));

        test_item(expected, input);
    }

    #[test]
    fn test_request_tag() {
        test_item(
            Action::<OtherFile>::RequestTag(RequestTag {
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
            Action::<OtherFile>::Logic(Logic {
                logic: LogicOp::Nand,
                opcode: OpCode::Logic,
            }),
            &[0b1111_0001],
        )
    }

    #[test]
    fn test_chunk() {
        test_item(Action::<OtherFile>::Chunk(ChunkStep::End.into()), &[0b1011_0000])
    }

    #[test]
    fn test_response_tag() {
        test_item(
            Action::<OtherFile>::ResponseTag(ResponseTag {
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
            Action::<OtherFile>::Status(
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
            Action::<OtherFile>::Extension(Extension {
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

        let item = Action::<OtherFile>::Status(
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
