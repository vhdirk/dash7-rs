use std::borrow::Cow;

use deku::{
    ctx::{BitSize, Endian},
    no_std_io,
    prelude::*,
};

use uniffi;

#[derive(Debug, Clone, PartialEq, strum::Display, uniffi::Error)]
pub enum VarIntError {
    ValueTooLarge(u32),
    ExponentTooLarge(u8),
    MantissaTooLarge(u8),
    Unknown,
}

impl Into<DekuError> for VarIntError {
    fn into(self) -> DekuError {
        match self {
            VarIntError::ValueTooLarge(value) => DekuError::InvalidParam(Cow::Owned(format!(
                "VarInt: Value too large: {:?}. Max: {:?}",
                value,
                VarInt::MAX
            ))),
            VarIntError::ExponentTooLarge(exponent) => {
                DekuError::InvalidParam(Cow::Owned(format!(
                    "VarInt: Exponent too large {:?}. Max: {:?}",
                    exponent,
                    2 ^ 3
                )))
            }
            VarIntError::MantissaTooLarge(mantissa) => {
                DekuError::InvalidParam(Cow::Owned(format!(
                    "VarInt: Mantissa too large {:?}. Max: {:?}",
                    mantissa,
                    2 ^ 5
                )))
            }
            VarIntError::Unknown => DekuError::Parse(Cow::Borrowed("VarInt: Unknown error")),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct FloatingPoint {
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

        Self::decompress(exponent, mantissa).map_err(Into::into)
    }

    fn write<W>(&self, writer: &mut Writer<W>) -> Result<(), DekuError>
    where
        W: no_std_io::Write + no_std_io::Seek,
    {
        match self.compress() {
            Ok(FloatingPoint { exponent, mantissa }) => {
                DekuWriter::to_writer(&exponent, writer, (Endian::Big, BitSize(3)))?;
                DekuWriter::to_writer(&mantissa, writer, (Endian::Big, BitSize(5)))?;
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
}

#[uniffi::export]
impl VarInt {
    #[uniffi::constructor]
    pub fn new(value: u32, ceil: bool) -> Result<Self, VarIntError> {
        if !Self::is_valid(value) {
            Err(VarIntError::ValueTooLarge(value))
        } else {
            Ok(Self { value, ceil })
        }
    }

    #[uniffi::constructor(name = "unchecked")]
    pub fn new_unchecked(value: u32, ceil: bool) -> Self {
        Self { value, ceil }
    }

    #[uniffi::constructor]
    pub fn decompress(exponent: u8, mantissa: u8) -> Result<u32, VarIntError> {
        if exponent & 0b11111000 > 0 {
            return Err(VarIntError::ExponentTooLarge(exponent));
        }

        if mantissa & 0b11100000 > 0 {
            return Err(VarIntError::ExponentTooLarge(mantissa));
        }

        Ok(4u32.pow(exponent as u32) * mantissa as u32)
    }

    pub fn compress(&self) -> Result<FloatingPoint, VarIntError> {
        for i in 0..8 {
            let exp = 4u32.pow(i);

            if self.value <= (exp * 31) {
                let mut mantissa = self.value / exp;
                let remainder = self.value % exp;

                if self.ceil && remainder > 0 {
                    mantissa += 1;
                }
                return Ok(FloatingPoint {
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
        assert_eq!(0, VarInt::decompress(0, 0).unwrap());
        assert_eq!(4, VarInt::decompress(1, 1).unwrap());
        assert_eq!(32, VarInt::decompress(2, 2).unwrap());
        assert_eq!(192, VarInt::decompress(3, 3).unwrap());
        assert_eq!(507904, VarInt::decompress(7, 31).unwrap());
    }

    #[test]
    fn test() {
        test_item(VarInt::default(), &[0x00]);
        test_item(VarInt::new_unchecked(1, false), &[0x01u8]);
        test_item(VarInt::new_unchecked(32, false), &[0b00101000u8]);
        test_item(VarInt::new_unchecked(507904, false), &[0xFFu8]);
    }
}
