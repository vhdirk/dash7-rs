use modular_bitfield::prelude::*;
use crate::types::CompressedValue;

/// Encryption algorithm for over-the-air packets
#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
#[bits = 4]
pub enum NlsMethod {
    None,
    AesCtr,
    AesCbcMac128,
    AesCbcMac64,
    AesCbcMac32,
    AesCcm128,
    AesCcm64,
    AesCcm32,
}

#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
#[bits = 2]
pub enum AddressType {
    NbId,
    NoId,
    Uid,
    Vid,
}

// #[derive(BitfieldSpecifier, Clone, Copy, Debug, PartialEq)]
pub enum Address {
    /// Broadcast to an estimated number of receivers, encoded in compressed format on a byte.
    NbId(u8),
    /// Broadcast to everyone
    NoId,
    /// Unicast to target via its UID (Unique Dash7 ID)
    Uid(u64),
    /// Unicast to target via its VID (Virtual ID)
    Vid(u16),
}

#[bitfield]
#[derive(BitfieldSpecifier, Clone, Copy, Debug, PartialEq)]
pub struct Addressee {
    // #[packed_field(bits = "0..=1", ty = "enum")]
    pub id_type: AddressType,

    pub nls_method: NlsMethod,

    pub access_class: B8,

    pub address: B10,
}
