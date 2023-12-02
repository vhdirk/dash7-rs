use deku::prelude::*;

use crate::varint::VarInt;

/// Encryption algorithm for over-the-air packets
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(ctx = "nls_method: u8", id = "nls_method")]
pub enum NlsState {
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
#[deku(ctx = "address_type: u8", id = "address_type" )]
pub enum Address {
    /// Broadcast to an estimated number of receivers, encoded in compressed format on a byte.
    #[deku(id = "0x00")]
    NbId(VarInt),
    /// Broadcast to everyone
    #[deku(id = "0x01")]
    NoId,
    /// Unicast to target via its UID (Unique Dash7 ID)
    #[deku(id = "0x02")]
    Uid(#[deku(endian="big")]u64),
    /// Unicast to target via its VID (Virtual ID)
    #[deku(id = "0x03")]
    Vid(#[deku(endian="big")]u16),
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
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

}
