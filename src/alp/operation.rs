use std::fmt;

use bitvec::{field::BitField, view::BitView};
use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    ctx::{BitSize, Endian},
    prelude::*,
};

use super::{
    data::FileHeader,
    interface::{IndirectInterface, InterfaceConfigurationOverload, InterfaceType},
    operand::{self, FileOffset, Length, Permission, PermissionLevel},
    query,
};

// ===============================================================================
// OpCodes
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

// #[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
// #[deku(ctx = "code: OpCode", id = "code")]
pub enum Operation {
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
    // #[deku(id = "34")]
    // Status(Status),
    ResponseTag(ResponseTag),

    // Special
    Chunk(Chunk),
    Logic(Logic),
    // #[deku(id = "50")]
    // Forward(Forward),
    IndirectForward(IndirectForward),
    RequestTag(RequestTag),
    Extension(Extension),
}

impl DekuRead<'_, ()> for Operation {
    fn read(
        input: &'_ BitSlice<u8, Msb0>,
        _: (),
    ) -> Result<(&'_ BitSlice<u8, Msb0>, Self), DekuError> {
        let (rest, _) = <u8 as DekuRead<'_, _>>::read(input, (Endian::Big, BitSize(2)))?;
        let (_, code) = <OpCode as DekuRead<'_, _>>::read(rest, ())?;

        let (rest, value) = match code {
            OpCode::Nop => {
                let (rest, action) = <Nop as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::Nop(action))
            }
            OpCode::ReadFileData => {
                let (rest, action) = <ReadFileData as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::ReadFileData(action))
            }
            OpCode::ReadFileProperties => {
                let (rest, action) = <FileId as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::ReadFileProperties(action))
            }
            OpCode::WriteFileData => {
                let (rest, action) = <FileData as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::WriteFileData(action))
            }
            OpCode::WriteFileDataFlush => {
                let (rest, action) = <FileData as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::WriteFileDataFlush(action))
            }
            OpCode::WriteFileProperties => {
                let (rest, action) = <FileProperties as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::WriteFileProperties(action))
            }
            OpCode::ActionQuery => {
                let (rest, action) = <ActionQuery as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::ActionQuery(action))
            }
            OpCode::BreakQuery => {
                let (rest, action) = <ActionQuery as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::BreakQuery(action))
            }
            OpCode::PermissionRequest => {
                let (rest, action) = <PermissionRequest as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::PermissionRequest(action))
            }
            OpCode::VerifyChecksum => {
                let (rest, action) = <ActionQuery as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::VerifyChecksum(action))
            }
            OpCode::ExistFile => {
                let (rest, action) = <FileId as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::ExistFile(action))
            }
            OpCode::CreateNewFile => {
                let (rest, action) = <FileProperties as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::CreateNewFile(action))
            }
            OpCode::DeleteFile => {
                let (rest, action) = <FileId as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::DeleteFile(action))
            }
            OpCode::RestoreFile => {
                let (rest, action) = <FileId as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::RestoreFile(action))
            }
            OpCode::FlushFile => {
                let (rest, action) = <FileId as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::FlushFile(action))
            }
            OpCode::CopyFile => {
                let (rest, action) = <CopyFile as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::CopyFile(action))
            }
            OpCode::ExecuteFile => {
                let (rest, action) = <FileId as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::ExecuteFile(action))
            }
            OpCode::ReturnFileData => {
                let (rest, action) = <FileData as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::ReturnFileData(action))
            }
            OpCode::ReturnFileProperties => {
                let (rest, action) = <FileProperties as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::ReturnFileProperties(action))
            }
            OpCode::ResponseTag => {
                let (rest, action) = <ResponseTag as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::ResponseTag(action))
            }
            OpCode::Chunk => {
                let (rest, action) = <Chunk as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::Chunk(action))
            }
            OpCode::Logic => {
                let (rest, action) = <Logic as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::Logic(action))
            }
            OpCode::RequestTag => {
                let (rest, action) = <RequestTag as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::RequestTag(action))
            }
            OpCode::Extension => {
                let (rest, action) = <Extension as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::Extension(action))
            }
            OpCode::Status => todo!(),
            OpCode::Forward => todo!(),
            OpCode::IndirectForward => {
                let (rest, action) = <IndirectForward as DekuRead<'_, _>>::read(input, ())?;
                (rest, Self::IndirectForward(action))
            }
        };
        Ok((rest, value))
    }
}

impl TryFrom<&'_ [u8]> for Operation {
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
impl DekuContainerRead<'_> for Operation {
    fn from_bytes(input: (&'_ [u8], usize)) -> Result<((&'_ [u8], usize), Self), DekuError> {
        let input_bits = input.0.view_bits::<Msb0>();
        let (rest, value) = <Self as DekuRead>::read(&input_bits[input.1..], ())?;

        let pad = 8 * ((rest.len() + 7) / 8) - rest.len();
        let read_idx = input_bits.len() - (rest.len() + pad);
        Ok((
            (input_bits[read_idx..].domain().region().unwrap().1, pad),
            value,
        ))
    }
}

