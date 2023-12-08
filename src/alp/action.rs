#[cfg(feature="std")]
use std::fmt;

#[cfg(not(feature="std"))]
use alloc::fmt;

use bitvec::field::BitField;
use deku::{
    bitvec::{BitSlice, BitVec, BitView, Msb0},
    ctx::{BitSize, Endian},
    prelude::*,
};

use super::{
    data::FileHeader,
    interface::{InterfaceConfiguration, InterfaceType},
    operand::{ActionStatus, FileOffset, Length, Permission, PermissionLevel},
    query::Query,
    session::InterfaceStatus,
};

// ===============================================================================
// OpCodes
// ===============================================================================

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
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

    // Special
    Chunk(Chunk),
    Logic(Logic),
    Forward(Forward),
    IndirectForward(IndirectForward),
    RequestTag(RequestTag),
    Extension(Extension),
}

macro_rules! read_action {
    ($action: ident, $operand: ty, $input: ident) => {{
        <$operand as DekuRead<'_, _>>::read($input, ()).map(|(rest, action)| (rest, Self::$action(action)))?
    }};
}

impl DekuRead<'_, ()> for Action {
    fn read(
        input: &'_ BitSlice<u8, Msb0>,
        _: (),
    ) -> Result<(&'_ BitSlice<u8, Msb0>, Self), DekuError> {
        println!("bitslice length {:?} {:?}", input, input.len());

        let (rest, _) = <u8 as DekuRead<'_, _>>::read(input, (Endian::Big, BitSize(2)))?;
        let (_, code) = <OpCode as DekuRead<'_, _>>::read(rest, ())?;

        let (rest, value) = match code {
            OpCode::Nop => read_action!(Nop, Nop, input),
            OpCode::ReadFileData => read_action!(ReadFileData, ReadFileData, input),
            OpCode::ReadFileProperties => read_action!(ReadFileProperties, FileId, input),
            OpCode::WriteFileData => read_action!(WriteFileData, FileData, input),
            OpCode::WriteFileDataFlush => read_action!(WriteFileDataFlush, FileData, input),
            OpCode::WriteFileProperties => read_action!(WriteFileProperties, FileProperties, input),
            OpCode::ActionQuery => read_action!(ActionQuery, ActionQuery, input),
            OpCode::BreakQuery => read_action!(BreakQuery, ActionQuery, input),
            OpCode::PermissionRequest => read_action!(PermissionRequest, PermissionRequest, input),
            OpCode::VerifyChecksum => read_action!(VerifyChecksum, ActionQuery, input),
            OpCode::ExistFile => read_action!(ExistFile, FileId, input),
            OpCode::CreateNewFile => read_action!(CreateNewFile, FileProperties, input),
            OpCode::DeleteFile => read_action!(DeleteFile, FileId, input),
            OpCode::RestoreFile => read_action!(RestoreFile, FileId, input),
            OpCode::FlushFile => read_action!(FlushFile, FileId, input),
            OpCode::CopyFile => read_action!(CopyFile, CopyFile, input),
            OpCode::ExecuteFile => read_action!(ExecuteFile, FileId, input),
            OpCode::ReturnFileData => read_action!(ReturnFileData, FileData, input),
            OpCode::ReturnFileProperties => {
                read_action!(ReturnFileProperties, FileProperties, input)
            }
            OpCode::ResponseTag => read_action!(ResponseTag, ResponseTag, input),
            OpCode::Chunk => read_action!(Chunk, Chunk, input),
            OpCode::Logic => read_action!(Logic, Logic, input),
            OpCode::RequestTag => read_action!(RequestTag, RequestTag, input),
            OpCode::Status => read_action!(Status, StatusOperand, input),
            OpCode::Forward => read_action!(Forward, Forward, input),
            OpCode::IndirectForward => read_action!(IndirectForward, IndirectForward, input),
            OpCode::Extension => read_action!(Extension, Extension, input),
        };
        Ok((rest, value))
    }
}

impl TryFrom<&'_ [u8]> for Action {
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

