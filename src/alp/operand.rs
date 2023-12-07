use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    ctx::{BitSize, Endian},
    prelude::*,
};

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, Copy)]
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

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct FileData {
    pub offset: FileOffset,

    #[deku(update = "self.data.len()")]
    length: Length,

    #[deku(count = "length", endian = "big")]
    data: Vec<u8>,
}

impl FileData {
    pub fn new(offset: FileOffset, data: Vec<u8>) -> Self {
        Self {
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

// // /// Meta data required to send a packet depending on the sending interface type
// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// #[deku(bits=8, type="u8")]
// pub enum InterfaceConfiguration {
//     #[deku(id="0")] Host,
//     #[deku(id="0xD7")] D7asp(dash7::InterfaceConfiguration),
// }

// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct InterfaceStatusUnknown {
//     pub id: u8,
//     pub data: Vec<u8>,
// }
/// Meta data from a received packet depending on the receiving interface type
// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// #[deku(bits=8, type="u8")]
// pub enum InterfaceStatus {
//     #[deku(id="0")] Host,
//     #[deku(id="0xD7")] D7asp(dash7::InterfaceStatus),
//     Unknown(InterfaceStatusUnknown),
// }
// #[derive(Debug, Copy, Clone, Hash, PartialEq)]
// pub enum InterfaceStatusDecodingError {
//     MissingBytes(usize),
//     BadInterfaceId(u8),
// }
// impl From<StdError> for InterfaceStatusDecodingError {
//     fn from(e: StdError) -> Self {
//         match e {
//             StdError::MissingBytes(n) => Self::MissingBytes(n),
//         }
//     }
// }
// impl Codec for InterfaceStatus {
//     type Error = InterfaceStatusDecodingError;
//     fn encoded_size(&self) -> usize {
//         let data_size = match self {
//             InterfaceStatus::Host => 0,
//             InterfaceStatus::D7asp(itf) => itf.encoded_size(),
//             InterfaceStatus::Unknown(InterfaceStatusUnknown { data, .. }) => data.len(),
//         };
//         1 + unsafe { varint::size(data_size as u32) } as usize + data_size
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         let mut offset = 1;
//         match self {
//             InterfaceStatus::Host => {
//                 out[0] = InterfaceId::Host as u8;
//                 out[1] = 0;
//                 offset += 1;
//             }
//             InterfaceStatus::D7asp(v) => {
//                 out[0] = InterfaceId::D7asp as u8;
//                 let size = v.encoded_size() as u32;
//                 let size_size = varint::encode_in(size, &mut out[offset..]);
//                 offset += size_size as usize;
//                 offset += v.encode_in(&mut out[offset..]);
//             }
//             InterfaceStatus::Unknown(InterfaceStatusUnknown { id, data, .. }) => {
//                 out[0] = *id;
//                 let size = data.len() as u32;
//                 let size_size = varint::encode_in(size, &mut out[offset..]);
//                 offset += size_size as usize;
//                 out[offset..offset + data.len()].clone_from_slice(data);
//                 offset += data.len();
//             }
//         };
//         offset
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.is_empty() {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
//         }
//         const HOST: u8 = InterfaceId::Host as u8;
//         const D7ASP: u8 = InterfaceId::D7asp as u8;
//         let mut offset = 1;
//         let value = match out[0] {
//             HOST => {
//                 offset += 1;
//                 InterfaceStatus::Host
//             }
//             D7ASP => {
//                 let WithSize {
//                     value: size,
//                     size: size_size,
//                 } = varint::decode(&out[offset..]).map_err(|e| {
//                     let WithOffset { offset: off, value } = e;
//                     WithOffset {
//                         offset: offset + off,
//                         value: value.into(),
//                     }
//                 })?;
//                 let size = size as usize;
//                 offset += size_size;
//                 let WithSize { value, size } =
//                     dash7::InterfaceStatus::decode(&out[offset..offset + size]).map_err(|e| {
//                         let WithOffset { offset: off, value } = e;
//                         WithOffset {
//                             offset: offset + off,
//                             value: value.into(),
//                         }
//                     })?;
//                 offset += size;
//                 InterfaceStatus::D7asp(value)
//             }
//             id => {
//                 let WithSize {
//                     value: size,
//                     size: size_size,
//                 } = varint::decode(&out[offset..]).map_err(|e| {
//                     let WithOffset { offset: off, value } = e;
//                     WithOffset {
//                         offset: offset + off,
//                         value: value.into(),
//                     }
//                 })?;
//                 let size = size as usize;
//                 offset += size_size;
//                 if out.len() < offset + size {
//                     return Err(WithOffset::new(
//                         offset,
//                         Self::Error::MissingBytes(offset + size - out.len()),
//                     ));
//                 }
//                 let mut data = vec![0u8; size].into_boxed_slice();
//                 data.clone_from_slice(&out[offset..size]);
//                 offset += size;
//                 InterfaceStatus::Unknown(InterfaceStatusUnknown { id, data })
//             }
//         };
//         Ok(WithSize {
//             value,
//             size: offset,
//         })
//     }
// }
// #[test]
// fn test_interface_status_d7asp() {
//     test_item(
//         InterfaceStatus::D7asp(dash7::InterfaceStatus {
//             ch_header: 1,
//             ch_idx: 0x0123,
//             rxlev: 2,
//             lb: 3,
//             snr: 4,
//             status: 0xB0,
//             token: 6,
//             seq: 7,
//             resp_to: 8,
//             access_class: 0xFF,
//             address: dash7::Address::Vid([0xAB, 0xCD]),
//             nls_state: dash7::NlsState::AesCcm32(hex!("00 11 22 33 44")),
//         }),
//         &hex!("D7 13    01 0123 02 03 04 B0 06 07 08   37 FF ABCD  0011223344"),
//     )
// }
// #[test]
// fn test_interface_status_host() {
//     test_item(InterfaceStatus::Host, &hex!("00 00"))
// }

// ===============================================================================
// Operands
// ===============================================================================
/// Describe the location of some data on the filesystem (file + data offset).

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct FileOffset {
    pub file_id: u8,
    pub offset: Length,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(type = "u8")]
pub enum StatusCode {
    /// Status code that can be received as a result of some ALP actions.
    /// Action received and partially completed at response. To be completed after response
    #[deku(id = "0x00")]
    Ok,
    #[deku(id = "0x01")]
    Received,
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
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
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
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(type = "u8")]
pub enum Permission {
    #[deku(id = "0x42")] // ALP_SPEC Undefined
    Dash7([u8; 8]),
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(type = "u8")]
pub enum PermissionLevel {
    #[deku(id = "0")]
    User,
    #[deku(id = "1")]
    Root,
    // ALP SPEC: Does something else exist?
}

// /// Non Dash7 interface
// #[derive(Clone, Debug, PartialEq)]
// // ALP SPEC: This seems undoable if we do not know the interface (per protocol specific support)
// //  which is still a pretty legitimate policy on a low power protocol.
// pub struct NonOverloadedIndirectInterface {
//     pub interface_file_id: u8,
//     // ALP SPEC: Where is this defined? Is this ID specific?
//     pub data: Box<[u8]>,
// }

// impl Codec for NonOverloadedIndirectInterface {
//     type Error = StdError;
//     fn encoded_size(&self) -> usize {
//         1 + self.data.len()
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         out[0] = self.interface_file_id;
//         let mut offset = 1;
//         out[offset..offset + self.data.len()].clone_from_slice(&self.data);
//         offset += self.data.len();
//         offset
//     }
//     fn decode(_out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         todo!("TODO")
//     }
// }

// #[derive(Clone, Debug, PartialEq)]
// pub enum IndirectInterface {
//     Overloaded(OverloadedIndirectInterface),
//     NonOverloaded(NonOverloadedIndirectInterface),
// }

// impl Codec for IndirectInterface {
//     type Error = StdError;
//     fn encoded_size(&self) -> usize {
//         match self {
//             IndirectInterface::Overloaded(v) => v.encoded_size(),
//             IndirectInterface::NonOverloaded(v) => v.encoded_size(),
//         }
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         match self {
//             IndirectInterface::Overloaded(v) => v.encode_in(out),
//             IndirectInterface::NonOverloaded(v) => v.encode_in(out),
//         }
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.is_empty() {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
//         }
//         Ok(if out[0] & 0x80 != 0 {
//             let WithSize { size, value } =
//                 OverloadedIndirectInterface::decode(&out[1..]).map_err(|e| e.shift(1))?;
//             WithSize {
//                 size: size + 1,
//                 value: Self::Overloaded(value),
//             }
//         } else {
//             let WithSize { size, value } =
//                 NonOverloadedIndirectInterface::decode(&out[1..]).map_err(|e| e.shift(1))?;
//             WithSize {
//                 size: size + 1,
//                 value: Self::NonOverloaded(value),
//             }
//         })
//     }
// }

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;
    use crate::test_tools::test_item;

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
}
