use packed_struct::prelude::*;

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct CompressedValue {
    #[packed_field(bits = "0..=2",endian="msb")]
    pub exponent: Integer<u8, packed_bits::Bits<3>>,

    #[packed_field(bits = "3..=7",endian="msb")]
    pub mantisse: Integer<u8, packed_bits::Bits<5>>,
}