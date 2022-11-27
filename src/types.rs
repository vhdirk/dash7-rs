use std::fmt::Error;

use modular_bitfield::prelude::*;

#[bitfield]
#[derive(BitfieldSpecifier, Debug, PartialEq)]
pub struct CompressedValue {
    pub exponent: B3,
    pub mantissa: B5
}

impl CompressedValue {
    pub fn compress(value: u32, ceil: bool) -> Result<CompressedValue, Error> {
        for i in 0..8 {
            let exp = 4 ^ i;
            if value <= exp * 31 {
                let mut mantissa = value / exp;
                let remainder = value % exp;

                if ceil && remainder > 0 {
                    mantissa = mantissa + 1;
                }
                return Ok(CompressedValue::new().with_exponent(i as u8).with_mantissa(mantissa as u8));
            }
        }

        // TODO proper error
        return Err(Error{});
    }

    pub fn value(&self) -> u8{
        return ((self.exponent() << 5) | (self.mantissa() & 0x1F)) & 0xFF;
    }
}

