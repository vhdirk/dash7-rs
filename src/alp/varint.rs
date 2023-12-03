use deku::{
    bitvec::{BitSlice, BitVec, BitView, Msb0},
    ctx::{BitSize, Endian},
    prelude::*,
};

use core::convert::TryFrom;
use core::ops::Deref;

#[cfg(not(feature = "std"))]
use alloc::fmt;

#[cfg(feature = "std")]
use std::fmt;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VarInt {
    value: u32,
    ceil: bool,
}

impl VarInt {
    pub const MAX: u32 = 507904;

    pub fn new(value: u32, ceil: bool) -> Self {
        // TODO: check bounds
        Self { value, ceil }
    }

    pub fn decompress(exponent: u8, mantissa: u8, ceil: bool) -> Self {
        // TODO: bounds checks on exp and mantissa
        Self {
            value: 4u32.pow(exponent as u32) * mantissa as u32,
            ceil,
        }
    }

    pub fn compress(&self) -> Result<(/*exponent: */ u8, /*mantissa: */ u8), ()> {
        if !Self::is_valid(self.value) {
            // TODO proper error
            return Err(());
        }

        for i in 0..8 {
            let exp = 4u32.pow(i);

            if self.value <= (exp * 31) {
                let mut mantissa = self.value / exp;
                let remainder = self.value % exp;

                if self.ceil && remainder > 0 {
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
}

impl Deref for VarInt {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DekuContainerRead<'_> for VarInt {
    fn from_bytes(input: (&'_ [u8], usize)) -> Result<((&'_ [u8], usize), Self), DekuError> {
        let input_bits = input.0.view_bits::<Msb0>();

        let (rest, this) = VarInt::read(&input_bits[input.1..], ())?;
        let pad = 8 * ((rest.len() + 7) / 8) - rest.len();
        let index = input_bits.len() - (rest.len() + pad);
        Ok((
            (input_bits[index..].domain().region().unwrap().1, pad),
            this,
        ))
    }
}
impl DekuRead<'_, ()> for VarInt {
    fn read(
        input: &'_ BitSlice<u8, Msb0>,
        _ctx: (),
    ) -> Result<(&'_ BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        let (rest, exponent) = <u8 as DekuRead<'_, _>>::read(input, (Endian::Big, BitSize(3)))?;
        let (rest, mantissa) = <u8 as DekuRead<'_, _>>::read(rest, (Endian::Big, BitSize(5)))?;
        Ok((rest, Self::decompress(exponent, mantissa, false)))
    }
}

impl TryFrom<&'_ [u8]> for VarInt {
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

impl DekuContainerWrite for VarInt {
    fn to_bytes(&self) -> Result<Vec<u8>, DekuError> {
        let acc: BitVec<u8, Msb0> = self.to_bits()?;
        Ok(acc.into_vec())
    }
    fn to_bits(&self) -> Result<BitVec<u8, Msb0>, DekuError> {
        let mut out: BitVec<u8, Msb0> = BitVec::new();
        self.write(&mut out, ())?;
        Ok(out)
    }
}
impl DekuUpdate for VarInt {
    fn update(&mut self) -> Result<(), DekuError> {
        Ok(())
    }
}
impl DekuWrite<()> for VarInt {
    fn write(&self, output: &mut BitVec<u8, Msb0>, _: ()) -> Result<(), DekuError> {
        match self.compress() {
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

impl TryFrom<VarInt> for BitVec<u8, Msb0> {
    type Error = DekuError;
    fn try_from(input: VarInt) -> Result<Self, Self::Error> {
        input.to_bits()
    }
}
impl TryFrom<VarInt> for Vec<u8> {
    type Error = DekuError;
    fn try_from(input: VarInt) -> Result<Self, Self::Error> {
        DekuContainerWrite::to_bytes(&input)
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
        assert_eq!(0, *VarInt::decompress(0, 0, false));
        assert_eq!(4, *VarInt::decompress(1, 1, false));
        assert_eq!(32, *VarInt::decompress(2, 2, false));
        assert_eq!(192, *VarInt::decompress(3, 3, false));
        assert_eq!(507904, *VarInt::decompress(7, 31, false));
    }

    #[test]
    fn test() {
        test_item(VarInt::new(0, false), &[0x00], &[]);
        test_item(VarInt::new(1, false), &[0x01u8], &[]);
        test_item(VarInt::new(32, false), &[0b00101000u8], &[]);
        test_item(VarInt::new(507904, false), &[0xFFu8], &[]);
    }


}