fn pad_rest<'a>(input_bits: &'a BitSlice<u8,Msb0>, rest: &'a BitSlice<u8,Msb0>)  -> (&'a [u8], usize){
    let pad = 8 * ((rest.len() + 7) / 8) - rest.len();
    let read_idx = input_bits.len() - (rest.len() + pad);
        (input_bits[read_idx..].domain().region().unwrap().1, pad)
}

impl DekuContainerRead<'_> for Action {
    fn from_bytes(input: (&'_ [u8], usize)) -> Result<((&'_ [u8], usize), Self), DekuError> {
        let input_bits = input.0.view_bits::<Msb0>();
        let (rest, value) = <Self as DekuRead>::read(&input_bits[input.1..], ())?;

        Ok((
            pad_rest(input_bits,rest),
            value,
        ))
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

macro_rules! write_action {
    ($action: ident, $output: ident) => {{
        DekuWrite::write($action, $output, ())?;
    }};
}

impl DekuWrite<()> for Action {
    fn write(&self, output: &mut BitVec<u8, Msb0>, _: ()) -> Result<(), DekuError> {
        println!("output {:?} {:?}", output, output.len());
        let offset = output.len();
        match self {
            Action::Nop(action) => write_action!(action, output),
            Action::ReadFileData(action) => write_action!(action, output),
            Action::ReadFileProperties(action) => write_action!(action, output),
            Action::WriteFileData(action) => write_action!(action, output),
            Action::WriteFileDataFlush(action) => write_action!(action, output),
            Action::WriteFileProperties(action) => write_action!(action, output),
            Action::ActionQuery(action) => write_action!(action, output),
            Action::BreakQuery(action) => write_action!(action, output),
            Action::PermissionRequest(action) => write_action!(action, output),
            Action::VerifyChecksum(action) => write_action!(action, output),
            Action::ExistFile(action) => write_action!(action, output),
            Action::CreateNewFile(action) => write_action!(action, output),
            Action::DeleteFile(action) => write_action!(action, output),
            Action::RestoreFile(action) => write_action!(action, output),
            Action::FlushFile(action) => write_action!(action, output),
            Action::CopyFile(action) => write_action!(action, output),
            Action::ExecuteFile(action) => write_action!(action, output),
            Action::ReturnFileData(action) => write_action!(action, output),
            Action::ReturnFileProperties(action) => write_action!(action, output),
            Action::ResponseTag(action) => write_action!(action, output),
            Action::Chunk(action) => write_action!(action, output),
            Action::Logic(action) => write_action!(action, output),
            Action::Status(action) => write_action!(action, output),
            Action::Forward(action) => write_action!(action, output),
            Action::IndirectForward(action) => write_action!(action, output),
            Action::RequestTag(action) => write_action!(action, output),
            Action::Extension(action) => write_action!(action, output),
        }

        // now write the opcode with offset 2
        let code = self.deku_id()?.deku_id()? as u8;
        println!("code {:?}", code);
        output[offset+2..offset+8].store_be(code);
        Ok(())
    }
}

impl TryFrom<Action> for Vec<u8> {
    type Error = DekuError;
    fn try_from(input: Action) -> Result<Self, Self::Error> {
        DekuContainerWrite::to_bytes(&input)
    }
}
impl DekuContainerWrite for Action {
    fn to_bytes(&self) -> Result<Vec<u8>, DekuError> {
        let output = self.to_bits()?;
        Ok(output.into_vec())
    }

    fn to_bits(&self) -> Result<BitVec<u8, Msb0>, DekuError> {
        let mut output: BitVec<u8, Msb0> = BitVec::new();
        self.write(&mut output, ())?;
        Ok(output)
    }
}

/// File access type event that will trigger an ALP action.
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
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

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct ActionHeader {
    /// Group with next action
    #[deku(bits = 1)]
    pub group: bool,
    /// Ask for a response (status)
    #[deku(bits = 1, pad_bits_after = "6")]
    pub response: bool,
    //OpCode would be here. 6 bits padding instead
}

// ===============================================================================
// Actions
// ===============================================================================
// Nop
/// Does nothing
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct Nop {
    pub header: ActionHeader,
}

/// Checks whether a file exists
// ALP_SPEC: How is the result of this command different from a read file of size 0?
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct FileId {
    pub header: ActionHeader,
    pub file_id: u8,
}

// Write data to a file
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct FileData {
    pub header: ActionHeader,

    pub offset: FileOffset,

    #[deku(update = "self.data.len()")]
    length: Length,

    #[deku(count = "length", endian = "big")]
    data: Vec<u8>,
}

impl FileData {
    pub fn new(header: ActionHeader, offset: FileOffset, data: Vec<u8>) -> Self {
        Self {
            header,
            offset,
            length: data.len().into(),
            data,
        }
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.length = data.len().into();
        self.data = data;
    }
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct FileProperties {
    pub header: ActionHeader,

    pub file_id: u8,
    pub file_header: FileHeader,
}

// Read
/// Read data from a file
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct ReadFileData {
    pub header: ActionHeader,

    pub offset: FileOffset,
    pub length: Length,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct ActionQuery {
    pub header: ActionHeader,

    pub query: Query,
}

/// Request a level of permission using some permission type
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct PermissionRequest {
    pub header: ActionHeader,
    /// See operand::permission_level
    pub level: PermissionLevel,
    pub permission: Permission,
}

/// Copy a file to another file
// ALP_SPEC: What does that mean? Is it a complete file copy including the file properties or just
// the data? If not then if the destination file is bigger than the source, does the copy only
// overwrite the first part of the destination file?
//
// Wouldn't it be more appropriate to have 1 size and 2 file offsets?
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct CopyFile {
    pub header: ActionHeader,
    pub src_file_id: u8,
    pub dst_file_id: u8,
}

#[derive(DekuRead, DekuWrite, Clone, Copy, Debug, PartialEq)]
#[deku(bits = 2, type = "u8")]

pub enum StatusType {
    #[deku(id = "0")]
    Action,
    #[deku(id = "1")]
    Interface,
}

/// Forward rest of the command over the interface
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct StatusOperand {
    #[deku(update = "self.status.deku_id().unwrap()", pad_bits_after = "6")]
    status_type: StatusType,

    #[deku(ctx = "*status_type")]
    pub status: Status,
}

impl Into<StatusOperand> for Status {
    fn into(self) -> StatusOperand {
        StatusOperand {
            status_type: self.deku_id().unwrap(),
            status: self,
        }
    }
}

impl Into<Status> for StatusOperand {
    fn into(self) -> Status {
        self.status
    }
}

/// Statuses regarding actions sent in a request
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "status_type: StatusType", id = "status_type")]
pub enum Status {
    // ALP SPEC: This is named status, but it should be named action status compared to the '2'
    // other statuses.
    #[deku(id = "StatusType::Action")]
    Action(ActionStatus),
    #[deku(id = "StatusType::Interface")]
    Interface(InterfaceStatus),
    // ALP SPEC: Where are the stack errors?
}

/// Action received before any responses to a request that contained a RequestTag
///
/// This allows matching responses to requests when doing multiple requests in parallel.
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct ResponseTag {
    /// End of packet
    ///
    /// Signal the last response packet for the request `id`
    #[deku(bits = 1)]
    pub eop: bool,
    /// An error occured
    #[deku(bits = 1, pad_bits_after = "6")]
    pub error: bool,

    pub id: u8,
}

// Special
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
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
/// ALP Command Chunk to define its chunk state: START, CONTINUE or END (see 6.2.2.1). If the Chunk Action is not
/// present, the ALP Command is not chunked (implicit START/END). The Group (11.5.3) and Break Query conditions are
/// extended over all chunks of the ALP Command.
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct Chunk {
    #[deku(pad_bits_after = "6")]
    pub step: ChunkStep,
}

/// Provide logical link of a group of queries
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
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

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct Logic {
    #[deku(pad_bits_after = "6")]
    pub logic: LogicOp,
}

/// Forward rest of the command over the interface
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct Forward {
    // ALP_SPEC Ask for response ?
    #[deku(bits = 1, pad_bits_before = "1", pad_bits_after = "6")]
    pub response: bool,

    #[deku(update = "self.configuration.deku_id().unwrap()")]
    interface_type: InterfaceType,

    #[deku(ctx = "*interface_type")]
    pub configuration: InterfaceConfiguration,
}

impl Forward {
    pub fn new(response: bool, configuration: InterfaceConfiguration) -> Self {
        Self {
            response,
            interface_type: configuration.deku_id().unwrap(),
            configuration,
        }
    }
}

/// Forward rest of the command over the interface
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct IndirectForward {
    #[deku(bits = 1, update = "self.configuration.is_some()")]
    overloaded: bool,

    #[deku(bits = 1, pad_bits_after = "6")]
    pub response: bool,

    pub interface_file_id: u8,

    #[deku(
        reader = "IndirectForward::read(deku::rest, *overloaded)",
        writer = "IndirectForward::write(deku::output, &self.configuration)"
    )]
    pub configuration: Option<InterfaceConfiguration>,
}

