use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    ctx::{BitSize, Endian},
    prelude::*,
};

use super::interface::InterfaceConfiguration;
pub use super::query::Query;
use crate::session::InterfaceType;
use crate::utils::{read_length_prefixed, write_length_prefixed};
use crate::{data::FileHeader, file::File, session::InterfaceStatus};

#[cfg(feature = "_wizzilab")]
pub use super::interface_final::*;

// ===============================================================================
// Operands
// ===============================================================================

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, Copy)]
pub struct Length(
    #[deku(
        reader = "Length::read(deku::rest)",
        writer = "Length::write(deku::output, &self.0)"
    )]
    u32,
);

impl Into<u32> for Length {
    fn into(self) -> u32 {
        self.0 as u32
    }
}

impl From<u32> for Length {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Into<usize> for Length {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl From<usize> for Length {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl Length {
    fn required_bits(value: u32) -> u32 {
        // This may be slow. There are faster ways, but we're not optimising for speed anyway
        value.checked_ilog2().unwrap_or(0) + 1
    }

    fn read(rest: &BitSlice<u8, Msb0>) -> Result<(&BitSlice<u8, Msb0>, u32), DekuError> {
        let (rest, size) = <u8 as DekuRead<'_, _>>::read(rest, (Endian::Big, BitSize(2)))?;
        let (rest, value) = <u32 as DekuRead<'_, _>>::read(
            rest,
            (Endian::Big, BitSize((6 + (size * u8::BITS as u8)) as usize)),
        )?;
        Ok((rest, value))
    }

    fn write(output: &mut BitVec<u8, Msb0>, value: &u32) -> Result<(), DekuError> {
        let num_extra_bits = Length::required_bits(*value).checked_sub(6).unwrap_or(0);

        let mut num_extra_bytes = num_extra_bits.checked_div(u8::BITS).unwrap_or(0);
        if (num_extra_bits % u8::BITS) > 0 {
            num_extra_bytes += 1;
        }

        DekuWrite::write(&num_extra_bytes, output, (Endian::Big, BitSize(2)))?;
        DekuWrite::write(
            value,
            output,
            (
                Endian::Big,
                BitSize((6 + num_extra_bytes * u8::BITS) as usize),
            ),
        )?;

        Ok(())
    }
}

/// Describe the location of some data on the filesystem (file + data offset).
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct FileOffset {
    pub file_id: u8,
    pub offset: Length,
}

impl FileOffset {
    pub fn no_offset(file_id: u8) -> Self {
        Self {
            file_id,
            offset: 0u32.into(),
        }
    }
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(type = "u8")]
pub enum StatusCode {
    /// Status code that can be received as a result of some ALP actions.
    /// Action received and partially completed at response. To be completed after response
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
    OperandIncomplete,
    #[deku(id = "0xF4")]
    OperandWrongFormat,
    #[deku(id = "0x80")]
    UnknownError,
}

impl StatusCode {
    pub fn is_err(&self) -> bool {
        self.deku_id().unwrap() > 0x80
    }
}

/// Result of an action in a previously sent request
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
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
#[deku(type = "u8")]
pub enum Permission {
    #[deku(id = "0x42")] // ALP_SPEC Undefined
    Dash7([u8; 8]),
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(type = "u8")]
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
    #[deku(bits = 1, pad_bits_after = "6")]
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

/// Write data to a file
// TODO: figure out a way to immediately decode the file
// This will probably invole
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct FileData {
    pub header: ActionHeader,

    pub offset: FileOffset,

    #[deku(
        reader = "FileData::read(deku::rest, offset)",
        writer = "FileData::write(deku::output, &self.data, &self.offset)"
    )]
    data: File,
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

    fn read<'a>(
        rest: &'a BitSlice<u8, Msb0>,
        offset: &FileOffset,
    ) -> Result<(&'a BitSlice<u8, Msb0>, File), DekuError> {
        let (rest, length) = <Length as DekuRead<'_, _>>::read(rest, ())?;
        let file_id = offset.file_id.try_into()?;
        File::read(rest, (file_id, Into::<u32>::into(length)))
    }

    fn write(
        output: &mut BitVec<u8, Msb0>,
        data: &File,
        offset: &FileOffset,
    ) -> Result<(), DekuError> {
        let vec_size = match data {
            File::Other(val) => val.len() as u32,
            _ => 0,
        };

        write_length_prefixed(output, data, offset.file_id, vec_size)
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

    //#[cfg(feature="wizzilab")]
    #[deku(id = "2")]
    InterfaceFinal,
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
    Interface(InterfaceStatusOperand),

    #[cfg(feature = "_wizzilab")]
    #[deku(id = "StatusType::InterfaceFinal")]
    InterfaceFinal(InterfaceFinalStatusOperand),
    // ALP SPEC: Where are the stack errors?
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct InterfaceStatusOperand {
    pub interface_id: u8,

    #[deku(
        reader = "InterfaceStatusOperand::read(deku::rest, *interface_id)",
        writer = "InterfaceStatusOperand::write(deku::output, &self.status, self.interface_id)"
    )]
    pub status: InterfaceStatus,
}

impl From<InterfaceStatus> for InterfaceStatusOperand {
    fn from(status: InterfaceStatus) -> Self {
        Self {
            interface_id: status.deku_id().unwrap().deku_id().unwrap(),
            status,
        }
    }
}

impl Into<StatusOperand> for InterfaceStatusOperand {
    fn into(self) -> StatusOperand {
        Status::Interface(self).into()
    }
}

impl InterfaceStatusOperand {
    #[cfg(not(feature = "subiot_v0_0"))]
    pub fn read<'a>(
        rest: &'a BitSlice<u8, Msb0>,
        interface_id: u8,
    ) -> Result<(&'a BitSlice<u8, Msb0>, InterfaceStatus), DekuError> {
        // Subiot v0.0 was missing the length field
        #[cfg(feature = "subiot_v0_0")]
        return InterfaceStatus::read(rest, (interface_id.try_into()?, 0));

        #[cfg(not(feature = "subiot_v0_0"))]
        return read_length_prefixed(rest, interface_id);
    }

    #[cfg(not(feature = "subiot_v0_0"))]
    pub fn write(
        output: &mut BitVec<u8, Msb0>,
        status: &InterfaceStatus,
        interface_id: u8,
    ) -> Result<(), DekuError> {
        let vec_size = match status {
            InterfaceStatus::Other(val) => val.len() as u32,
            _ => 0,
        };

        // Subiot v0.0 was missing the length field
        #[cfg(feature = "subiot_v0_0")]
        return DekuWrite::write(status, output, (interface_id.try_into()?, vec_size));

        #[cfg(not(feature = "subiot_v0_0"))]
        return write_length_prefixed(output, status, interface_id, vec_size);
    }
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
        network::{Address, Addressee, NlsState},
        physical::{Channel, ChannelBand, ChannelClass, ChannelCoding, ChannelHeader},
        session::Dash7InterfaceStatus,
        test_tools::test_item,
        transport::GroupCondition,
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

        let item: InterfaceStatusOperand = InterfaceStatus::Dash7(Dash7InterfaceStatus {
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
                false,
                GroupCondition::Any,
                Address::Uid(4123107267735781422u64),
                NlsState::None,
                1,
            ),
        })
        .into();

        test_item::<InterfaceStatusOperand>(item, data);
    }
}