impl DekuEnumExt<'_, OpCode> for Operation {
    fn deku_id(&self) -> Result<OpCode, DekuError> {
        match self {
            Operation::Nop(_) => Ok(OpCode::Nop),
            Operation::ReadFileData(_) => Ok(OpCode::ReadFileData),
            Operation::ReadFileProperties(_) => Ok(OpCode::ReadFileProperties),
            Operation::WriteFileData(_) => Ok(OpCode::WriteFileData),
            Operation::WriteFileDataFlush(_) => Ok(OpCode::WriteFileDataFlush),
            Operation::WriteFileProperties(_) => Ok(OpCode::WriteFileProperties),
            Operation::ActionQuery(_) => Ok(OpCode::ActionQuery),
            Operation::BreakQuery(_) => Ok(OpCode::BreakQuery),
            Operation::PermissionRequest(_) => Ok(OpCode::PermissionRequest),
            Operation::VerifyChecksum(_) => Ok(OpCode::VerifyChecksum),
            Operation::ExistFile(_) => Ok(OpCode::ExistFile),
            Operation::CreateNewFile(_) => Ok(OpCode::CreateNewFile),
            Operation::DeleteFile(_) => Ok(OpCode::DeleteFile),
            Operation::RestoreFile(_) => Ok(OpCode::RestoreFile),
            Operation::FlushFile(_) => Ok(OpCode::FlushFile),
            Operation::CopyFile(_) => Ok(OpCode::CopyFile),
            Operation::ExecuteFile(_) => Ok(OpCode::ExecuteFile),
            Operation::ReturnFileData(_) => Ok(OpCode::ReturnFileData),
            Operation::ReturnFileProperties(_) => Ok(OpCode::ReturnFileProperties),
            Operation::ResponseTag(_) => Ok(OpCode::ResponseTag),
            Operation::Chunk(_) => Ok(OpCode::Chunk),
            Operation::Logic(_) => Ok(OpCode::Logic),
            Operation::IndirectForward(_) => Ok(OpCode::IndirectForward),
            Operation::RequestTag(_) => Ok(OpCode::RequestTag),
            Operation::Extension(_) => Ok(OpCode::Extension),
        }
    }
}

impl DekuUpdate for Operation {
    fn update(&mut self) -> Result<(), DekuError> {
        Ok(())
    }
}

