use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    ctx::{BitSize, Endian},
    prelude::*,
};

use crate::alp::filesystem::FileHeader;

use super::operand::{FileOffset, Length};

// ===============================================================================
// Opcodes
// ===============================================================================

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
#[deku(bits = 6, type = "u8")]
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
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct Operation(
    #[deku(
        reader = "Operation::read(deku::rest)",
        writer = "Operation::write(deku::output, &self.0)"
    )]
    pub Action,
);

impl From<Action> for Operation {
    fn from(value: Action) -> Self {
        Operation(value)
    }
}

impl Into<Action> for Operation {
    fn into(self) -> Action {
        self.0
    }
}

impl Operation {
    fn read(input: &BitSlice<u8, Msb0>) -> Result<(&BitSlice<u8, Msb0>, Action), DekuError> {
        let (rest, _) = <u8 as DekuRead<'_, _>>::read(input, (Endian::Big, BitSize(2)))?;
        let (_, opcode) = <OpCode as DekuRead<'_, _>>::read(rest, ())?;

        <Action as DekuRead<'_, _>>::read(input, opcode)
    }

    fn write(output: &mut BitVec<u8, Msb0>, action: &Action) -> Result<(), DekuError> {
        DekuWrite::write(action, output, action.deku_id().unwrap())?;

        use bitvec::field::BitField;
        // TODO: proper errors
        let opcode = action.deku_id().unwrap().deku_id().unwrap() as u8;
        // now write the opcode with offset 2
        output[2..8].store_be(opcode);
        Ok(())
    }
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(ctx = "opcode: OpCode", id = "opcode")]
pub enum Action {
    /// Nop
    #[deku(id = "OpCode::Nop")]
    Nop(Nop),
    /// Read
    #[deku(id = "OpCode::ReadFileData")]
    ReadFileData(ReadFileData),

    #[deku(id = "OpCode::ReadFileProperties")]
    ReadFileProperties(ReadFileProperties),

    // Write
    #[deku(id = "OpCode::WriteFileData")]
    WriteFileData(WriteFileData),
    // #[deku(id = "5")]
    // WriteFileDataFlush(FileDataAction),
    // #[deku(id = "6")]
    // WriteFileProperties(FilePropertiesAction),
    // #[deku(id = "8")]
    // ActionQuery(QueryAction),
    // #[deku(id = "9")]
    // BreakQuery(QueryAction),
    // #[deku(id = "10")]
    // PermissionRequest(PermissionRequest),
    // #[deku(id = "11")]
    // VerifyChecksum(QueryAction),

    // // Management
    // #[deku(id = "16")]
    // ExistFile(FileIdAction),
    // #[deku(id = "17")]
    // CreateNewFile(FilePropertiesAction),
    // #[deku(id = "18")]
    // DeleteFile(FileIdAction),
    // #[deku(id = "19")]
    // RestoreFile(FileIdAction),
    // #[deku(id = "20")]
    // FlushFile(FileIdAction),
    // #[deku(id = "23")]
    // CopyFile(CopyFile),
    // #[deku(id = "31")]
    // ExecuteFile(FileIdAction),

    // // Response
    // #[deku(id = "32")]
    // ReturnFileData(FileDataAction),
    // #[deku(id = "33")]
    // ReturnFileProperties(FilePropertiesAction),
    // #[deku(id = "34")]
    // Status(Status),
    // #[deku(id = "35")]
    // ResponseTag(ResponseTag),

    // // Special
    // #[deku(id = "48")]
    // Chunk(Chunk),
    // #[deku(id = "49")]
    // Logic(Logic),
    // #[deku(id = "50")]
    // Forward(Forward),
    // #[deku(id = "51")]
    // IndirectForward(IndirectForward),
    // #[deku(id = "52")]
    // RequestTag(RequestTag),
    // #[deku(id = "63")]
    // Extension(Extension),
}

