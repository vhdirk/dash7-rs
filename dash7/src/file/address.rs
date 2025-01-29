use deku::prelude::*;

use crate::network::{Address, AddressType};

use super::AlpFile;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "address_type: AddressType")]
pub struct AddressFile {
    #[deku(ctx = "address_type")]
    pub address: Address,
}

impl AlpFile for AddressFile {}