impl DekuWrite<()> for Operation {
    fn write(&self, output: &mut BitVec<u8, Msb0>, _: ()) -> Result<(), DekuError> {
        match self {
            Operation::Nop(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::ReadFileData(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::ReadFileProperties(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::WriteFileData(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::WriteFileDataFlush(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::WriteFileProperties(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::ActionQuery(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::BreakQuery(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::PermissionRequest(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::VerifyChecksum(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::ExistFile(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::CreateNewFile(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::DeleteFile(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::RestoreFile(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::FlushFile(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::CopyFile(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::ExecuteFile(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::ReturnFileData(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::ReturnFileProperties(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::ResponseTag(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::Chunk(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::Logic(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::IndirectForward(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::RequestTag(action) => {
                DekuWrite::write(action, output, ())?;
            }
            Operation::Extension(action) => {
                DekuWrite::write(action, output, ())?;
            }
        }

        // now write the opcode with offset 2
        let code = self.deku_id()?.deku_id()? as u8;
        output[2..8].store_be(code);
        Ok(())
    }
}

impl TryFrom<Operation> for Vec<u8> {
    type Error = DekuError;
    fn try_from(input: Operation) -> Result<Self, Self::Error> {
        DekuContainerWrite::to_bytes(&input)
    }
}
impl DekuContainerWrite for Operation {
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
    //OpCode would be here. 6 bits padding instead
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
pub struct FileId {
    pub header: ActionHeader,
    pub file_id: u8,
}

// Write data to a file
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct FileData {
    pub header: ActionHeader,
    pub operand: operand::FileData,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct FileProperties {
    pub header: ActionHeader,

    pub file_id: u8,
    pub file_header: FileHeader,
}

// Read
/// Read data from a file
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ReadFileData {
    pub header: ActionHeader,

    pub offset: FileOffset,
    pub length: Length,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ActionQuery {
    pub header: ActionHeader,

    pub query: query::Query,
}

/// Request a level of permission using some permission type
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
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
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct CopyFile {
    pub header: ActionHeader,
    pub src_file_id: u8,
    pub dst_file_id: u8,
}

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
    #[deku(bits = 1, pad_bits_after = "6")]
    pub error: bool,

    pub id: u8,
}

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
/// ALP Command Chunk to define its chunk state: START, CONTINUE or END (see 6.2.2.1). If the Chunk Action is not
/// present, the ALP Command is not chunked (implicit START/END). The Group (11.5.3) and Break Query conditions are
/// extended over all chunks of the ALP Command.
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    #[deku(pad_bits_after = "6")]
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
    #[deku(pad_bits_after = "6")]
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
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct IndirectForward {
    // ALP_SPEC Ask for response ?
    #[deku(bits = 1, update = "self.interface.deku_id().unwrap()")]
    overloaded: bool,

    #[deku(bits = 1, pad_bits_after = "6")]
    pub response: bool,

    #[deku(ctx = "*overloaded")]
    pub interface: IndirectInterface,
}

impl IndirectForward {
    pub fn new(response: bool, interface: IndirectInterface) -> Self {
        Self {
            overloaded: interface.deku_id().unwrap(),
            response,
            interface,
        }
    }
}

/// Provide command payload identifier
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct RequestTag {
    /// Ask for end of packet
    ///
    /// Signal the last response packet for the request `id`
    #[deku(bits = 1, pad_bits_after = "7")]
    pub eop: bool,
    pub id: u8,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
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
            network::{Address, Addressee, NlsMethod, NlsState},
            operand::PermissionLevel,
            query::NonVoid,
            session::{Dash7InterfaceConfiguration, QoS},
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
            Operation::Nop(Nop {
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
            Operation::ReadFileData(ReadFileData {
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
            Operation::ReadFileProperties(FileId {
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
            Operation::WriteFileData(FileData {
                header: ActionHeader {
                    group: true,
                    response: false,
                },
                operand: operand::FileData::new(
                    FileOffset {
                        file_id: 9,
                        offset: 5u32.into(),
                    },
                    data,
                ),
            }),
            &hex!("84 09 05 03 010203"),
        )
    }

    #[test]
    fn test_return_file_properties() {
        test_item(
            Operation::ReturnFileProperties(FileProperties {
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
            Operation::WriteFileProperties(FileProperties {
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
            Operation::PermissionRequest(PermissionRequest {
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
            Operation::ExistFile(FileId {
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
            Operation::CreateNewFile(FileProperties {
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
            Operation::DeleteFile(FileId {
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
            Operation::RestoreFile(FileId {
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
            Operation::FlushFile(FileId {
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
            Operation::CopyFile(CopyFile {
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
            Operation::ExecuteFile(FileId {
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
            Operation::ReturnFileData(FileData {
                header: ActionHeader {
                    group: false,
                    response: false,
                },
                operand: operand::FileData::new(
                    FileOffset {
                        file_id: 9,
                        offset: 5u32.into(),
                    },
                    data,
                ),
            }),
            &hex!("20 09 05 03 010203"),
        )
    }

    #[test]
    fn test_action_query() {
        test_item(
            Operation::ActionQuery(ActionQuery {
                header: ActionHeader {
                    group: true,
                    response: true,
                },
                query: query::Query::NonVoid(NonVoid {
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
            Operation::BreakQuery(ActionQuery {
                header: ActionHeader {
                    group: true,
                    response: true,
                },
                query: query::Query::NonVoid(NonVoid {
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
            Operation::VerifyChecksum(ActionQuery {
                header: ActionHeader {
                    group: false,
                    response: false,
                },
                query: query::Query::NonVoid(NonVoid {
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
    fn test_indirect_forward_dash7() {
        test_item(
            Operation::IndirectForward(IndirectForward::new(
                true,
                IndirectInterface::Overloaded(InterfaceConfigurationOverload::Dash7(
                    Dash7InterfaceConfiguration {
                        qos: QoS::default(),
                        dormant_session_timeout: VarInt::default(),
                        addressee: Addressee::new(
                            Address::Vid(0xABCD),
                            NlsState::AesCcm32([1, 2, 3, 4, 5]),
                            0xFF,
                        ),
                    },
                )),
            )),
            &hex!("F3 D7 00 00 37 FF ABCD 01 02 03 04 05"),
        )
    }

    #[test]
    fn test_request_tag() {
        test_item(
            Operation::RequestTag(RequestTag { eop: true, id: 8 }),
            &hex!("B4 08"),
        )
    }

    #[test]
    fn test_logic() {
        test_item(
            Operation::Logic(Logic {
                logic: LogicOp::Nand,
            }),
            &[0b1111_0001],
        )
    }

    #[test]
    fn test_chunk() {
        test_item(
            Operation::Chunk(Chunk {
                step: ChunkStep::End,
            }),
            &[0b1011_0000],
        )
    }

    #[test]
    fn test_response_tag() {
        test_item(
            Operation::ResponseTag(ResponseTag {
                eop: true,
                error: false,
                id: 8,
            }),
            &hex!("A3 08"),
        )
    }

    #[test]
    fn test_extension() {
        test_item(
            Operation::Extension(Extension {
                header: ActionHeader {
                    group: true,
                    response: true,
                },
            }),
            &[0xFF],
        )
    }
}
