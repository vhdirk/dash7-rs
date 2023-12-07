use deku::prelude::*;

use crate::alp::varint::VarInt;

/// Network Layer Security
#[derive(DekuRead, DekuWrite, Default, Debug, Copy, Clone, PartialEq)]
#[deku(bits = 4, type = "u8")]
pub enum NlsMethod {
    #[default]
    #[deku(id = "0x00")]
    None,
    #[deku(id = "0x01")]
    AesCtr,
    #[deku(id = "0x02")]
    AesCbcMac128,
    #[deku(id = "0x03")]
    AesCbcMac64,
    #[deku(id = "0x04")]
    AesCbcMac32,
    #[deku(id = "0x05")]
    AesCcm128,
    #[deku(id = "0x06")]
    AesCcm64,
    #[deku(id = "0x07")]
    AesCcm32,
}

/// Encryption algorithm for over-the-air packets
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
#[deku(ctx = "nls_method: NlsMethod", id = "nls_method")]
pub enum NlsState {
    #[default]
    #[deku(id = "NlsMethod::None")]
    None,
    #[deku(id = "NlsMethod::AesCtr")]
    AesCtr([u8; 5]),
    #[deku(id = "NlsMethod::AesCbcMac128")]
    AesCbcMac128([u8; 5]),
    #[deku(id = "NlsMethod::AesCbcMac64")]
    AesCbcMac64([u8; 5]),
    #[deku(id = "NlsMethod::AesCbcMac32")]
    AesCbcMac32([u8; 5]),
    #[deku(id = "NlsMethod::AesCcm128")]
    AesCcm128([u8; 5]),
    #[deku(id = "NlsMethod::AesCcm64")]
    AesCcm64([u8; 5]),
    #[deku(id = "NlsMethod::AesCcm32")]
    AesCcm32([u8; 5]),
}

#[derive(DekuRead, DekuWrite, Default, Debug, Copy, Clone, PartialEq)]
#[deku(bits = 2, type = "u8")]
pub enum AddressType {
    /// Broadcast to an estimated number of receivers, encoded in compressed format on a byte.
    #[default]
    #[deku(id = "0x00")]
    NbId,
    /// Broadcast to everyone
    #[deku(id = "0x01")]
    NoId,
    /// Unicast to target via its UID (Unique Dash7 ID)
    #[deku(id = "0x02")]
    Uid,
    /// Unicast to target via its VID (Virtual ID)
    #[deku(id = "0x03")]
    Vid,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
#[deku(ctx = "address_type: AddressType", id = "address_type")]
pub enum Address {
    /// Broadcast to an estimated number of receivers, encoded in compressed format on a byte.
    #[deku(id = "AddressType::NbId")]
    NbId(VarInt),
    /// Broadcast to everyone
    #[deku(id = "AddressType::NoId")]
    NoId,
    /// Unicast to target via its UID (Unique Dash7 ID)
    #[deku(id = "AddressType::Uid")]
    Uid(#[deku(endian = "big")] u64),
    /// Unicast to target via its VID (Virtual ID)
    #[deku(id = "AddressType::Vid")]
    Vid(#[deku(endian = "big")] u16),
}

impl Default for Address {
    fn default() -> Self {
        Self::NbId(VarInt::default())
    }
}

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct Addressee {
    #[deku(update = "self.address.deku_id().unwrap()", pad_bits_before = "2")]
    address_type: AddressType,

    #[deku(update = "self.nls_state.deku_id().unwrap()")]
    nls_method: NlsMethod,

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
    fn test_vid_aesccm32() {
        test_item(
            Addressee::new(
                Address::Vid(0xABCD),
                NlsState::AesCcm32(hex!("00 11 22 33 44")),
                0xFF,
            ),
            &hex!("37 FF ABCD 0011223344"),
        )
    }

    #[test]
    fn test_noid_none() {
        test_item(
            Addressee::new(Address::NoId, NlsState::None, 0),
            &[0b0010000, 0],
        );
    }

    #[test]
    fn test_nbid_none() {
        test_item(
            Addressee::new(Address::NbId(VarInt::new(0, false).unwrap()), NlsState::None, 0),
            &[0, 0, 0],
        );
    }

    #[test]
    fn test_uid_none() {
        test_item(
            Addressee::new(Address::Uid(0), NlsState::None, 0),
            &[0b00100000, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        );
    }

    #[test]
    fn test_nbid_aesctr() {
        test_item(
            Addressee::new(
                Address::NbId(VarInt::new(1, false).unwrap()),
                NlsState::AesCtr([0, 1, 2, 3, 4]),
                0,
            ),
            &[0b00000001, 0, 1, 0, 1, 2, 3, 4],
        );
    }

    #[test]
    fn test_vid_none() {
        test_item(
            Addressee::new(Address::Vid(0x1234), NlsState::None, 5),
            &[0b00110000, 5, 0b00010010, 0b00110100],
        );
    }

    #[test]
    fn test_uid_none2() {
        test_item(
            Addressee::new(Address::Uid(0x1234567890123456), NlsState::None, 105),
            &[
                0b00100000, 105, 0b00010010, 0b00110100, 0b01010110, 0b01111000, 0b10010000,
                0b00010010, 0b00110100, 0b01010110,
            ],
        );
    }

    #[test]
    fn test_noid_aescbcmac128() {
        test_item(
            Addressee::new(
                Address::NoId,
                NlsState::AesCbcMac128([10, 20, 30, 40, 50]),
                0xBE,
            ),
            &[0b00010010, 0xBE, 10, 20, 30, 40, 50],
        );
    }

    #[test]
    fn test_nbid_none2() {
        test_item(
            Addressee::new(Address::NbId(VarInt::new(100, false).unwrap()), NlsState::None, 0),
            &[0, 0, 0x39],
        );
    }
}
