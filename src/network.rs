use packed_struct::prelude::PrimitiveEnum_u8;

use crate::types::CompressedValue;


/// Encryption algorithm for over-the-air packets
#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
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

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum AddressType {
    NbId,
    NoId,
    Uid,
    Vid,
}


pub struct Addressee {
    pub id_type: AddressType,
    pub access_class: u8,
    pub nls_method: NlsMethod,
    pub compressed_value: CompressedValue,
    pub id: long,

}