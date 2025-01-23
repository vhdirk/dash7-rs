use std::borrow::Cow;
use std::sync::Arc;

use deku::{
    ctx::{BitSize, Endian},
    no_std_io,
    prelude::*,
};

use uniffi;

#[derive(Debug, Clone, PartialEq, thiserror::Error, uniffi::Error)]
pub enum VarIntError {
    #[error("VarInt: Value too large: {value:?}. Max: {max:?}", max=VarInt::MAX)]
    ValueTooLarge { value: u32 },

    #[error("VarInt: Exponent too large: {exponent:?}. Max: {max:?}", max=VarInt::MAX_EXPONENT)]
    ExponentTooLarge { exponent: u8 },

    #[error("VarInt: Mantissa too large: {mantissa:?}. Max: {max:?}", max=VarInt::MAX_MANTISSA)]
    MantissaTooLarge { mantissa: u8 },

    #[error("VarInt: Unknown error")]
    Unknown,
}

impl Into<DekuError> for VarIntError {
    fn into(self) -> DekuError {
        match self {
            VarIntError::ValueTooLarge { .. }
            | VarIntError::ExponentTooLarge { .. }
            | VarIntError::MantissaTooLarge { .. } => {
                DekuError::InvalidParam(Cow::Owned(self.to_string()))
            }
            VarIntError::Unknown => DekuError::Parse(Cow::Owned(self.to_string())),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct VarIntParts {
    pub exponent: u8,
    pub mantissa: u8,
}

/// Variable int format
/// SPEC: 6.2.2 Compressed Format
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, Copy, PartialEq, uniffi::Object)]
pub struct VarInt {
    #[deku(
        reader = "VarInt::read(deku::reader)",
        writer = "VarInt::write(self, deku::writer)"
    )]
    value: u32,

    #[deku(skip, default = "false")]
    ceil: bool,
}

impl VarInt {
    // TODO: replace with calc
    pub const MAX: u32 = 507904;

    const EXPONENT_BITS: u8 = 3;
    const MANTISSA_BITS: u8 = 5;

    const MAX_EXPONENT: u8 = 2 ^ Self::EXPONENT_BITS;
    const MAX_MANTISSA: u8 = 2 ^ Self::MANTISSA_BITS;

    /// Returns whether the value is encodable into a VarInt or not.
    /// Makes no guarantees about precision
    pub fn is_valid(n: u32) -> bool {
        n <= Self::MAX
    }

    fn read<R>(reader: &mut Reader<R>) -> Result<u32, DekuError>
    where
        R: no_std_io::Read + no_std_io::Seek,
    {
        let exponent =
            <u8 as DekuReader<'_, _>>::from_reader_with_ctx(reader, (Endian::Big, BitSize(3)))?;
        let mantissa =
            <u8 as DekuReader<'_, _>>::from_reader_with_ctx(reader, (Endian::Big, BitSize(5)))?;

        Self::decompress(exponent, mantissa)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn write<W>(&self, writer: &mut Writer<W>) -> Result<(), DekuError>
    where
        W: no_std_io::Write + no_std_io::Seek,
    {
        match self.compress() {
            Ok(VarIntParts { exponent, mantissa }) => {
                DekuWriter::to_writer(&exponent, writer, (Endian::Big, BitSize(3)))?;
                DekuWriter::to_writer(&mantissa, writer, (Endian::Big, BitSize(5)))?;
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
    pub fn new(value: u32, ceil: bool) -> Result<Arc<Self>, VarIntError> {
        if !Self::is_valid(value) {
            Err(VarIntError::ValueTooLarge { value })
        } else {
            Ok(Arc::new(Self { value, ceil }))
        }
    }

    pub const fn decompress(exponent: u8, mantissa: u8) -> Result<Self, VarIntError> {
        if exponent & (1 << Self::EXPONENT_BITS) - 1 > 0 {
            return Err(VarIntError::ExponentTooLarge { exponent });
        }

        if mantissa & (1 << Self::MANTISSA_BITS) - 1 > 0 {
            return Err(VarIntError::MantissaTooLarge { mantissa });
        }

        Ok(Self::new_unchecked(
            4u32.pow(exponent as u32) * mantissa as u32,
            false,
        ))
    }
}

#[uniffi::export]
impl VarInt {
    #[uniffi::constructor]
    pub const fn new_unchecked(value: u32, ceil: bool) -> Self {
        Self { value, ceil }
    }

    #[uniffi::method]
    pub fn compress(&self) -> Result<VarIntParts, VarIntError> {
        for i in 0..8 {
            let exp = 4u32.pow(i);

            if self.value <= (exp * 31) {
                let mut mantissa = self.value / exp;
                let remainder = self.value % exp;

                if self.ceil && remainder > 0 {
                    mantissa += 1;
                }
                return Ok(VarIntParts {
                    exponent: i as u8,
                    mantissa: mantissa as u8,
                });
            }
        }

        // TODO when does this happen?
        Err(VarIntError::Unknown)
    }
}

impl Into<u32> for VarInt {
    fn into(self) -> u32 {
        self.value as u32
    }
}

impl From<u32> for VarInt {
    fn from(value: u32) -> Self {
        Self { value, ceil: false }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_tools::test_item;

    #[test]
    fn test_is_valid() {
        assert!(VarInt::is_valid(507904));
        assert!(!VarInt::is_valid(0x40_00_00_00));
    }

    #[test]
    fn test_decompress() {
        assert_eq!(0u32, (*VarInt::decompress(0, 0).unwrap()).into());
        assert_eq!(4u32, (*VarInt::decompress(1, 1).unwrap()).into());
        assert_eq!(32u32, (*VarInt::decompress(2, 2).unwrap()).into());
        assert_eq!(192u32, (*VarInt::decompress(3, 3).unwrap()).into());
        assert_eq!(507904u32, (*VarInt::decompress(7, 31).unwrap()).into());
    }

    #[test]
    fn test() {
        test_item(VarInt::default(), &[0x00]);
        test_item(*VarInt::new_unchecked(1, false), &[0x01u8]);
        test_item(*VarInt::new_unchecked(32, false), &[0b00101000u8]);
        test_item(*VarInt::new_unchecked(507904, false), &[0xFFu8]);
    }
}
