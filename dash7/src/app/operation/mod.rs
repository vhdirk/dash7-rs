use std::{borrow::Cow, default};

use deku::{
    ctx::{BitSize, Endian},
    no_std_io,
    prelude::*,
};

use super::interface::InterfaceConfiguration;
pub use super::query::Query;
use crate::{session::InterfaceType, utils::write_length_prefixed_ext};
use crate::utils::{read_length_prefixed, write_length_prefixed};
use crate::{data::FileHeader, file::File, session::InterfaceStatus};

mod length;
mod file_offset;
pub use length::*;
pub use file_offset::*;

#[cfg(feature = "_wizzilab")]
pub use super::interface_final::*;

pub trait Operation {
    type Header;

    fn header(&self) -> Result<Cow<Self::Header>, DekuError>
    where
        Self::Header: Clone;
}

// ===============================================================================
// Operations
// ===============================================================================


#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
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
    #[deku(id = "0xFF")]
    FileIdMissing,
    #[deku(id = "0xFE")]
    CreateFileIdAlreadyExist,
    #[deku(id = "0xFD")]
    FileIsNotRestorable,
    #[deku(id = "0xFC")]
    InsufficientPermission,
    #[deku(id = "0xFB")]
    CreateFileLengthOverflow,
    #[deku(id = "0xFA")]
    CreateFileAllocationOverflow, // ALP_SPEC: ??? Difference with the previous one?;
    #[deku(id = "0xF9")]
    WriteOffsetOverflow,
    #[deku(id = "0xF8")]
    WriteDataOverflow,
    #[deku(id = "0xF7")]
    WriteStorageUnavailable,
    #[deku(id = "0xF6")]
    UnknownOperation,
    #[deku(id = "0xF5")]
    OperationIncomplete,
    #[deku(id = "0xF4")]
    OperationWrongFormat,
    #[deku(id = "0x80")]
    UnknownError,
}

impl StatusCode {
    pub fn is_err(&self) -> bool {
        self.deku_id().unwrap() > 0x80
    }
}

/// Result of an action in a previously sent request
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
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
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(id_type = "u8")]
pub enum Permission {
    #[deku(id = "0x42")] // ALP_SPEC Undefined
    Dash7([u8; 8]),
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(id_type = "u8")]
pub enum PermissionLevel {
    #[deku(id = "0")]
    User,
    #[deku(id = "1")]
    Root,
    // ALP SPEC: Does something else exist?
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
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
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct Nop {
    #[deku(skip, default = "header")]
    pub header: <Self as Operation>::Header,
}

impl Operation for Nop {
    type Header = ActionHeader;

    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
}

/// Checks whether a file exists
// ALP_SPEC: How is the result of this command different from a read file of size 0?
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct FileId {
    #[deku(skip, default = "header")]
    pub header: <Self as Operation>::Header,
    pub file_id: u8,
}

impl Operation for FileId {
    type Header = ActionHeader;

    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
}

/// Write data to a file
// TODO: figure out a way to immediately decode the file
// This will probably invole
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct FileData {
    #[deku(skip, default = "header")]
    pub header: <Self as Operation>::Header,

    pub offset: FileOffset,

    #[deku(
        reader = "FileData::read(deku::reader, offset)",
        writer = "FileData::write(deku::writer, &self.data, &self.offset)"
    )]
    data: File,
}

impl Operation for FileData {
    type Header = ActionHeader;

    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
}

impl FileData {
    pub fn new(header: ActionHeader, offset: FileOffset, data: File) -> Self {
        // TODO file id has to match data!
        Self {
            header,
            offset,
            data,
        }
    }

    fn read<'a, R>(reader: &mut Reader<R>, offset: &FileOffset) -> Result<File, DekuError>
    where
        R: no_std_io::Read + no_std_io::Seek,
    {
        let length = <Length as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?;
        let file_id = offset.file_id.try_into()?;
        File::from_reader_with_ctx(reader, (file_id, Into::<u32>::into(length)))
    }

