use deku::prelude::*;

use crate::link;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct AccessProfile<const S: u8> {
    pub profile: link::AccessProfile,
}

impl<const S: u8> AccessProfile<S> {
    pub fn specifier(&self) -> u8 {
        S
    }
}
