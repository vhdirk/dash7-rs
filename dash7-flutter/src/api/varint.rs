pub use dash7::types::{self, VarIntError};
use flutter_rust_bridge::frb;

#[frb(opaque)]
pub struct VarInt(pub types::VarInt);

impl From<types::VarInt> for VarInt {
    fn from(value: types::VarInt) -> Self {
        VarInt(value)
    }
}

impl Into<types::VarInt> for VarInt {
    fn into(self) -> types::VarInt {
        self.0
    }
}


impl VarInt {
    #[frb(sync)]
    pub fn new(value: u32, ceil: bool) -> Result<Self, VarIntError> {
        Ok(types::VarInt::new(value, ceil)?.into())
    }

    #[frb(sync, getter)]
    pub fn value(&self) -> u32 {
         self.0.into()
    }

    #[frb(sync)]
    pub fn decompress(exponent: u8, mantissa: u8) -> Result<VarInt, VarIntError> {
        Ok(types::VarInt::new(types::VarInt::decompress(exponent, mantissa)?, false)?.into())
    }
}
