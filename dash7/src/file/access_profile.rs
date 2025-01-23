use deku::prelude::*;

use crate::link;

use super::SystemFile;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Record)]
pub struct AccessProfileFile<const S: u8 = 0> {
    pub profile: link::AccessProfile,
}

impl<const S: u8> AccessProfileFile<S> {
    pub fn specifier(&self) -> u8 {
        S
    }
}

impl<const S: u8> SystemFile for AccessProfileFile<S> {
    const ID: u8 = 0x20 + S;
    const SIZE: u32 = 0;
}