impl IndirectForward {
    pub fn new(
        response: bool,
        interface_file_id: u8,
        configuration: Option<InterfaceConfiguration>,
    ) -> Self {
        Self {
            overloaded: configuration.is_some(),
            response,
            interface_file_id,
            configuration,
        }
    }

    fn read(
        rest: &BitSlice<u8, Msb0>,
        overloaded: bool,
    ) -> Result<(&BitSlice<u8, Msb0>, Option<InterfaceConfiguration>), DekuError> {
        // ALP_SPEC: The first byte in the interface_file defines how to parse the
        // configuration overload, or even its byte size.
        // We can not continue parsing here!

        let config = if !overloaded {
            None
        } else {
            Some(InterfaceConfiguration::Unknown)
        };

        Ok((rest, config))
    }

    fn write(
        output: &mut BitVec<u8, Msb0>,
        configuration: &Option<InterfaceConfiguration>,
    ) -> Result<(), DekuError> {
        if let Some(config) = configuration.as_ref() {
            DekuWrite::write(config, output, config.deku_id().unwrap())?;
        }
        Ok(())
    }
}

/// Provide command payload identifier
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct RequestTag {
    /// Ask for end of packet
    ///
    /// Signal the last response packet for the request `id`
    #[deku(bits = 1, pad_bits_after = "7")]
    pub eop: bool,

    pub id: u8,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct Extension {
    pub header: ActionHeader,
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;
    use crate::{
        alp::{
            data::{self, FilePermissions, UserPermissions},
            interface::{Dash7InterfaceConfiguration, GroupCondition},
            network::{Address, Addressee, NlsState},
            operand::{PermissionLevel, StatusCode},
            query::NonVoid,
            session::QoS,
            varint::VarInt,
        },
        test_tools::test_item,
    };

    #[test]
    fn test_header() {
        test_item(
            ActionHeader {
                group: true,
                response: false,
            },
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
                    file_id: 9,
                    offset: 5u32.into(),
                },
                data,
            )),
            &hex!("84 09 05 03 010203"),
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
                    file_id: 9,
                    offset: 5u32.into(),
                },
                data,
            )),
            &hex!("20 09 05 03 010203"),
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
            }),
            &hex!("C8   00 04  05 06"),
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
            }),
            &hex!("C9   00 04  05 06"),
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
            }),
            &hex!("0B   00 04  05 06"),
        )
    }

    #[test]
    fn test_forward() {
        test_item(
            Action::Forward(Forward::new(true, InterfaceConfiguration::Host)),
            &hex!("72 00"),
        )
    }

    #[cfg(not(feature = "subiot_v0"))]
    #[test]
    fn test_indirect_forward_dash7_serialization() {
        let item = Action::IndirectForward(IndirectForward::new(
            true,
            9,
            Some(InterfaceConfiguration::Dash7(Dash7InterfaceConfiguration {
                qos: QoS::default(),
                dormant_session_timeout: VarInt::default(),
                te: VarInt::default(),
                addressee: Addressee::new(
                    false,
                    GroupCondition::Any,
                    Address::Vid(0xABCD),
                    NlsState::AesCcm32([1, 2, 3, 4, 5]),
                    0xFF,
                ),
            })),
        ));

        let data = &hex!("F3 09 00 00 00 37 FF ABCD 01 02 03 04 05");
        let result = item.to_bytes().unwrap();

        assert_eq!(result.as_slice(), data, "{:?} == {:?}", &item, data);
    }

    #[cfg(not(feature = "subiot_v0"))]
    #[test]
    fn test_indirect_forward_dash7_deserialization() {
        let input = &hex!("F3 09 00 00 00 37 FF ABCD 01 02 03 04 05");

        let expected = Action::IndirectForward(IndirectForward::new(
            true,
            9,
            Some(InterfaceConfiguration::Unknown),
        ));

        let ((rest, offset), result) =
            Action::from_bytes((input, 0)).expect("should be parsed without error");

        assert_eq!(result, expected.clone(), "{:?} == {:?}", result, &expected);

        let expected_config = Dash7InterfaceConfiguration {
            qos: QoS::default(),
            dormant_session_timeout: VarInt::default(),
            te: VarInt::default(),
            addressee: Addressee::new(
                false,
                GroupCondition::Any,
                Address::Vid(0xABCD),
                NlsState::AesCcm32([1, 2, 3, 4, 5]),
                0xFF,
            ),
        };

        // now continue parsing the config itself
        let (_, config_result) = Dash7InterfaceConfiguration::from_bytes((rest, offset))
            .expect("should be parsed without error");

        assert_eq!(
            config_result,
            expected_config.clone(),
            "{:?} == {:?}",
            config_result,
            &expected_config
        );
    }

    #[cfg(feature = "subiot_v0")]
    #[test]
    fn test_indirect_forward_dash7_serialization_subiot() {
        let item = Action::IndirectForward(IndirectForward::new(
            true,
            9,
            Some(InterfaceConfiguration::Dash7(Dash7InterfaceConfiguration {
                qos: QoS::default(),
                dormant_session_timeout: VarInt::default(),
                te: VarInt::default(),
                addressee: Addressee::new(
                    false,
                    GroupCondition::Any,
                    Address::Vid(0xABCD),
                    NlsState::AesCcm32([1, 2, 3, 4, 5]),
                    0xFF,
                ),
            })),
        ));

        let data = &hex!("F3 09 00 00 37 FF ABCD 01 02 03 04 05");
        let result = item.to_bytes().unwrap();

        assert_eq!(result.as_slice(), data, "{:?} == {:?}", &item, data);
    }

    #[cfg(feature = "subiot_v0")]
    #[test]
    fn test_indirect_forward_dash7_deserialization_subiot() {
        let input = &hex!("F3 09 00 00 37 FF ABCD 01 02 03 04 05");

        let expected = Action::IndirectForward(IndirectForward::new(
            true,
            9,
            Some(InterfaceConfiguration::Unknown),
        ));

        let ((rest, offset), result) =
            Action::from_bytes((input, 0)).expect("should be parsed without error");

        assert_eq!(result, expected.clone(), "{:?} == {:?}", result, &expected);

        let expected_config = Dash7InterfaceConfiguration {
            qos: QoS::default(),
            dormant_session_timeout: VarInt::default(),
            te: VarInt::default(),
            addressee: Addressee::new(
                false,
                GroupCondition::Any,
                Address::Vid(0xABCD),
                NlsState::AesCcm32([1, 2, 3, 4, 5]),
                0xFF,
            ),
        };

        // now continue parsing the config itself
        let (_, config_result) = Dash7InterfaceConfiguration::from_bytes((rest, offset))
            .expect("should be parsed without error");

        assert_eq!(
            config_result,
            expected_config.clone(),
            "{:?} == {:?}",
            config_result,
            &expected_config
        );
    }

    #[test]
    fn test_request_tag() {
        test_item(
            Action::RequestTag(RequestTag { eop: true, id: 8 }),
            &hex!("B4 08"),
        )
    }

    #[test]
    fn test_logic() {
        test_item(
            Action::Logic(Logic {
                logic: LogicOp::Nand,
            }),
            &[0b1111_0001],
        )
    }

    #[test]
    fn test_chunk() {
        test_item(
            Action::Chunk(Chunk {
                step: ChunkStep::End,
            }),
            &[0b1011_0000],
        )
    }

    #[test]
    fn test_response_tag() {
        test_item(
            Action::ResponseTag(ResponseTag {
                eop: true,
                error: false,
                id: 8,
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
            }),
            &[0xFF],
        )
    }
}
