use deku::prelude::*;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Record)]
pub struct SecurityKeyFile {
    #[deku(count="4")]
    pub key: Vec<u32>,
}
