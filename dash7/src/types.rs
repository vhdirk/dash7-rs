use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    ctx::{BitSize, Endian},
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub enum VarIntError {
    ValueTooLarge(u32),
    ExponentTooLarge(u8),
    MantissaTooLarge(u8),
    Unknown,
}

impl Into<DekuError> for VarIntError {
    fn into(self) -> DekuError {
        match self {
            VarIntError::ValueTooLarge(value) => DekuError::InvalidParam(format!(
                "VarInt: Value too large: {:?}. Max: {:?}",
                value,
                VarInt::MAX
            )),
            VarIntError::ExponentTooLarge(exponent) => DekuError::InvalidParam(format!(
                "VarInt: Exponent too large {:?}. Max: {:?}",
                exponent,
                2 ^ 3
            )),
            VarIntError::MantissaTooLarge(mantissa) => DekuError::InvalidParam(format!(
                "VarInt: Mantissa too large {:?}. Max: {:?}",
                mantissa,
                2 ^ 5
            )),
            VarIntError::Unknown => DekuError::Unexpected("VarInt: Unknown error".to_string()),
        }
    }
}

/// Variable int format
/// SPEC: 6.2.2 Compressed Format
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, Copy, PartialEq)]
pub struct VarInt {
    #[deku(
        reader = "VarInt::read(deku::rest)",
        writer = "VarInt::write(deku::output, &self.value, &self.ceil)"
    )]
    value: u32,

    #[deku(skip, default = "false")]
    ceil: bool,
}

impl VarInt {
    pub const MAX: u32 = 507904;

    pub fn new(value: u32, ceil: bool) -> Result<Self, VarIntError> {
        if !Self::is_valid(value) {
            Err(VarIntError::ValueTooLarge(value))
        } else {
            Ok(Self { value, ceil })
        }
    }

    pub fn new_unchecked(value: u32, ceil: bool) -> Self {
        Self { value, ceil }
    }

    pub fn decompress(exponent: u8, mantissa: u8) -> Result<u32, VarIntError> {
        if exponent & 0b11111000 > 0 {
            return Err(VarIntError::ExponentTooLarge(exponent));
        }

        if mantissa & 0b11100000 > 0 {
            return Err(VarIntError::ExponentTooLarge(mantissa));
        }

        Ok(4u32.pow(exponent as u32) * mantissa as u32)
    }

    pub fn compress(
        value: u32,
        ceil: bool,
    ) -> Result<(/*exponent: */ u8, /*mantissa: */ u8), VarIntError> {
        if !Self::is_valid(value) {
            return Err(VarIntError::ValueTooLarge(value));
        }

        for i in 0..8 {
            let exp = 4u32.pow(i);

            if value <= (exp * 31) {
                let mut mantissa = value / exp;
                let remainder = value % exp;

                if ceil && remainder > 0 {
                    mantissa += 1;
                }
                return Ok((i as u8, mantissa as u8));
            }
        }

        // TODO when does this happen?
        Err(VarIntError::Unknown)
    }

    /// Returns whether the value is encodable into a VarInt or not.
    /// Makes no guarantees about precision
    pub fn is_valid(n: u32) -> bool {
        n <= Self::MAX
    }

    fn read(rest: &BitSlice<u8, Msb0>) -> Result<(&BitSlice<u8, Msb0>, u32), DekuError> {
        let (rest, exponent) = <u8 as DekuRead<'_, _>>::read(rest, (Endian::Big, BitSize(3)))?;
        let (rest, mantissa) = <u8 as DekuRead<'_, _>>::read(rest, (Endian::Big, BitSize(5)))?;

        Self::decompress(exponent, mantissa)
            .map_err(Into::into)
            .map(|value| (rest, value))
    }

    fn write(output: &mut BitVec<u8, Msb0>, value: &u32, ceil: &bool) -> Result<(), DekuError> {
        match Self::compress(*value, *ceil) {
            Ok((exponent, mantissa)) => {
                DekuWrite::write(&exponent, output, (Endian::Big, BitSize(3)))?;
                DekuWrite::write(&mantissa, output, (Endian::Big, BitSize(5)))?;
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
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
