use deku::prelude::*;

mod addressee;
mod network;

pub use addressee::Addressee;
pub use network::{Control, Frame, HoppingControl};

use crate::types::VarInt;

/// Network Layer Security
/// SPEC: 7.4
#[derive(DekuRead, DekuWrite, Default, Debug, Copy, Clone, PartialEq)]
#[deku(bits = 3, type = "u8")]
pub enum NlsMethod {
    /// No security
    #[default]
    #[deku(id = "0x00")]
    None,

    /// Encryption only, Counter Mode
    #[deku(id = "0x01")]
    AesCtr,

    /// No encryption, Authentication, Cipher-block chaining with 128 bit MAC
    #[deku(id = "0x02")]
    AesCbcMac128,

    /// No encryption, Authentication, Cipher-block chaining with 64 bit MAC
    #[deku(id = "0x03")]
    AesCbcMac64,

    /// No encryption, Authentication, Cipher-block chaining with 32 bit MAC
    #[deku(id = "0x04")]
    AesCbcMac32,

    /// Authentication with CBC-MAC-128 and Encryption with Counter Mode
    #[deku(id = "0x05")]
    AesCcm128,

    /// Authentication with CBC-MAC-64 and Encryption with Counter Mode
    #[deku(id = "0x06")]
    AesCcm64,

    /// Authentication with CBC-MAC-32 and Encryption with Counter Mode
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
    #[deku(id = "0x00")]
    NbId,
    /// Broadcast to everyone
    #[default]
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
        Self::NoId
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    use crate::{link::AccessClass, test_tools::test_item, transport::GroupCondition};

    #[test]
    fn test_vid_aesccm32() {
        test_item(
            Addressee::new(
                false,
                GroupCondition::Any,
                Address::Vid(0xABCD),
                NlsState::AesCcm32(hex!("00 11 22 33 44")),
                AccessClass::new(0x0F, 0x0F),
            ),
            &hex!("37 FF ABCD 0011223344"),
        )
    }

    #[test]
    fn test_noid_none() {
        test_item(
            Addressee::new(
                false,
                GroupCondition::Any,
                Address::NoId,
                NlsState::None,
                AccessClass::default(),
            ),
            &[0b0010000, 0],
        );
    }

    #[test]
    fn test_nbid_none() {
        test_item(
            Addressee::new(
                false,
                GroupCondition::Any,
                Address::NbId(VarInt::new(0, false).unwrap()),
                NlsState::None,
                AccessClass::default(),
            ),
            &[0, 0, 0],
        );
    }

    #[test]
    fn test_uid_none() {
        test_item(
            Addressee::new(
                false,
                GroupCondition::Any,
                Address::Uid(0),
                NlsState::None,
                AccessClass::default(),
            ),
            &[0b00100000, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        );
    }

    #[test]
    fn test_nbid_aesctr() {
        test_item(
            Addressee::new(
                false,
                GroupCondition::Any,
                Address::NbId(VarInt::new(1, false).unwrap()),
                NlsState::AesCtr([0, 1, 2, 3, 4]),
                AccessClass::default(),
            ),
            &[0b00000001, 0, 1, 0, 1, 2, 3, 4],
        );
    }

    #[test]
    fn test_vid_none() {
        test_item(
            Addressee::new(
                false,
                GroupCondition::Any,
                Address::Vid(0x1234),
                NlsState::None,
                AccessClass::new(0, 5),
            ),
            &[0b00110000, 5, 0b00010010, 0b00110100],
        );
    }

    #[test]
    fn test_uid_none2() {
        test_item(
            Addressee::new(
                false,
                GroupCondition::Any,
                Address::Uid(0x1234567890123456),
                NlsState::None,
                AccessClass::new(0x06, 0x09),
            ),
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
                false,
                GroupCondition::Any,
                Address::NoId,
                NlsState::AesCbcMac128([10, 20, 30, 40, 50]),
                AccessClass::new(0x0B, 0x0E),
            ),
            &[0b00010010, 0xBE, 10, 20, 30, 40, 50],
        );
    }

    #[test]
    fn test_nbid_none2() {
        test_item(
            Addressee::new(
                false,
                GroupCondition::Any,
                Address::NbId(VarInt::new(100, false).unwrap()),
                NlsState::None,
                AccessClass::new(0, 0),
            ),
            &[0, 0, 0x39],
        );
    }
}
