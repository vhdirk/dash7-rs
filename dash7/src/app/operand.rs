pub use super::query::Query;
use super::{
    operation::OpCode,
    interface::{IndirectInterface, InterfaceConfiguration},
};
use crate::{data::FileHeader, file::{FileCtx, SystemFile}, session::InterfaceStatus};
use crate::{file::FileData, types::Length, utils::write_length_prefixed};
use crate::{session::InterfaceType, utils::write_length_prefixed_ext};
use deku::{no_std_io, prelude::*};

#[cfg(feature = "_wizzilab")]
pub use super::interface_final::*;

// ===============================================================================
// Operations
// ===============================================================================

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
#[deku(id_type = "u8")]
pub enum StatusCode {
    /// Status code that can be received as a result of some ALP actions.
    /// Action received and partially completed at response. To be completed after response
    #[default]
    #[deku(id = "0x00")]
    Ok,
    #[deku(id = "0x01")]
    Received,
    #[deku(id = "0x02")]
    ItfFull,

    #[deku(id_pat = "0x80..0xF4")]
    UnknownError(u8),

    #[deku(id = "0xF4")]
    OperationWrongFormat,
    #[deku(id = "0xF5")]
    OperationIncomplete,
    #[deku(id = "0xF6")]
    UnknownOperation,
    #[deku(id = "0xF7")]
    WriteStorageUnavailable,
    #[deku(id = "0xF8")]
    WriteDataOverflow,
    #[deku(id = "0xF9")]
    WriteOffsetOverflow,
    #[deku(id = "0xFA")]
    CreateFileAllocationOverflow, // ALP_SPEC: ??? Difference with the previous one?;
    #[deku(id = "0xFB")]
    CreateFileLengthOverflow,
    #[deku(id = "0xFC")]
    InsufficientPermission,
    #[deku(id = "0xFD")]
    FileIsNotRestorable,
    #[deku(id = "0xFE")]
    CreateFileIdAlreadyExist,
    #[deku(id = "0xFF")]
    FileIdMissing,

    #[deku(id_pat = "_")]
    Other(u8),
}

impl StatusCode {
    pub fn is_err(&self) -> bool {
        self.deku_id().unwrap_or(0) > 0x80
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct FileOffset {
    pub file_id: u8,
    pub offset: Length,
}

/// Result of an action in a previously sent request
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct ActionStatus {
    /// Index of the ALP action associated with this status, in the original request as seen from
    /// the receiver side.
    // ALP_SPEC This is complicated to process because we have to known/possibly infer the position
    // of the action on the receiver side, and that we have to do that while also interpreting who
    // responded (the local modem won't have the same index as the distant device.).
    pub action_id: u8,
    /// Result code
    pub status: StatusCode,
}

// ALP SPEC: where is this defined? Link? Not found in either specs !
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
#[deku(id_type = "u8", endian = "big")]
pub enum Permission {
    #[deku(id = "0x42")] // ALP_SPEC Undefined
    Dash7(u64),
}

impl Default for Permission {
    fn default() -> Self {
        Self::Dash7(0)
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
#[deku(id_type = "u8")]
pub enum PermissionLevel {
    #[default]
    #[deku(id = "0")]
    User,
    #[deku(id = "1")]
    Root,
    // ALP SPEC: Does something else exist?
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct ActionHeader {
    /// Group with next action
    #[deku(bits = 1)]
    pub group: bool,
    /// Ask for a response (status)
    #[deku(bits = 1)]
    pub response: bool,
    //OpCode would be here. 6 bits padding instead
}

impl ActionHeader {
    pub fn new(group: bool, response: bool) -> Self {
        Self { group, response }
    }
}

// Nop
/// Does nothing
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct Nop {
    pub header: ActionHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,
}

/// Checks whether a file exists
// ALP_SPEC: How is the result of this command different from a read file of size 0?
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct FileIdOperand {
    pub header: ActionHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    pub file_id: u8,
}

/// Write data to a file
// TODO: figure out a way to immediately decode the file
// This will probably invole
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(ctx = "_opcode: OpCode")]
pub struct FileDataOperand<F>
where
    F: for<'a> DekuReader<'a, FileCtx> + DekuWriter<FileCtx>,
{
    pub header: ActionHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    pub data: FileData<F>,
}

impl<F> FileDataOperand<F>
where
    F: for<'a> DekuReader<'a, FileCtx> + DekuWriter<FileCtx>,
{
    pub fn new(header: ActionHeader, data: FileData<F>, opcode: OpCode) -> Self {
        // TODO file id has to match data!
        Self {
            header,
            data,
            opcode,
        }
    }
}
// impl FileDataOperand {
//     fn read<'a, R>(reader: &mut Reader<R>, offset: &Length) -> Result<File, DekuError>
//     where
//         R: no_std_io::Read + no_std_io::Seek,
//     {
//         let length = <Length as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?;
//         let file_id = offset.file_id.try_into()?;
//         File::from_reader_with_ctx(reader, (file_id, Into::<u32>::into(length)))
//     }

//     fn write<W>(writer: &mut Writer<W>, data: &File, offset: &Length) -> Result<(), DekuError>
//     where
//         W: no_std_io::Write + no_std_io::Seek,
//     {
//         let vec_size = match data {
//             File::User { buffer, .. } => buffer.len() as u32,
//             _ => 0,
//         };

//         write_length_prefixed_ext(writer, data, data.deku_id()?, vec_size)
//     }
// }

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct FilePropertiesOperand {
    pub header: ActionHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    pub file_id: u8,
    pub file_header: FileHeader,
}

// Read
/// Read data from a file
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct ReadFileData {
    pub header: ActionHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    pub file_id: u8,
    pub offset: Length,
    pub length: Length,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct ActionQuery {
    pub header: ActionHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    pub query: Query,
}

/// Request a level of permission using some permission type
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct PermissionRequest {
    pub header: ActionHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    /// See operation::permission_level
    pub level: PermissionLevel,
    pub permission: Permission,
}

/// Copy a file to another file
// ALP_SPEC: What does that mean? Is it a complete file copy including the file properties or just
// the data? If not then if the destination file is bigger than the source, does the copy only
// overwrite the first part of the destination file?
//
// Wouldn't it be more appropriate to have 1 size and 2 file offsets?
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct CopyFile {
    pub header: ActionHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    pub src_file_id: u8,

    pub dst_file_id: u8,
}

#[derive(
    DekuRead, DekuWrite, Default, Clone, Copy, Debug, PartialEq, strum::Display, uniffi::Enum,
)]
#[deku(bits = 2, id_type = "u8")]

pub enum StatusType {
    #[default]
    #[deku(id = "0")]
    Action,
    #[deku(id = "1")]
    Interface,

    //#[cfg(feature="wizzilab")]
    #[deku(id = "2")]
    InterfaceFinal,
}

/// Statuses regarding actions sent in a request
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
#[deku(
    ctx = "status_type: StatusType",
    id = "status_type",
    ctx_default = "Default::default()"
)]
pub enum Status {
    // ALP SPEC: This is named status, but it should be named action status compared to the '2'
    // other statuses.
    #[deku(id = "StatusType::Action")]
    Action(ActionStatus),
    #[deku(id = "StatusType::Interface")]
    Interface(InterfaceStatusOperation),

