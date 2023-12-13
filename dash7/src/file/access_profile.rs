use deku::prelude::*;

use crate::link;

use super::SystemFile;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct AccessProfile<const S: u8> {
    pub profile: link::AccessProfile,
}

impl<const S: u8> AccessProfile<S> {
    pub fn specifier(&self) -> u8 {
        S
    }
}

impl<const S: u8> SystemFile for AccessProfile<S> {
    const ID: u8 = 0x20 + S;
    const SIZE: u32 = 0;
}
