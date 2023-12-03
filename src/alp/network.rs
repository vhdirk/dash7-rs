use deku::prelude::*;

use crate::alp::varint::VarInt;

/// Encryption algorithm for over-the-air packets
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
#[deku(ctx = "nls_method: u8", id = "nls_method")]
#[repr(u8)]
pub enum NlsState {
    #[default]
    #[deku(id = "0x00")]
    None,
    #[deku(id = "0x01")]
    AesCtr([u8; 5]),
    #[deku(id = "0x02")]
    AesCbcMac128([u8; 5]),
    #[deku(id = "0x03")]
    AesCbcMac64([u8; 5]),
    #[deku(id = "0x04")]
    AesCbcMac32([u8; 5]),
    #[deku(id = "0x05")]
    AesCcm128([u8; 5]),
    #[deku(id = "0x06")]
    AesCcm64([u8; 5]),
    #[deku(id = "0x07")]
    AesCcm32([u8; 5]),
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(ctx = "address_type: u8", id = "address_type")]
#[repr(u8)]
pub enum Address {
    /// Broadcast to an estimated number of receivers, encoded in compressed format on a byte.
    #[deku(id = "0x00")]
    NbId(VarInt),
    /// Broadcast to everyone
    #[deku(id = "0x01")]
    NoId,
    /// Unicast to target via its UID (Unique Dash7 ID)
    #[deku(id = "0x02")]
    Uid(#[deku(endian = "big")] u64),
    /// Unicast to target via its VID (Virtual ID)
    #[deku(id = "0x03")]
    Vid(#[deku(endian = "big")] u16),
}

impl Default for Address {
    fn default() -> Self {
        Self::NbId(VarInt::default())
    }
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Addressee {
    #[deku(pad_bits_before = "2", bits = 2)]
    address_type: u8,

    #[deku(bits = 4)]
    nls_method: u8,

    pub access_class: u8,

    #[deku(ctx = "*address_type")]
    pub address: Address,

    #[deku(ctx = "*nls_method")]
    pub nls_state: NlsState,
}

impl Addressee {
    pub fn new(address: Address, nls_state: NlsState, access_class: u8) -> Self {
        Self {
            address_type: address.deku_id().unwrap(),
            nls_method: nls_state.deku_id().unwrap(),
            access_class,
            address,
            nls_state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    use crate::test_tools::test_item;

    #[test]
    fn test_adressee() {
        test_item(
            Addressee::new(
                Address::Vid(0xABCD),
                NlsState::AesCcm32(hex!("00 11 22 33 44")),
                0xFF,
            ),
            &hex!("37 FF ABCD 0011223344"),
            &[],
        )
    }

    #[test]
    fn test() {
        test_item(
            Addressee::new(Address::NoId, NlsState::None, 0),
            &[0b00010000, 0],
            &[],
        );

        test_item(
            Addressee::new(Address::NbId(VarInt::new(0, false)), NlsState::None, 0),
            &[0, 0, 0],
            &[],
        );

        test_item(
            Addressee::new(Address::Uid(0), NlsState::None, 0),
            &[0b00100000, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            &[],
        );

        test_item(
            Addressee::new(
                Address::NbId(VarInt::new(1, false)),
                NlsState::AesCtr([0, 1, 2, 3, 4]),
                0,
            ),
            &[0b00000001, 0, 1, 0, 1, 2, 3, 4],
            &[],
        );

        test_item(
            Addressee::new(Address::Vid(0x1234), NlsState::None, 5),
            &[0b00110000, 5, 0b00010010, 0b00110100],
            &[],
        );

        test_item(
            Addressee::new(Address::Uid(0x1234567890123456), NlsState::None, 105),
            &[
                0b00100000, 105, 0b00010010, 0b00110100, 0b01010110, 0b01111000, 0b10010000,
                0b00010010, 0b00110100, 0b01010110,
            ],
            &[],
        );

        test_item(
            Addressee::new(
                Address::NoId,
                NlsState::AesCbcMac128([10, 20, 30, 40, 50]),
                0xBE,
            ),
            &[0b00010010, 0xBE, 10, 20, 30, 40, 50],
            &[],
        );

        test_item(
            Addressee::new(Address::NbId(VarInt::new(100, false)), NlsState::None, 0),
            &[0, 0, 0x39],
            &[],
        );
    }
}