    #[cfg(feature = "_wizzilab")]
    #[deku(id = "StatusType::InterfaceFinal")]
    InterfaceFinal(InterfaceFinalStatusOperation),
    // ALP SPEC: Where are the stack errors?
}

impl Default for Status {
    fn default() -> Self {
        Status::Action(Default::default())
    }
}

impl Into<StatusOperand> for Status {
    fn into(self) -> StatusOperand {
        StatusOperand {
            status_type: self.deku_id().unwrap(),
            opcode: OpCode::Status,
            status: self,
        }
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct StatusOperand {
    pub status_type: StatusType,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    #[deku(ctx = "*status_type")]
    pub status: Status,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct InterfaceStatusOperation {
    pub interface_type: InterfaceType,

    #[deku(
        reader = "InterfaceStatusOperation::read(deku::reader, *interface_type)",
        writer = "InterfaceStatusOperation::write(deku::writer, &self.status)"
    )]
    pub status: InterfaceStatus,
}

impl From<InterfaceStatus> for InterfaceStatusOperation {
    fn from(status: InterfaceStatus) -> Self {
        Self {
            interface_type: status.deku_id().unwrap(),
            status,
        }
    }
}

impl Into<Status> for InterfaceStatusOperation {
    fn into(self) -> Status {
        Status::Interface(self).into()
    }
}

impl InterfaceStatusOperation {
    #[cfg(not(feature = "subiot_v0_0"))]
    pub fn read<'a, R>(
        reader: &mut Reader<'a, R>,
        interface_type: InterfaceType,
    ) -> Result<InterfaceStatus, DekuError>
    where
        R: no_std_io::Read + no_std_io::Seek,
    {
        // Subiot v0.0 was missing the length field
        #[allow(unused_assignments)]
        let mut length = Length(0);

        #[cfg(not(feature = "subiot_v0_0"))]
        {
            length = Length::from_reader_with_ctx(reader, ())?;
        }

        InterfaceStatus::from_reader_with_ctx(reader, (interface_type, length.into()))
    }

