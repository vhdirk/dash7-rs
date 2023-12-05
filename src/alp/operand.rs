use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    ctx::{BitSize, Endian},
    prelude::*,
};

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 3, type = "u8")]
pub enum ArithmeticComparisonType {
    #[deku(id = "0")]
    Inequal,
    #[deku(id = "1")]
    Equal,
    #[deku(id = "2")]
    LessThan,
    #[deku(id = "3")]
    LessThanOrEqual,
    #[deku(id = "4")]
    GreaterThan,
    #[deku(id = "5")]
    GreaterThanOrEqual,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ArithmeticQueryParams {
    #[deku(bits = 1)]
    pub signed: bool,
    pub comparison_type: ArithmeticComparisonType,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq, Copy)]
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

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct FileData {
    pub offset: FileOffset,

    #[deku(temp, update = "self.data.len()")]
    length: Length,

    #[deku(count = "length", endian = "big")]
    pub data: Vec<u8>,
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

pub mod status_code {
    //! Status code that can be received as a result of some ALP actions.
    /// Action received and partially completed at response. To be completed after response
    pub const RECEIVED: u8 = 1;
    pub const OK: u8 = 0;
    pub const FILE_ID_MISSING: u8 = 0xFF;
    pub const CREATE_FILE_ID_ALREADY_EXIST: u8 = 0xFE;
    pub const FILE_IS_NOT_RESTORABLE: u8 = 0xFD;
    pub const INSUFFICIENT_PERMISSION: u8 = 0xFC;
    pub const CREATE_FILE_LENGTH_OVERFLOW: u8 = 0xFB;
    pub const CREATE_FILE_ALLOCATION_OVERFLOW: u8 = 0xFA; // ALP_SPEC: ??? Difference with the previous one?;
    pub const WRITE_OFFSET_OVERFLOW: u8 = 0xF9;
    pub const WRITE_DATA_OVERFLOW: u8 = 0xF8;
    pub const WRITE_STORAGE_UNAVAILABLE: u8 = 0xF7;
    pub const UNKNOWN_OPERATION: u8 = 0xF6;
    pub const OPERAND_INCOMPLETE: u8 = 0xF5;
    pub const OPERAND_WRONG_FORMAT: u8 = 0xF4;
    pub const UNKNOWN_ERROR: u8 = 0x80;
}

/// Result of an action in a previously sent request
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Status {
    /// Index of the ALP action associated with this status, in the original request as seen from
    /// the receiver side.
    // ALP_SPEC This is complicated to process because we have to known/possibly infer the position
    // of the action on the receiver side, and that we have to do that while also interpreting who
    // responded (the local modem won't have the same index as the distant device.).
    pub action_id: u8,
    /// Result code
    pub status: u8,
}
// impl Codec for Status {
//     type Error = StdError;
//     fn encoded_size(&self) -> usize {
//         1 + 1
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         out[0] = self.action_id;
//         out[1] = self.status as u8;
//         2
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.len() < 2 {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(
//                 2 - out.len(),
//             )));
//         }
//         Ok(WithSize {
//             value: Self {
//                 action_id: out[0],
//                 status: out[1],
//             },
//             size: 2,
//         })
//     }
// }
// #[test]
// fn test_status_operand() {
//     test_item(
//         Status {
//             action_id: 2,
//             status: status_code::UNKNOWN_OPERATION,
//         },
//         &hex!("02 F6"),
//     )
// }

// // ALP SPEC: where is this defined? Link? Not found in either specs !
// #[derive(Clone, Copy, Debug, PartialEq)]
// pub enum Permission {
//     Dash7([u8; 8]),
// }

// impl Permission {
//     fn id(self) -> u8 {
//         match self {
//             Permission::Dash7(_) => 0x42, // ALP_SPEC Undefined
//         }
//     }
// }

// #[derive(Debug, Copy, Clone, Hash, PartialEq)]
// pub enum PermissionDecodingError {
//     MissingBytes(usize),
//     UnknownId(u8),
// }

// impl Codec for Permission {
//     type Error = PermissionDecodingError;
//     fn encoded_size(&self) -> usize {
//         1 + match self {
//             Permission::Dash7(_) => 8,
//         }
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         out[0] = self.id();
//         1 + match self {
//             Permission::Dash7(token) => {
//                 out[1..1 + token.len()].clone_from_slice(&token[..]);
//                 8
//             }
//         }
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.is_empty() {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
//         }
//         let mut offset = 1;
//         match out[0] {
//             0x42 => {
//                 let mut token = [0; 8];
//                 token.clone_from_slice(&out[offset..offset + 8]);
//                 offset += 8;
//                 Ok(WithSize {
//                     value: Permission::Dash7(token),
//                     size: offset,
//                 })
//             }
//             x => Err(WithOffset::new_head(Self::Error::UnknownId(x))),
//         }
//     }
// }

// pub mod permission_level {
//     pub const USER: u8 = 0;
//     pub const ROOT: u8 = 1;
//     // ALP SPEC: Does something else exist?
// }

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 3, type = "u8")]
pub enum RangeComparisonType {
    NotInRange = 0,
    InRange = 1,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 3, type = "u8")]
pub enum QueryCode {
    NonVoid = 0,
    ComparisonWithZero = 1,
    ComparisonWithValue = 2,
    ComparisonWithOtherFile = 3,
    BitmapRangeComparison = 4,
    StringTokenSearch = 7,
}

// // ALP_SPEC Does this fail if the content overflows the file?
// /// Checks if the file content exists.
// #[derive(Clone, Copy, Debug, PartialEq)]
// pub struct NonVoid {
//     pub size: u32,
//     pub file: FileOffset,
// }
// impl Codec for NonVoid {
//     type Error = QueryOperandDecodingError;
//     fn encoded_size(&self) -> usize {
//         1 + unsafe { varint::size(self.size) } as usize + self.file.encoded_size()
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         out[0] = QueryCode::NonVoid as u8;
//         let mut offset = 1;
//         offset += varint::encode_in(self.size, &mut out[offset..]) as usize;
//         offset += self.file.encode_in(&mut out[offset..]);
//         offset
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.len() < 3 {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(
//                 3 - out.len(),
//             )));
//         }
//         let mut offset = 1;
//         let WithSize {
//             value: size,
//             size: size_size,
//         } = varint::decode(&out[offset..]).map_err(|e| {
//             let WithOffset { offset: off, value } = e;
//             WithOffset {
//                 offset: offset + off,
//                 value: Self::Error::Size(value),
//             }
//         })?;
//         offset += size_size;
//         let WithSize {
//             value: file,
//             size: file_size,
//         } = FileOffset::decode(&out[offset..]).map_err(|e| {
//             let WithOffset { offset: off, value } = e;
//             WithOffset {
//                 offset: offset + off,
//                 value: Self::Error::FileOffset1(value),
//             }
//         })?;
//         offset += file_size;
//         Ok(WithSize {
//             value: Self { size, file },
//             size: offset,
//         })
//     }
// }
// #[test]
// fn test_non_void_query_operand() {
//     test_item(
//         NonVoid {
//             size: 4,
//             file: FileOffset { id: 5, offset: 6 },
//         },
//         &hex!("00 04  05 06"),
//     )
// }

// /// Compare file content, optionally masked, with 0.
// #[derive(Clone, Debug, PartialEq)]
// pub struct ComparisonWithZero {
//     pub signed_data: bool,
//     pub comparison_type: QueryComparisonType,
//     pub size: u32,
//     pub mask: Option<Box<[u8]>>,
//     pub file: FileOffset,
// }
// impl ComparisonWithZero {
//     pub fn validate(&self) -> Result<(), QueryValidationError> {
//         if self.size > varint::MAX {
//             return Err(QueryValidationError::SizeTooBig);
//         }
//         if let Some(mask) = &self.mask {
//             if mask.len() as u32 != self.size {
//                 return Err(QueryValidationError::BadMaskSize);
//             }
//         }
//         Ok(())
//     }
// }
// impl Codec for ComparisonWithZero {
//     type Error = QueryOperandDecodingError;
//     fn encoded_size(&self) -> usize {
//         let mask_size = match self.mask {
//             Some(_) => self.size as usize,
//             None => 0,
//         };
//         1 + unsafe { varint::size(self.size) } as usize + mask_size + self.file.encoded_size()
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         let mask_flag = match self.mask {
//             Some(_) => 1,
//             None => 0,
//         };
//         let signed_flag = if self.signed_data { 1 } else { 0 };
//         let mut offset = 0;
//         out[0] = ((QueryCode::ComparisonWithZero as u8) << 5)
//             | (mask_flag << 4)
//             | (signed_flag << 3)
//             | self.comparison_type as u8;
//         offset += 1;
//         offset += varint::encode_in(self.size, &mut out[offset..]) as usize;
//         if let Some(mask) = &self.mask {
//             out[offset..offset + (self.size as usize)].clone_from_slice(mask);
//             offset += mask.len();
//         }
//         offset += self.file.encode_in(&mut out[offset..]);
//         offset
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.len() < 1 + 1 + 2 {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(
//                 1 + 1 + 2 - out.len(),
//             )));
//         }
//         let mask_flag = out[0] & (1 << 4) != 0;
//         let signed_data = out[0] & (1 << 3) != 0;
//         let comparison_type = QueryComparisonType::from(out[0] & 0x07)
//             .map_err(|e| WithOffset::new_head(Self::Error::UnknownComparisonType(e)))?;
//         let WithSize {
//             value: size,
//             size: size_size,
//         } = varint::decode(&out[1..]).map_err(|e| {
//             let WithOffset { offset, value } = e;
//             WithOffset {
//                 offset: offset + 1,
//                 value: Self::Error::Size(value),
//             }
//         })?;
//         let mut offset = 1 + size_size;
//         let mask = if mask_flag {
//             let mut data = vec![0u8; size as usize].into_boxed_slice();
//             data.clone_from_slice(&out[offset..offset + size as usize]);
//             offset += size as usize;
//             Some(data)
//         } else {
//             None
//         };
//         let WithSize {
//             value: file,
//             size: offset_size,
//         } = FileOffset::decode(&out[offset..]).map_err(|e| {
//             let WithOffset { offset: off, value } = e;
//             WithOffset {
//                 offset: offset + off,
//                 value: Self::Error::FileOffset1(value),
//             }
//         })?;
//         offset += offset_size;
//         Ok(WithSize {
//             value: Self {
//                 signed_data,
//                 comparison_type,
//                 size,
//                 mask,
//                 file,
//             },
//             size: offset,
//         })
//     }
// }
// #[test]
// fn test_comparison_with_zero_operand() {
//     test_item(
//         ComparisonWithZero {
//             signed_data: true,
//             comparison_type: QueryComparisonType::Inequal,
//             size: 3,
//             mask: Some(vec![0, 1, 2].into_boxed_slice()),
//             file: FileOffset { id: 4, offset: 5 },
//         },
//         &hex!("38 03  000102  04 05"),
//     )
// }

// /// Compare some file content optionally masked, with a value
// #[derive(Clone, Debug, PartialEq)]
// pub struct ComparisonWithValue {
//     pub signed_data: bool,
//     pub comparison_type: QueryComparisonType,
//     pub size: u32,
//     pub mask: Option<Box<[u8]>>,
//     pub value: Box<[u8]>,
//     pub file: FileOffset,
// }
// impl ComparisonWithValue {
//     pub fn validate(&self) -> Result<(), QueryValidationError> {
//         let size = self.value.len();
//         if size as u32 > varint::MAX {
//             return Err(QueryValidationError::SizeTooBig);
//         }
//         if let Some(mask) = &self.mask {
//             if mask.len() != size {
//                 return Err(QueryValidationError::BadMaskSize);
//             }
//         }
//         Ok(())
//     }
// }
// impl Codec for ComparisonWithValue {
//     type Error = QueryOperandDecodingError;
//     fn encoded_size(&self) -> usize {
//         let mask_size = match self.mask {
//             Some(_) => self.size as usize,
//             None => 0,
//         };
//         1 + unsafe { varint::size(self.size) } as usize
//             + mask_size
//             + self.value.len()
//             + self.file.encoded_size()
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         let mask_flag = match self.mask {
//             Some(_) => 1,
//             None => 0,
//         };
//         let signed_flag = if self.signed_data { 1 } else { 0 };
//         let mut offset = 0;
//         out[0] = ((QueryCode::ComparisonWithValue as u8) << 5)
//             | (mask_flag << 4)
//             | (signed_flag << 3)
//             | self.comparison_type as u8;
//         offset += 1;
//         offset += varint::encode_in(self.size, &mut out[offset..]) as usize;
//         if let Some(mask) = &self.mask {
//             out[offset..offset + self.size as usize].clone_from_slice(mask);
//             offset += mask.len();
//         }
//         out[offset..offset + self.size as usize].clone_from_slice(&self.value[..]);
//         offset += self.value.len();
//         offset += self.file.encode_in(&mut out[offset..]);
//         offset
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.len() < 1 + 1 + 2 {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(
//                 1 + 1 + 2 - out.len(),
//             )));
//         }
//         let mask_flag = out[0] & (1 << 4) != 0;
//         let signed_data = out[0] & (1 << 3) != 0;
//         let comparison_type = QueryComparisonType::from(out[0] & 0x07)
//             .map_err(|e| WithOffset::new_head(Self::Error::UnknownComparisonType(e)))?;
//         let WithSize {
//             value: size,
//             size: size_size,
//         } = varint::decode(&out[1..]).map_err(|e| {
//             let WithOffset { offset, value } = e;
//             WithOffset {
//                 offset: offset + 1,
//                 value: Self::Error::Size(value),
//             }
//         })?;
//         let mut offset = 1 + size_size;
//         let mask = if mask_flag {
//             let mut data = vec![0u8; size as usize].into_boxed_slice();
//             data.clone_from_slice(&out[offset..offset + size as usize]);
//             offset += size as usize;
//             Some(data)
//         } else {
//             None
//         };
//         let mut value = vec![0u8; size as usize].into_boxed_slice();
//         value.clone_from_slice(&out[offset..offset + size as usize]);
//         offset += size as usize;
//         let WithSize {
//             value: file,
//             size: offset_size,
//         } = FileOffset::decode(&out[offset..]).map_err(|e| {
//             let WithOffset { offset: off, value } = e;
//             WithOffset {
//                 offset: offset + off,
//                 value: Self::Error::FileOffset1(value),
//             }
//         })?;
//         offset += offset_size;
//         Ok(WithSize {
//             value: Self {
//                 signed_data,
//                 comparison_type,
//                 size,
//                 mask,
//                 value,
//                 file,
//             },
//             size: offset,
//         })
//     }
// }
// #[test]
// fn test_comparison_with_value_operand() {
//     test_item(
//         ComparisonWithValue {
//             signed_data: false,
//             comparison_type: QueryComparisonType::Equal,
//             size: 3,
//             mask: None,
//             value: vec![9, 9, 9].into_boxed_slice(),
//             file: FileOffset { id: 4, offset: 5 },
//         },
//         &hex!("41 03   090909  04 05"),
//     )
// }

// /// Compare content of 2 files optionally masked
// #[derive(Clone, Debug, PartialEq)]
// pub struct ComparisonWithOtherFile {
//     pub signed_data: bool,
//     pub comparison_type: QueryComparisonType,
//     pub size: u32,
//     pub mask: Option<Box<[u8]>>,
//     pub file1: FileOffset,
//     pub file2: FileOffset,
// }
// impl ComparisonWithOtherFile {
//     pub fn validate(&self) -> Result<(), QueryValidationError> {
//         if self.size > varint::MAX {
//             return Err(QueryValidationError::SizeTooBig);
//         }
//         if let Some(mask) = &self.mask {
//             if mask.len() as u32 != self.size {
//                 return Err(QueryValidationError::BadMaskSize);
//             }
//         }
//         Ok(())
//     }
// }
// impl Codec for ComparisonWithOtherFile {
//     type Error = QueryOperandDecodingError;
//     fn encoded_size(&self) -> usize {
//         let mask_size = match self.mask {
//             Some(_) => self.size as usize,
//             None => 0,
//         };
//         1 + unsafe { varint::size(self.size) } as usize
//             + mask_size
//             + self.file1.encoded_size()
//             + self.file2.encoded_size()
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         let mask_flag = match self.mask {
//             Some(_) => 1,
//             None => 0,
//         };
//         let signed_flag = if self.signed_data { 1 } else { 0 };
//         let mut offset = 0;
//         out[0] = ((QueryCode::ComparisonWithOtherFile as u8) << 5)
//             | (mask_flag << 4)
//             | (signed_flag << 3)
//             | self.comparison_type as u8;
//         offset += 1;
//         offset += varint::encode_in(self.size, &mut out[offset..]) as usize;
//         if let Some(mask) = &self.mask {
//             out[offset..offset + self.size as usize].clone_from_slice(mask);
//             offset += mask.len();
//         }
//         offset += self.file1.encode_in(&mut out[offset..]);
//         offset += self.file2.encode_in(&mut out[offset..]);
//         offset
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.len() < 1 + 1 + 2 + 2 {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(
//                 1 + 1 + 2 + 2 - out.len(),
//             )));
//         }
//         let mask_flag = out[0] & (1 << 4) != 0;
//         let signed_data = out[0] & (1 << 3) != 0;
//         let comparison_type = QueryComparisonType::from(out[0] & 0x07)
//             .map_err(|e| WithOffset::new_head(Self::Error::UnknownComparisonType(e)))?;
//         let WithSize {
//             value: size,
//             size: size_size,
//         } = varint::decode(&out[1..]).map_err(|e| {
//             let WithOffset { offset: _, value } = e;
//             WithOffset {
//                 offset: 1,
//                 value: Self::Error::Size(value),
//             }
//         })?;
//         let mut offset = 1 + size_size;
//         let mask = if mask_flag {
//             let mut data = vec![0u8; size as usize].into_boxed_slice();
//             data.clone_from_slice(&out[offset..offset + size as usize]);
//             offset += size as usize;
//             Some(data)
//         } else {
//             None
//         };
//         let WithSize {
//             value: file1,
//             size: file1_size,
//         } = FileOffset::decode(&out[offset..]).map_err(|e| {
//             let WithOffset { offset: off, value } = e;
//             WithOffset {
//                 offset: offset + off,
//                 value: Self::Error::FileOffset1(value),
//             }
//         })?;
//         offset += file1_size;
//         let WithSize {
//             value: file2,
//             size: file2_size,
//         } = FileOffset::decode(&out[offset..]).map_err(|e| {
//             let WithOffset { offset: off, value } = e;
//             WithOffset {
//                 offset: offset + off,
//                 value: Self::Error::FileOffset2(value),
//             }
//         })?;
//         offset += file2_size;
//         Ok(WithSize {
//             value: Self {
//                 signed_data,
//                 comparison_type,
//                 size,
//                 mask,
//                 file1,
//                 file2,
//             },
//             size: offset,
//         })
//     }
// }
// #[test]
// fn test_comparison_with_other_file_operand() {
//     test_item(
//         ComparisonWithOtherFile {
//             signed_data: false,
//             comparison_type: QueryComparisonType::GreaterThan,
//             size: 2,
//             mask: Some(vec![0xFF, 0xFF].into_boxed_slice()),
//             file1: FileOffset { id: 4, offset: 5 },
//             file2: FileOffset { id: 8, offset: 9 },
//         },
//         &hex!("74 02 FFFF   04 05    08 09"),
//     )
// }

// /// Check if the content of a file is (not) contained in the sent bitmap values
// #[derive(Clone, Debug, PartialEq)]
// pub struct BitmapRangeComparison {
//     pub signed_data: bool,
//     pub comparison_type: QueryRangeComparisonType,
//     pub size: u32,
//     // ALP SPEC: In theory, start and stop can be huge array thus impossible to cast into any trivial
//     // number. How do we deal with this.
//     // If the max size is ever settled by the spec, replace the buffer by the max size. This may take up more
//     // memory, but would be way easier to use. Also it would avoid having to specify the ".size"
//     // field.
//     pub start: u32,
//     pub stop: u32,
//     pub bitmap: Box<[u8]>,
//     pub file: FileOffset,
// }
// impl BitmapRangeComparison {
//     pub fn validate(&self) -> Result<(), QueryValidationError> {
//         if self.start > self.stop {
//             return Err(QueryValidationError::StartGreaterThanStop);
//         }

//         let bitmap_size = (self.stop - self.start + 6) / 8; // ALP SPEC: Thanks for the calculation
//         if self.bitmap.len() != bitmap_size as usize {
//             return Err(QueryValidationError::BadMaskSize);
//         }
//         Ok(())
//     }
// }
// impl Codec for BitmapRangeComparison {
//     type Error = QueryOperandDecodingError;
//     fn encoded_size(&self) -> usize {
//         1 + unsafe { varint::size(self.size) } as usize
//             + 2 * self.size as usize
//             + self.bitmap.len()
//             + self.file.encoded_size()
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         let mut offset = 0;
//         let signed_flag = if self.signed_data { 1 } else { 0 };
//         out[0] = ((QueryCode::BitmapRangeComparison as u8) << 5)
//             // | (0 << 4)
//             | (signed_flag << 3)
//             | self.comparison_type as u8;
//         offset += 1;
//         offset += varint::encode_in(self.size, &mut out[offset..]) as usize;
//         let size = self.size as usize;
//         out[offset..offset + size].clone_from_slice(&self.start.to_be_bytes()[4 - size..]);
//         offset += size;
//         out[offset..offset + size].clone_from_slice(&self.stop.to_be_bytes()[4 - size..]);
//         offset += size;
//         out[offset..offset + self.bitmap.len()].clone_from_slice(&self.bitmap[..]);
//         offset += self.bitmap.len();
//         offset += self.file.encode_in(&mut out[offset..]);
//         offset
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.len() < 1 + 1 + 2 {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(
//                 1 + 1 + 2 - out.len(),
//             )));
//         }
//         let signed_data = out[0] & (1 << 3) != 0;
//         let comparison_type = QueryRangeComparisonType::from(out[0] & 0x07)
//             .map_err(|e| WithOffset::new_head(Self::Error::UnknownComparisonType(e)))?;
//         let WithSize {
//             value: size32,
//             size: size_size,
//         } = varint::decode(&out[1..]).map_err(|e| {
//             let WithOffset { offset, value } = e;
//             WithOffset {
//                 offset: offset + 1,
//                 value: Self::Error::Size(value),
//             }
//         })?;
//         let size = size32 as usize;
//         let mut offset = 1 + size_size;
//         let mut raw_start = vec![0u8; size].into_boxed_slice();
//         raw_start.clone_from_slice(&out[offset..offset + size]);
//         offset += size;
//         let mut raw_stop = vec![0u8; size].into_boxed_slice();
//         raw_stop.clone_from_slice(&out[offset..offset + size]);
//         offset += size;
//         // TODO Current max start/stop size chosen is u32 because that is the file size limit.
//         // But in theory there is no requirement for the bitmap to have any relation with the
//         // file sizes. So this might panic if you download your amazon bluerays over ALP.
//         let mut start = 0u32;
//         let mut stop = 0u32;
//         for i in 0..size {
//             start = (start << 8) + raw_start[i] as u32;
//             stop = (stop << 8) + raw_stop[i] as u32;
//         }
//         let bitmap_size = (stop - start + 6) / 8; // ALP SPEC: Thanks for the calculation
//         let mut bitmap = vec![0u8; bitmap_size as usize].into_boxed_slice();
//         bitmap.clone_from_slice(&out[offset..offset + bitmap_size as usize]);
//         offset += bitmap_size as usize;
//         let WithSize {
//             value: file,
//             size: file_size,
//         } = FileOffset::decode(&out[offset..]).map_err(|e| {
//             let WithOffset { offset: off, value } = e;
//             WithOffset {
//                 offset: offset + off,
//                 value: Self::Error::FileOffset1(value),
//             }
//         })?;
//         offset += file_size;
//         Ok(WithSize {
//             value: Self {
//                 signed_data,
//                 comparison_type,
//                 size: size32,
//                 start,
//                 stop,
//                 bitmap,
//                 file,
//             },
//             size: offset,
//         })
//     }
// }
// #[test]
// fn test_bitmap_range_comparison_operand() {
//     test_item(
//         BitmapRangeComparison {
//             signed_data: false,
//             comparison_type: QueryRangeComparisonType::InRange,
//             size: 2,

//             start: 3,
//             stop: 32,
//             bitmap: Box::new(hex!("01020304")),

//             file: FileOffset { id: 0, offset: 4 },
//         },
//         &hex!("81 02 0003  0020  01020304  00 04"),
//     )
// }

// /// Compare some file content, optional masked, with an array of bytes and up to a certain number
// /// of errors.
// #[derive(Clone, Debug, PartialEq)]
// pub struct StringTokenSearch {
//     pub max_errors: u8,
//     pub size: u32,
//     pub mask: Option<Box<[u8]>>,
//     pub value: Box<[u8]>,
//     pub file: FileOffset,
// }
// impl StringTokenSearch {
//     pub fn validate(&self) -> Result<(), QueryValidationError> {
//         if self.size > varint::MAX {
//             return Err(QueryValidationError::SizeTooBig);
//         }
//         if let Some(mask) = &self.mask {
//             if mask.len() as u32 != self.size {
//                 return Err(QueryValidationError::BadMaskSize);
//             }
//         }
//         Ok(())
//     }
// }
// impl Codec for StringTokenSearch {
//     type Error = QueryOperandDecodingError;
//     fn encoded_size(&self) -> usize {
//         let mask_size = match self.mask {
//             Some(_) => self.size as usize,
//             None => 0,
//         };
//         1 + unsafe { varint::size(self.size) } as usize
//             + mask_size
//             + self.value.len()
//             + self.file.encoded_size()
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         let mask_flag = match self.mask {
//             Some(_) => 1,
//             None => 0,
//         };
//         let mut offset = 0;
//         out[0] = ((QueryCode::StringTokenSearch as u8) << 5)
//             | (mask_flag << 4)
//             // | (0 << 3)
//             | self.max_errors;
//         offset += 1;
//         offset += varint::encode_in(self.size, &mut out[offset..]) as usize;
//         if let Some(mask) = &self.mask {
//             out[offset..offset + self.size as usize].clone_from_slice(mask);
//             offset += mask.len();
//         }
//         out[offset..offset + self.size as usize].clone_from_slice(&self.value[..]);
//         offset += self.value.len();
//         offset += self.file.encode_in(&mut out[offset..]);
//         offset
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.len() < 1 + 1 + 2 {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(
//                 1 + 1 + 2 - out.len(),
//             )));
//         }
//         let mask_flag = out[0] & (1 << 4) != 0;
//         let max_errors = out[0] & 0x07;
//         let WithSize {
//             value: size32,
//             size: size_size,
//         } = varint::decode(&out[1..]).map_err(|e| {
//             let WithOffset { offset, value } = e;
//             WithOffset {
//                 offset: offset + 1,
//                 value: Self::Error::Size(value),
//             }
//         })?;
//         let size = size32 as usize;
//         let mut offset = 1 + size_size;
//         let mask = if mask_flag {
//             let mut data = vec![0u8; size].into_boxed_slice();
//             data.clone_from_slice(&out[offset..offset + size]);
//             offset += size;
//             Some(data)
//         } else {
//             None
//         };
//         let mut value = vec![0u8; size].into_boxed_slice();
//         value.clone_from_slice(&out[offset..offset + size]);
//         offset += size;
//         let WithSize {
//             value: file,
//             size: offset_size,
//         } = FileOffset::decode(&out[offset..]).map_err(|e| {
//             let WithOffset { offset: off, value } = e;
//             WithOffset {
//                 offset: offset + off,
//                 value: Self::Error::FileOffset1(value),
//             }
//         })?;
//         offset += offset_size;
//         Ok(WithSize {
//             value: Self {
//                 max_errors,
//                 size: size32,
//                 mask,
//                 value,
//                 file,
//             },
//             size: offset,
//         })
//     }
// }
// #[test]
// fn test_string_token_search_operand() {
//     test_item(
//         StringTokenSearch {
//             max_errors: 2,
//             size: 4,
//             mask: Some(Box::new(hex!("FF00FF00"))),
//             value: Box::new(hex!("01020304")),
//             file: FileOffset { id: 0, offset: 4 },
//         },
//         &hex!("F2 04 FF00FF00  01020304  00 04"),
//     )
// }

/// The query operand provides a way to do optional actions. It represents a condition.
// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Debug, Clone, PartialEq)]
// #[deku(bits = 3, type = "u8")]
// pub enum Query {
//     NonVoid(NonVoid),
//     ComparisonWithZero(ComparisonWithZero),
//     ComparisonWithValue(ComparisonWithValue),
//     ComparisonWithOtherFile(ComparisonWithOtherFile),
//     BitmapRangeComparison(BitmapRangeComparison),
//     StringTokenSearch(StringTokenSearch),
// }

// impl Codec for Query {
//     type Error = QueryDecodingError;
//     fn encoded_size(&self) -> usize {
//         match self {
//             Query::NonVoid(v) => v.encoded_size(),
//             Query::ComparisonWithZero(v) => v.encoded_size(),
//             Query::ComparisonWithValue(v) => v.encoded_size(),
//             Query::ComparisonWithOtherFile(v) => v.encoded_size(),
//             Query::BitmapRangeComparison(v) => v.encoded_size(),
//             Query::StringTokenSearch(v) => v.encoded_size(),
//         }
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         match self {
//             Query::NonVoid(v) => v.encode_in(out),
//             Query::ComparisonWithZero(v) => v.encode_in(out),
//             Query::ComparisonWithValue(v) => v.encode_in(out),
//             Query::ComparisonWithOtherFile(v) => v.encode_in(out),
//             Query::BitmapRangeComparison(v) => v.encode_in(out),
//             Query::StringTokenSearch(v) => v.encode_in(out),
//         }
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         match QueryCode::from(out[0] >> 5)
//             .map_err(|e| WithOffset::new_head(Self::Error::UnknownQueryCode(e)))?
//         {
//             QueryCode::NonVoid => NonVoid::decode(out)
//                 .map(|ok| ok.map_value(Query::NonVoid))
//                 .map_err(|e| e.map_value(Self::Error::NonVoid)),
//             QueryCode::ComparisonWithZero => ComparisonWithZero::decode(out)
//                 .map(|ok| ok.map_value(Query::ComparisonWithZero))
//                 .map_err(|e| e.map_value(Self::Error::ComparisonWithZero)),
//             QueryCode::ComparisonWithValue => ComparisonWithValue::decode(out)
//                 .map(|ok| ok.map_value(Query::ComparisonWithValue))
//                 .map_err(|e| e.map_value(Self::Error::ComparisonWithValue)),
//             QueryCode::ComparisonWithOtherFile => ComparisonWithOtherFile::decode(out)
//                 .map(|ok| ok.map_value(Query::ComparisonWithOtherFile))
//                 .map_err(|e| e.map_value(Self::Error::ComparisonWithOtherFile)),
//             QueryCode::BitmapRangeComparison => BitmapRangeComparison::decode(out)
//                 .map(|ok| ok.map_value(Query::BitmapRangeComparison))
//                 .map_err(|e| e.map_value(Self::Error::BitmapRangeComparison)),
//             QueryCode::StringTokenSearch => StringTokenSearch::decode(out)
//                 .map(|ok| ok.map_value(Query::StringTokenSearch))
//                 .map_err(|e| e.map_value(Self::Error::StringTokenSearch)),
//         }
//     }
// }

/// Dash7 interface
// #[deku_derive(DekuRead, DekuWrite)]
// #[derive(Clone, Debug, PartialEq)]
// pub struct OverloadedIndirectInterface {
//     /// File containing the `QoS`, `to` and `te` to use for the transmission (see
//     /// dash7::InterfaceConfiguration
//     pub interface_file_id: u8,
//     pub nls_method: NlsMethod,
//     pub access_class: u8,
//     pub address: Address,
// }

// impl Codec for OverloadedIndirectInterface {
//     type Error = StdError;
//     fn encoded_size(&self) -> usize {
//         1 + 2 + self.address.encoded_size()
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         out[0] = self.interface_file_id;
//         out[1] = ((self.address.id_type() as u8) << 4) | (self.nls_method as u8);
//         out[2] = self.access_class;
//         1 + 2 + self.address.encode_in(&mut out[3..])
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.len() < 1 + 2 {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(
//                 1 + 2 - out.len(),
//             )));
//         }
//         let interface_file_id = out[0];
//         let address_type = dash7::AddressType::from((out[1] & 0x30) >> 4);
//         let nls_method = unsafe { dash7::NlsMethod::from(out[1] & 0x0F) };
//         let access_class = out[2];
//         let WithSize {
//             value: address,
//             size: address_size,
//         } = dash7::Address::parse(address_type, &out[3..]).map_err(|e| e.shift(3))?;
//         Ok(WithSize {
//             value: Self {
//                 interface_file_id,
//                 nls_method,
//                 access_class,
//                 address,
//             },
//             size: 1 + 2 + address_size,
//         })
//     }
// }
// #[test]
// fn test_overloaded_indirect_interface() {
//     test_item(
//         OverloadedIndirectInterface {
//             interface_file_id: 4,
//             nls_method: dash7::NlsMethod::AesCcm32,
//             access_class: 0xFF,
//             address: dash7::Address::Vid([0xAB, 0xCD]),
//         },
//         &hex!("04   37 FF ABCD"),
//     )
// }

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
        test_item(Length(1), &[0x01], (&[], 0));
        test_item(Length(65), &[0x40, 0x41], (&[], 0));
        test_item(Length(4263936), &[0xC0, 0x41, 0x10, 0x00], (&[], 0));
    }

    #[test]
    fn test_file_offset() {
        test_item(
            FileOffset {
                file_id: 2,
                offset: 0x3F_FFu32.into(),
            },
            &hex!("02 7F FF"),
            (&[], 0),
        )
    }
}