    fn write<W>(writer: &mut Writer<W>, data: &File, offset: &FileOffset) -> Result<(), DekuError>
    where
        W: no_std_io::Write + no_std_io::Seek,
    {
        let vec_size = match data {
            File::Other(val) => val.len() as u32,
            _ => 0,
        };

        write_length_prefixed_ext(writer, data, offset.file_id, vec_size)
    }
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct FileProperties {
    #[deku(skip, default = "header")]
    pub header: <Self as Operation>::Header,

    pub file_id: u8,
    pub file_header: FileHeader,
}

impl Operation for FileProperties {
    type Header = ActionHeader;

    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
}

// Read
/// Read data from a file
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct ReadFileData {
    #[deku(skip, default = "header")]
    pub header: <Self as Operation>::Header,

    pub offset: FileOffset,
    pub length: Length,
}

impl Operation for ReadFileData {
    type Header = ActionHeader;
    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct ActionQuery {
    #[deku(skip, default = "header")]
    pub header: <Self as Operation>::Header,

    pub query: Query,
}

impl Operation for ActionQuery {
    type Header = ActionHeader;
    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
}

/// Request a level of permission using some permission type
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct PermissionRequest {
    #[deku(skip, default = "header")]
    pub header: <Self as Operation>::Header,
    /// See operation::permission_level
    pub level: PermissionLevel,
    pub permission: Permission,
}

impl Operation for PermissionRequest {
    type Header = ActionHeader;
    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
}

/// Copy a file to another file
// ALP_SPEC: What does that mean? Is it a complete file copy including the file properties or just
// the data? If not then if the destination file is bigger than the source, does the copy only
// overwrite the first part of the destination file?
//
// Wouldn't it be more appropriate to have 1 size and 2 file offsets?
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct CopyFile {
    #[deku(skip, default = "header")]
    pub header: <Self as Operation>::Header,
    pub src_file_id: u8,
    pub dst_file_id: u8,
}

impl Operation for CopyFile {
    type Header = ActionHeader;
    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
}

#[derive(DekuRead, DekuWrite, Default, Clone, Copy, Debug, PartialEq)]
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
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "status_type: StatusType", id = "status_type", ctx_default="Default::default()")]
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



impl Operation for Status {
    type Header = StatusType;
    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        self.deku_id().map(|x| Cow::Owned(x))
    }
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct InterfaceStatusOperation {
    pub interface_id: u8,

    #[deku(
        reader = "InterfaceStatusOperation::read(deku::reader)",
        writer = "InterfaceStatusOperation::write(deku::writer, &self.status)"
    )]
    pub status: InterfaceStatus,
}