    #[cfg(not(feature = "subiot_v0_0"))]
    pub fn write<W>(writer: &mut Writer<W>, status: &InterfaceStatus) -> Result<(), DekuError>
    where
        W: no_std_io::Write + no_std_io::Seek,
    {
        let vec_size = match status {
            InterfaceStatus::Other(val) => val.len() as u32,
            _ => 0,
        };

        // Subiot v0.0 was missing the length field
        #[cfg(feature = "subiot_v0_0")]
        return DekuWriter::to_writer(status, writer, (interface_type.try_into()?, vec_size));

        #[cfg(not(feature = "subiot_v0_0"))]
        return write_length_prefixed(writer, status, (status.deku_id().unwrap(), vec_size));
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct ResponseTagHeader {
    /// Header
    /// End of packet
    ///
    /// Signal the last response packet for the request `id`
    #[deku(bits = 1)]
    pub end_of_packet: bool,
    /// An error occured
    #[deku(bits = 1)]
    pub error: bool,
}

/// Action received before any responses to a request that contained a RequestTag
///
/// This allows matching responses to requests when doing multiple requests in parallel.
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct ResponseTag {
    pub header: ResponseTagHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    pub id: u8,
}

// Special
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
#[deku(bits = 2, id_type = "u8")]
pub enum ChunkStep {
    #[default]
    #[deku(id = "0")]
    Continue,
    #[deku(id = "1")]
    Start,
    #[deku(id = "2")]
    End,
    #[deku(id = "3")]
    StartEnd,
}

impl Into<Chunk> for ChunkStep {
    fn into(self) -> Chunk {
        Chunk {
            step: self,
            opcode: OpCode::Chunk,
        }
    }
}

/// Provide chunk information and therefore allows to send an ALP command by chunks.
///
/// Specification:
/// An ALP Command may be chunked into multiple Chunks. A special Chunk Action is inserted at the beginning of each
/// ALP Command Chunk to define its chunk state: START, CONTINUE or END (see 6.2.2.1). If the Chunk Action is not
/// present, the ALP Command is not chunked (implicit START/END). The Group (11.5.3) and Break Query conditions are
/// extended over all chunks of the ALP Command.
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct Chunk {
    pub step: ChunkStep,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,
}

/// Provide logical link of a group of queries
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
#[deku(bits = 2, id_type = "u8")]
pub enum LogicOp {
    #[default]
    #[deku(id = "0")]
    Or,
    #[deku(id = "1")]
    Xor,
    #[deku(id = "2")]
    Nor,
    #[deku(id = "3")]
    Nand,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct Logic {
    pub logic: LogicOp,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct ForwardHeader {
    #[deku(bits = 1, pad_bits_before = "1")]
    pub response: bool,
}

/// Forward rest of the command over the interface
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct Forward {
    pub header: ForwardHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    pub configuration: InterfaceConfiguration,
}

impl Forward {
    pub fn new(response: bool, configuration: InterfaceConfiguration) -> Self {
        Self {
            header: ForwardHeader { response },
            configuration,
            opcode: OpCode::Forward,
        }
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct IndirectForwardHeader {
    #[deku(bits = 1)]
    pub overloaded: bool,

    #[deku(bits = 1)]
    pub response: bool,
}

impl IndirectForwardHeader {
    pub fn new(overloaded: bool, response: bool) -> Self {
        Self {
            overloaded,
            response,
        }
    }
}

/// Forward rest of the command over the interface
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct IndirectForward {
    #[deku(
        update = "IndirectForwardHeader::new(self.configuration.is_some(), self.header.response)"
    )]
    pub header: IndirectForwardHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    #[deku(cond = "header.overloaded")]
    pub configuration: Option<IndirectInterface>,
}

impl IndirectForward {
    pub fn new(response: bool, configuration: Option<IndirectInterface>) -> Self {
        Self {
            header: IndirectForwardHeader {
                overloaded: configuration.is_some(),
                response,
            },
            opcode: OpCode::IndirectForward,
            configuration,
        }
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct RequestTagHeader {
    #[deku(bits = 1, pad_bits_after = "1")]
    pub end_of_packet: bool,
}

/// Provide command payload identifier
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct RequestTag {
    pub header: RequestTagHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    pub id: u8,
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "_opcode: OpCode")]
pub struct Extension {
    pub header: ActionHeader,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;
    #[cfg(feature = "_wizzilab")]
    use crate::transport::GroupCondition;
    use crate::{
        link::AccessClass,
        network::{Address, Addressee, NlsState},
        physical::{Channel, ChannelBand, ChannelClass, ChannelCoding, ChannelHeader},
        session::Dash7InterfaceStatus,
        test_tools::test_item,
    };

    #[test]
    fn test_length() {
        test_item(Length(1), &[0x01]);
        test_item(Length(65), &[0x40, 0x41]);
        test_item(Length(4263936), &[0xC0, 0x41, 0x10, 0x00]);
    }

    #[test]
    fn test_file_offset() {
        test_item(
            FileOffset {
                file_id: 2,
                offset: 0x3F_FFu32.into(),
            },
            &hex!("02 7F FF"),
        )
    }

    #[test]
    fn test_action_status() {
        test_item(
            ActionStatus {
                action_id: 2,
                status: StatusCode::UnknownOperation,
            },
            &hex!("02 F6"),
        )
    }

    #[test]
    fn test_interface_status() {
        let data = &hex!("D7 14 32 00 32 2D 3E 50 80 00 00 58 20 01 39 38 38 37 00 39 00 2E");

        let item: InterfaceStatusOperation = InterfaceStatus::Dash7(Dash7InterfaceStatus {
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
        .into();

        test_item::<InterfaceStatusOperation>(item, data);
    }
}