/// File access type event that will trigger an ALP action.
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
#[deku(bits = 3, type = "u8")]
pub enum ActionCondition {
    /// Check for existence
    #[default]
    #[deku(id = "0")]
    List,
    /// Trigger upon file read
    #[deku(id = "1")]
    Read,
    /// Trigger upon file write
    #[deku(id = "2")]
    Write,
    /// Trigger upon file write-flush
    // ALP_SPEC Action write-flush does not exist. Only write and flush exist.
    #[deku(id = "3")]
    WriteFlush,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ActionHeader {
    /// Group with next action
    #[deku(bits = 1)]
    pub group: bool,
    /// Ask for a response (status)
    #[deku(bits = 1, pad_bits_after = "6")]
    pub response: bool,
    //opcode would be here. 6 bits padding instead
}

// ===============================================================================
// Actions
// ===============================================================================
// Nop
/// Does nothing
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct Nop {
    pub header: ActionHeader,
}

/// Checks whether a file exists
// ALP_SPEC: How is the result of this command different from a read file of size 0?
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct FileIdAction {
    pub header: ActionHeader,
    pub file_id: u8,
}

// Write data to a file
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct WriteFileData {
    pub header: ActionHeader,
    pub offset: FileOffset,
    pub length: Length,

    #[deku(count="Into::<u32>::into(*length)")]
    pub data: Vec<u8>,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct FilePropertiesAction {
    pub header: ActionHeader,

    pub file_id: u8,
    pub file_header: FileHeader,
}

// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct QueryAction {
//     pub header: ActionHeader,

//     pub query: operand::Query,
// }

// Read
/// Read data from a file
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ReadFileData {
    pub header: ActionHeader,

    pub offset: FileOffset,
    pub length: Length,
}

/// Read properties of a file
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ReadFileProperties {
    pub header: ActionHeader,

    pub file_id: u8,
}

/// Write the properties of a file
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct WriteFileProperties {
    pub header: ActionHeader,

    pub file_id: u8,
    pub file_header: FileHeader,
}
// impl_header_op!(WriteFileProperties, group, resp, file_id, header);
// #[test]
// fn test_write_file_properties() {
//     test_item(
//         WriteFileProperties {
//             group: true,
//             resp: false,
//             file_id: 9,
//             header: data::FileHeader {
//                 permissions: data::Permissions {
//                     encrypted: true,
//                     executable: false,
//                     user: data::UserPermissions {
//                         read: true,
//                         write: true,
//                         run: true,
//                     },
//                     guest: data::UserPermissions {
//                         read: false,
//                         write: false,
//                         run: false,
//                     },
//                 },
//                 properties: data::FileProperties {
//                     act_en: false,
//                     act_cond: data::ActionCondition::Read,
//                     storage_class: data::StorageClass::Permanent,
//                 },
//                 alp_cmd_fid: 1,
//                 interface_file_id: 2,
//                 file_size: 0xDEAD_BEEF,
//                 allocated_size: 0xBAAD_FACE,
//             },
//         },
//         &hex!("86   09   B8 13 01 02 DEADBEEF BAADFACE"),
//     )
// }

/// Add a condition on the execution of the next group of action.
///
/// If the condition is not met, the next group of action should be skipped.
// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct ActionQuery {
//     pub header: ActionHeader,
//     pub query: operand::Query,
// }
// impl_op_serialized!(
//     ActionQuery,
//     group,
//     resp,
//     query,
//     Query,
//     operand::QueryDecodingError
// );
// #[test]
// fn test_action_query() {
//     test_item(
//         ActionQuery {
//             group: true,
//             resp: true,
//             query: operand::Query::NonVoid(operand::NonVoid {
//                 size: 4,
//                 file: operand::FileOffset { id: 5, offset: 6 },
//             }),
//         },
//         &hex!("C8   00 04  05 06"),
//     )
// }

/// Add a condition to continue the processing of this ALP command.
///
/// If the condition is not met the all the next ALP action of this command should be ignored.
// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct BreakQuery {
//     /// Group with next action
//     pub header: ActionHeader,
//     pub query: Query,
// }
// impl_op_serialized!(
//     BreakQuery,
//     group,
//     resp,
//     query,
//     Query,
//     operand::QueryDecodingError
// );
// #[test]
// fn test_break_query() {
//     test_item(
//         BreakQuery {
//             group: true,
//             resp: true,
//             query: operand::Query::NonVoid(operand::NonVoid {
//                 size: 4,
//                 file: operand::FileOffset { id: 5, offset: 6 },
//             }),
//         },
//         &hex!("C9   00 04  05 06"),
//     )
// }

/// Request a level of permission using some permission type
// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct PermissionRequest {
//     pub header: ActionHeader,
//     /// See operand::permission_level
//     pub level: u8,
//     pub permission: Permission,
// }
// #[derive(Debug, Copy, Clone, Hash, PartialEq)]
// pub enum PermissionRequestDecodingError {
//     MissingBytes(usize),
//     Permission(operand::PermissionDecodingError),
// }
// impl Codec for PermissionRequest {
//     type Error = PermissionRequestDecodingError;
//     fn encoded_size(&self) -> usize {
//         1 + 1 + encoded_size!(self.permission)
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         out[0] = control_byte!(self.group, self.resp, OpCode::PermissionRequest);
//         out[1] = self.level;
//         1 + serialize_all!(&mut out[2..], self.permission)
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.is_empty() {
//             Err(WithOffset::new_head(Self::Error::MissingBytes(1)))
//         } else {
//             let mut offset = 1;
//             let level = out[offset];
//             offset += 1;
//             let WithSize {
//                 value: permission,
//                 size,
//             } = operand::Permission::decode(&out[offset..])
//                 .map_err(|e| e.shift(offset).map_value(Self::Error::Permission))?;
//             offset += size;
//             Ok(WithSize {
//                 value: Self {
//                     group: out[0] & 0x80 != 0,
//                     resp: out[0] & 0x40 != 0,
//                     level,
//                     permission,
//                 },
//                 size: offset,
//             })
//         }
//     }
// }
// #[test]
// fn test_permission_request() {
//     test_item(
//         PermissionRequest {
//             group: false,
//             resp: false,
//             level: operand::permission_level::ROOT,
//             permission: operand::Permission::Dash7(hex!("0102030405060708")),
//         },
//         &hex!("0A   01 42 0102030405060708"),
//     )
// }

/// Calculate checksum of file and compare with checksum in query
// ALP_SPEC: Is the checksum calculated on the targeted data (offset, size) or the whole file?
// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct VerifyChecksum {
//     pub header: ActionHeader,
//     pub query: Query,
// }
// impl_op_serialized!(
//     VerifyChecksum,
//     group,
//     resp,
//     query,
//     Query,
//     operand::QueryDecodingError
// );
// #[test]
// fn test_verify_checksum() {
//     test_item(
//         VerifyChecksum {
//             group: false,
//             resp: false,
//             query: operand::Query::NonVoid(operand::NonVoid {
//                 size: 4,
//                 file: operand::FileOffset { id: 5, offset: 6 },
//             }),
//         },
//         &hex!("0B   00 04  05 06"),
//     )
// }

// Management
/// Checks whether a file exists
// ALP_SPEC: How is the result of this command different from a read file of size 0?
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ExistFile {
    /// Group with next action
    pub header: ActionHeader,
    pub file_id: u8,
}
// impl_simple_op!(ExistFile, group, resp, file_id);
// #[test]
// fn test_exist_file() {
//     test_item(
//         ExistFile {
//             group: false,
//             resp: false,
//             file_id: 9,
//         },
//         &hex!("10 09"),
//     )
// }

/// Create a new file
// ALP_SPEC: How do you create a remote file? Is this Wizzilab specific.
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct CreateNewFile {
    /// Group with next action
    pub header: ActionHeader,
    pub file_id: u8,
    pub file_header: FileHeader,
}
// impl_header_op!(CreateNewFile, group, resp, file_id, header);
// #[test]
// fn test_create_new_file() {
//     test_item(
//         CreateNewFile {
//             group: true,
//             resp: false,
//             file_id: 3,
//             header: data::FileHeader {
//                 permissions: data::Permissions {
//                     encrypted: true,
//                     executable: false,
//                     user: data::UserPermissions {
//                         read: true,
//                         write: true,
//                         run: true,
//                     },
//                     guest: data::UserPermissions {
//                         read: false,
//                         write: false,
//                         run: false,
//                     },
//                 },
//                 properties: data::FileProperties {
//                     act_en: false,
//                     act_cond: data::ActionCondition::Read,
//                     storage_class: data::StorageClass::Permanent,
//                 },
//                 alp_cmd_fid: 1,
//                 interface_file_id: 2,
//                 file_size: 0xDEAD_BEEF,
//                 allocated_size: 0xBAAD_FACE,
//             },
//         },
//         &hex!("91   03   B8 13 01 02 DEADBEEF BAADFACE"),
//     )
// }

/// Deletes a file.
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteFile {
    pub header: ActionHeader,
    pub file_id: u8,
}
// impl_simple_op!(DeleteFile, group, resp, file_id);
// #[test]
// fn test_delete_file() {
//     test_item(
//         DeleteFile {
//             group: false,
//             resp: true,
//             file_id: 9,
//         },
//         &hex!("52 09"),
//     )
// }

/// Restores a restorable file
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct RestoreFile {
    pub header: ActionHeader,
    pub file_id: u8,
}
// impl_simple_op!(RestoreFile, group, resp, file_id);
// #[test]
// fn test_restore_file() {
//     test_item(
//         RestoreFile {
//             group: true,
//             resp: true,
//             file_id: 9,
//         },
//         &hex!("D3 09"),
//     )
// }

/// Flush a file
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct FlushFile {
    pub header: ActionHeader,

    pub file_id: u8,
}
// impl_simple_op!(FlushFile, group, resp, file_id);
// #[test]
// fn test_flush_file() {
//     test_item(
//         FlushFile {
//             group: false,
//             resp: false,
//             file_id: 9,
//         },
//         &hex!("14 09"),
//     )
// }

/// Copy a file to another file
// ALP_SPEC: What does that mean? Is it a complete file copy including the file properties or just
// the data? If not then if the destination file is bigger than the source, does the copy only
// overwrite the first part of the destination file?
//
// Wouldn't it be more appropriate to have 1 size and 2 file offsets?
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct CopyFile {
    pub header: ActionHeader,
    pub src_file_id: u8,
    pub dst_file_id: u8,
}
// impl_simple_op!(CopyFile, group, resp, src_file_id, dst_file_id);
// #[test]
// fn test_copy_file() {
//     test_item(
//         CopyFile {
//             group: false,
//             resp: false,
//             src_file_id: 0x42,
//             dst_file_id: 0x24,
//         },
//         &hex!("17 42 24"),
//     )
// }

/// Execute a file if executable
// ALP_SPEC: Is that an "ALP executable" or a binary executable?
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ExecuteFile {
    pub header: ActionHeader,
    pub file_id: u8,
}
// impl_simple_op!(ExecuteFile, group, resp, file_id);
// #[test]
// fn test_execute_file() {
//     test_item(
//         ExecuteFile {
//             group: false,
//             resp: false,
//             file_id: 9,
//         },
//         &hex!("1F 09"),
//     )
// }

// Response
/// Responds to a ReadFileData request.
///
/// This can also be used to report unsollicited data.
// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct ReturnFileData {
//     /// Group with next action
//     pub header: ActionHeader,
//     pub file_id: u8,

//     pub offset: u32,
//     pub data: Vec<u8>,
// }
// impl ReturnFileData {
//     pub fn validate(&self) -> Result<(), OperandValidationError> {
//         if self.offset > varint::MAX {
//             return Err(OperandValidationError::OffsetTooBig);
//         }
//         let size = self.data.len() as u32;
//         if size > varint::MAX {
//             return Err(OperandValidationError::SizeTooBig);
//         }
//         Ok(())
//     }
// }
// impl Codec for ReturnFileData {
//     type Error = StdError;
//     fn encoded_size(&self) -> usize {
//         1 + 1
//             + unsafe_varint_serialize_sizes!(self.offset, self.data.len() as u32) as usize
//             + self.data.len()
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         out[0] = control_byte!(self.group, self.resp, OpCode::ReturnFileData);
//         out[1] = self.file_id;
//         let mut offset = 2;
//         offset += unsafe_varint_serialize!(out[2..], self.offset, self.data.len() as u32) as usize;
//         out[offset..offset + self.data.len()].clone_from_slice(&self.data[..]);
//         offset += self.data.len();
//         offset
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         let min_size = 1 + 1 + 1 + 1;
//         if out.len() < min_size {
//             return Err(WithOffset::new(
//                 0,
//                 Self::Error::MissingBytes(min_size - out.len()),
//             ));
//         }
//         let group = out[0] & 0x80 != 0;
//         let resp = out[0] & 0x40 != 0;
//         let file_id = out[1];
//         let mut off = 2;
//         let WithSize {
//             value: offset,
//             size: offset_size,
//         } = varint::decode(&out[off..])?;
//         off += offset_size;
//         let WithSize {
//             value: size,
//             size: size_size,
//         } = varint::decode(&out[off..])?;
//         off += size_size;
//         let size = size as usize;
//         let mut data = vec![0u8; size].into_boxed_slice();
//         data.clone_from_slice(&out[off..off + size]);
//         off += size;
//         Ok(WithSize {
//             value: Self {
//                 group,
//                 resp,
//                 file_id,
//                 offset,
//                 data,
//             },
//             size: off,
//         })
//     }
// }
// #[test]
// fn test_return_file_data() {
//     test_item(
//         ReturnFileData {
//             group: false,
//             resp: false,
//             file_id: 9,
//             offset: 5,
//             data: Box::new(hex!("01 02 03")),
//         },
//         &hex!("20   09 05 03  010203"),
//     )
// }

/// Result of a ReadFileProperties request
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnFileProperties {
    /// Group with next action
    pub header: ActionHeader,
    pub file_id: u8,
    pub file_header: FileHeader,
}
// impl_header_op!(ReturnFileProperties, group, resp, file_id, header);
// #[test]
// fn test_return_file_properties() {
//     test_item(
//         ReturnFileProperties {
//             group: false,
//             resp: false,
//             file_id: 9,
//             header: data::FileHeader {
//                 permissions: data::Permissions {
//                     encrypted: true,
//                     executable: false,
//                     user: data::UserPermissions {
//                         read: true,
//                         write: true,
//                         run: true,
//                     },
//                     guest: data::UserPermissions {
//                         read: false,
//                         write: false,
//                         run: false,
//                     },
//                 },
//                 properties: data::FileProperties {
//                     act_en: false,
//                     act_cond: data::ActionCondition::Read,
//                     storage_class: data::StorageClass::Permanent,
//                 },
//                 alp_cmd_fid: 1,
//                 interface_file_id: 2,
//                 file_size: 0xDEAD_BEEF,
//                 allocated_size: 0xBAAD_FACE,
//             },
//         },
//         &hex!("21   09   B8 13 01 02 DEADBEEF BAADFACE"),
//     )
// }

// #[derive(Clone, Copy, Debug, PartialEq)]
// pub enum StatusType {
//     Action = 0,
//     Interface = 1,
// }
// impl StatusType {
//     fn from(n: u8) -> Result<Self, u8> {
//         Ok(match n {
//             0 => StatusType::Action,
//             1 => StatusType::Interface,
//             x => return Err(x),
//         })
//     }
// }

/// Statuses regarding actions sent in a request
// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// #[deku(bits = 1, type = "u8")]
// pub enum Status {
//     // ALP SPEC: This is named status, but it should be named action status compared to the '2'
//     // other statuses.
//     #[deku(id="0")]Action(operand::ActionStatus),
//     #[deku(id="1")]Interface(operand::InterfaceStatus),
//     // ALP SPEC: Where are the stack errors?
// }
// #[derive(Debug, Copy, Clone, Hash, PartialEq)]
// pub enum StatusDecodingError {
//     MissingBytes(usize),
//     UnknownType(u8),
//     Action(StdError),
//     Interface(operand::InterfaceStatusDecodingError),
// }
// impl Codec for Status {
//     type Error = StatusDecodingError;
//     fn encoded_size(&self) -> usize {
//         1 + match self {
//             Status::Action(op) => op.encoded_size(),
//             Status::Interface(op) => op.encoded_size(),
//         }
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         out[0] = OpCode::Status as u8
//             + ((match self {
//                 Status::Action(_) => StatusType::Action,
//                 Status::Interface(_) => StatusType::Interface,
//             } as u8)
//                 << 6);
//         let out = &mut out[1..];
//         1 + match self {
//             Status::Action(op) => op.encode_in(out),
//             Status::Interface(op) => op.encode_in(out),
//         }
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.is_empty() {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
//         }
//         let status_type = out[0] >> 6;
//         Ok(
//             match StatusType::from(status_type)
//                 .map_err(|e| WithOffset::new_head(Self::Error::UnknownType(e)))?
//             {
//                 StatusType::Action => {
//                     let WithSize { size, value } = operand::Status::decode(&out[1..])
//                         .map_err(|e| e.shift(1).map_value(Self::Error::Action))?;
//                     WithSize {
//                         size: size + 1,
//                         value: Self::Action(value),
//                     }
//                 }
//                 StatusType::Interface => {
//                     let WithSize { size, value } = operand::InterfaceStatus::decode(&out[1..])
//                         .map_err(|e| e.shift(1).map_value(Self::Error::Interface))?;
//                     WithSize {
//                         size: size + 1,
//                         value: Self::Interface(value),
//                     }
//                 }
//             },
//         )
//     }
// }
// #[test]
// fn test_status() {
//     test_item(
//         Status::Action(operand::Status {
//             action_id: 2,
//             status: operand::status_code::UNKNOWN_OPERATION,
//         }),
//         &hex!("22 02 F6"),
//     )
// }

/// Action received before any responses to a request that contained a RequestTag
///
/// This allows matching responses to requests when doing multiple requests in parallel.
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseTag {
    /// End of packet
    ///
    /// Signal the last response packet for the request `id`
    #[deku(bits = 1)]
    pub eop: bool,
    /// An error occured
    #[deku(bits = 1)]
    pub err: bool,
    pub id: u8,
}
// impl_simple_op!(ResponseTag, eop, err, id);
// #[test]
// fn test_response_tag() {
//     test_item(
//         ResponseTag {
//             eop: true,
//             err: false,
//             id: 8,
//         },
//         &hex!("A3 08"),
//     )
// }

// Special
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 2, type = "u8")]
pub enum ChunkStep {
    #[deku(id = "0")]
    Continue,
    #[deku(id = "1")]
    Start,
    #[deku(id = "2")]
    End,
    #[deku(id = "3")]
    StartEnd,
}

/// Provide chunk information and therefore allows to send an ALP command by chunks.
///
/// Specification:
/// An ALP Command may be chunked into multiple Chunks. A special Chunk Action is inserted at the beginning of each
/// ALP Command Chunk to define its chunk state – START, CONTINUE or END (see 6.2.2.1). If the Chunk Action is not
/// present, the ALP Command is not chunked (implicit START/END). The Group (11.5.3) and Break Query conditions are
/// extended over all chunks of the ALP Command.
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub step: ChunkStep,
}

/// Provide logical link of a group of queries
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 2, type = "u8")]
pub enum LogicOp {
    #[deku(id = "0")]
    Or,
    #[deku(id = "1")]
    Xor,
    #[deku(id = "2")]
    Nor,
    #[deku(id = "3")]
    Nand,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct Logic {
    pub logic: LogicOp,
}

/// Forward rest of the command over the interface
// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct Forward {
//     // ALP_SPEC Ask for response ?
//     #[deku(bits=1)]
//     pub response: bool,
//     pub configuration: operand::InterfaceConfiguration,
// }
// impl Codec for Forward {
//     type Error = operand::InterfaceConfigurationDecodingError;
//     fn encoded_size(&self) -> usize {
//         1 + self.conf.encoded_size()
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         out[0] = control_byte!(false, self.resp, OpCode::Forward);
//         1 + self.conf.encode_in(&mut out[1..])
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         let min_size = 1 + 1;
//         if out.len() < min_size {
//             return Err(WithOffset::new(
//                 0,
//                 Self::Error::MissingBytes(min_size - out.len()),
//             ));
//         }
//         let WithSize {
//             value: conf,
//             size: conf_size,
//         } = operand::InterfaceConfiguration::decode(&out[1..]).map_err(|e| e.shift(1))?;
//         Ok(WithSize {
//             value: Self {
//                 resp: out[0] & 0x40 != 0,
//                 conf,
//             },
//             size: 1 + conf_size,
//         })
//     }
// }
// #[test]
// fn test_forward() {
//     test_item(
//         Forward {
//             resp: true,
//             conf: operand::InterfaceConfiguration::Host,
//         },
//         &hex!("72 00"),
//     )
// }

/// Forward rest of the command over the interface
// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct IndirectForward {
//     // ALP_SPEC Ask for response ?
//     #[deku(bits=1)]
//     pub response: bool,
//     pub interface: operand::IndirectInterface,
// }
// impl Codec for IndirectForward {
//     type Error = StdError;
//     fn encoded_size(&self) -> usize {
//         1 + self.interface.encoded_size()
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         let overload = match self.interface {
//             operand::IndirectInterface::Overloaded(_) => true,
//             operand::IndirectInterface::NonOverloaded(_) => false,
//         };
//         out[0] = control_byte!(overload, self.resp, OpCode::IndirectForward);
//         1 + serialize_all!(&mut out[1..], &self.interface)
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.is_empty() {
//             Err(WithOffset::new_head(Self::Error::MissingBytes(1)))
//         } else {
//             let mut offset = 0;
//             let WithSize {
//                 value: op1,
//                 size: op1_size,
//             } = operand::IndirectInterface::decode(out)?;
//             offset += op1_size;
//             Ok(WithSize {
//                 value: Self {
//                     resp: out[0] & 0x40 != 0,
//                     interface: op1,
//                 },
//                 size: offset,
//             })
//         }
//     }
// }
// #[test]
// fn test_indirect_forward() {
//     test_item(
//         IndirectForward {
//             resp: true,
//             interface: operand::IndirectInterface::Overloaded(
//                 operand::OverloadedIndirectInterface {
//                     interface_file_id: 4,
//                     nls_method: dash7::NlsMethod::AesCcm32,
//                     access_class: 0xFF,
//                     address: dash7::Address::Vid([0xAB, 0xCD]),
//                 },
//             ),
//         },
//         &hex!("F3   04   37 FF ABCD"),
//     )
// }

/// Provide command payload identifier
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct RequestTag {
    /// Ask for end of packet
    ///
    /// Signal the last response packet for the request `id`
    #[deku(bits = 1)]
    pub eop: bool,
    pub id: u8,
}
// impl Codec for RequestTag {
//     type Error = StdError;
//     fn encoded_size(&self) -> usize {
//         1 + 1
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         out[0] = control_byte!(self.eop, false, OpCode::RequestTag);
//         out[1] = self.id;
//         1 + 1
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         let min_size = 1 + 1;
//         if out.len() < min_size {
//             return Err(WithOffset::new(
//                 0,
//                 Self::Error::MissingBytes(min_size - out.len()),
//             ));
//         }
//         Ok(WithSize {
//             value: Self {
//                 eop: out[0] & 0x80 != 0,
//                 id: out[1],
//             },
//             size: 2,
//         })
//     }
// }
// #[test]
// fn test_request_tag() {
//     test_item(RequestTag { eop: true, id: 8 }, &hex!("B4 08"))
// }

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct Extension {
    pub header: ActionHeader,
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;
    use crate::test_tools::test_item;

    #[test]
    fn test_header() {
        test_item(
            ActionHeader {
                group: true,
                response: false,
            },
            &[0b1000_0000],
            (&[], 0),
        )
    }

    #[test]
    fn test_nop() {
        test_item(
            Operation(Action::Nop(Nop {
                header: ActionHeader {
                    group: false,
                    response: true,
                },
            })),
            &[0b0100_0000],
            (&[], 0),
        )
    }

    #[test]
    fn test_read_file_data() {
        test_item(
            Operation(Action::ReadFileData(ReadFileData {
                header: ActionHeader {
                    group: false,
                    response: true,
                },
                offset: FileOffset {
                    file_id: 1,
                    offset: 2u32.into(),
                },
                length: 3u32.into(),
            })),
            &hex!("41 01 02 03"),
            (&[], 0),
        )
    }

    #[test]
    fn test_read_file_properties() {
        test_item(
            Operation(Action::ReadFileProperties(ReadFileProperties {
                header: ActionHeader {
                    group: false,
                    response: false,
                },
                file_id: 9,
            })),
            &hex!("02 09"),
            (&[], 0),
        )
    }

        #[test]
    fn test_write_file_data() {
        let data = hex!("01 02 03").to_vec();
        test_item(
            Operation(Action::WriteFileData(WriteFileData {
                header: ActionHeader { group: true, response: false },
                offset: FileOffset { file_id: 9, offset: 5u32.into() },
                length: data.len().into(),
                data,
            })),
            &hex!("84 09 05 03 010203"),
            (&[], 0)
        )
    }


    #[test]
    fn test_logic() {
        test_item(
            Logic {
                logic: LogicOp::Nand,
            },
            &[0b1100_0000],
            (&[0b1100_0000], 2),
        )
    }

    #[test]
    fn test_chunk() {
        test_item(
            Chunk {
                step: ChunkStep::End,
            },
            &[0b1000_0000],
            (&[0b1000_0000], 2),
        )
    }
}