impl From<InterfaceStatus> for InterfaceStatusOperation {
    fn from(status: InterfaceStatus) -> Self {
        Self {
            interface_id: status.deku_id().unwrap().deku_id().unwrap(),
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
    ) -> Result<InterfaceStatus, DekuError>
    where
        R: no_std_io::Read + no_std_io::Seek,
    {
        // Subiot v0.0 was missing the length field
        #[cfg(feature = "subiot_v0_0")]
        return InterfaceStatus::from_reader_with_ctx(reader, (interface_id.try_into()?, 0));

        #[cfg(not(feature = "subiot_v0_0"))]
        return read_length_prefixed::< _, u32, _>(reader);
    }

    #[cfg(not(feature = "subiot_v0_0"))]
    pub fn write<W>(
        writer: &mut Writer<W>,
        status: &InterfaceStatus,
    ) -> Result<(), DekuError>
    where
        W: no_std_io::Write + no_std_io::Seek,
    {
        let vec_size = match status {
            InterfaceStatus::Other(val) => val.len() as u32,
            _ => 0,
        };

        // Subiot v0.0 was missing the length field
        #[cfg(feature = "subiot_v0_0")]
        return DekuWriter::to_writer(status, writer, (interface_id.try_into()?, vec_size));

        #[cfg(not(feature = "subiot_v0_0"))]
        return write_length_prefixed(writer, status, vec_size);
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct ResponseTagHeader {
    /// Header
    /// End of packet
    ///
    /// Signal the last response packet for the request `id`
    #[deku(bits = 1)]
    pub eop: bool,
    /// An error occured
    #[deku(bits = 1)]
    pub error: bool,
}

/// Action received before any responses to a request that contained a RequestTag
///
/// This allows matching responses to requests when doing multiple requests in parallel.
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct ResponseTag {
    #[deku(skip, default = "header")]
    pub header: <Self as Operation>::Header,

    pub id: u8,
}

impl Operation for ResponseTag {
    type Header = ResponseTagHeader;
    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
}

// Special
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, Default)]
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

/// Provide chunk information and therefore allows to send an ALP command by chunks.
///
/// Specification:
/// An ALP Command may be chunked into multiple Chunks. A special Chunk Action is inserted at the beginning of each
/// ALP Command Chunk to define its chunk state: START, CONTINUE or END (see 6.2.2.1). If the Chunk Action is not
/// present, the ALP Command is not chunked (implicit START/END). The Group (11.5.3) and Break Query conditions are
/// extended over all chunks of the ALP Command.
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct Chunk {
    #[deku(skip, default = "header")]
    pub step: <Self as Operation>::Header,
}

impl Operation for Chunk {
    type Header = ChunkStep;
    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.step))
    }
}

/// Provide logical link of a group of queries
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, Default)]
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

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct Logic {
    #[deku(skip, default = "header")]
    pub logic: LogicOp,
}

impl Operation for Logic {
    type Header = LogicOp;
    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.logic))
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct ForwardHeader {
    #[deku(bits = 1, pad_bits_before = "1", pad_bits_after = "6")]
    pub response: bool,
}

/// Forward rest of the command over the interface
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct Forward {
    #[deku(skip, default = "header")]
    pub header: <Self as Operation>::Header,

    pub configuration: InterfaceConfiguration,
}

impl Operation for Forward {
    type Header = ForwardHeader;

    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
}

impl Forward {
    pub fn new(response: bool, configuration: InterfaceConfiguration) -> Self {
        Self {
            header: ForwardHeader { response },
            configuration,
        }
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
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
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct IndirectForward {
    #[deku(
        skip,
        default = "header",
        update = "IndirectForwardHeader::new(self.configuration.is_some(), self.header.response)"
    )]
    pub header: IndirectForwardHeader,

    pub interface_file_id: u8,

    #[deku(
        cond = "header.overloaded",
    )]
    pub configuration: Option<InterfaceConfiguration>,
}

impl Operation for IndirectForward {
    type Header = IndirectForwardHeader;

    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
}

impl IndirectForward {
    pub fn new(
        response: bool,
        interface_file_id: u8,
        configuration: Option<InterfaceConfiguration>,
    ) -> Self {
        Self {
            header: IndirectForwardHeader {
                overloaded: configuration.is_some(),
                response,
            },
            interface_file_id,
            configuration,
        }
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct RequestTagHeader {
    #[deku(bits = 1, pad_bits_after = "1")]
    pub eop: bool,
}

/// Provide command payload identifier
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct RequestTag {
    #[deku(skip, default = "header")]
    pub header: <Self as Operation>::Header,

    pub id: u8,
}

impl Operation for RequestTag {
    type Header = RequestTagHeader;

    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "header: <Self as Operation>::Header", ctx_default="Default::default()")]
pub struct Extension {
    #[deku(skip, default = "header")]
    pub header: ActionHeader,
}

impl Operation for Extension {
    type Header = ActionHeader;

    fn header(&self) -> Result<Cow<Self::Header>, DekuError> {
        Ok(Cow::Borrowed(&self.header))
    }
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
                Address::Uid(4123107267735781422u64),
                NlsState::None,
                AccessClass::new(0, 1),
            ),
        })
        .into();

        test_item::<InterfaceStatusOperation>(item, data);
    }
}
