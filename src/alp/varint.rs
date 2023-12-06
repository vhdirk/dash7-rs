use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    ctx::{BitSize, Endian},
    prelude::*,
};

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
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

    pub fn new(value: u32, ceil: bool) -> Self {
        // TODO: check bounds
        Self { value, ceil }
    }

    pub fn decompress(exponent: u8, mantissa: u8) -> u32 {
        // TODO: bounds checks on exp and mantissa
        4u32.pow(exponent as u32) * mantissa as u32
    }

    pub fn compress(value: u32, ceil: bool) -> Result<(/*exponent: */ u8, /*mantissa: */ u8), ()> {
        if !Self::is_valid(value) {
            // TODO proper error
            return Err(());
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

        // TODO proper error
        Err(())
    }

    /// Returns whether the value is encodable into a varint or not.
    /// Makes no guarantees about precision
    pub fn is_valid(n: u32) -> bool {
        n <= Self::MAX
    }

    fn read(rest: &BitSlice<u8, Msb0>) -> Result<(&BitSlice<u8, Msb0>, u32), DekuError> {
        let (rest, exponent) = <u8 as DekuRead<'_, _>>::read(rest, (Endian::Big, BitSize(3)))?;
        let (rest, mantissa) = <u8 as DekuRead<'_, _>>::read(rest, (Endian::Big, BitSize(5)))?;
        Ok((rest, Self::decompress(exponent, mantissa)))
    }

    fn write(output: &mut BitVec<u8, Msb0>, value: &u32, ceil: &bool) -> Result<(), DekuError> {
        match Self::compress(*value, *ceil) {
            Ok((exponent, mantissa)) => {
                DekuWrite::write(&exponent, output, (Endian::Big, BitSize(3)))?;
                DekuWrite::write(&mantissa, output, (Endian::Big, BitSize(5)))?;
                Ok(())
            }
            Err(()) => Err(DekuError::Unexpected(
                "Could not compress value".to_string(),
            )),
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
        assert_eq!(0, VarInt::decompress(0, 0));
        assert_eq!(4, VarInt::decompress(1, 1));
        assert_eq!(32, VarInt::decompress(2, 2));
        assert_eq!(192, VarInt::decompress(3, 3));
        assert_eq!(507904, VarInt::decompress(7, 31));
    }

    #[test]
    fn test() {
        test_item(VarInt::default(), &[0x00]);
        test_item(VarInt::new(1, false), &[0x01u8]);
        test_item(VarInt::new(32, false), &[0b00101000u8]);
        test_item(VarInt::new(507904, false), &[0xFFu8]);
    }
}
