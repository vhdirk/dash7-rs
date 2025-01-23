use deku::prelude::*;

use crate::utils::{read_string, write_string};

use super::SystemFile;

// TODO: actual fixed length strings would be better here
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Record)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,

    #[deku(
        reader = "read_string::<_, 6>(deku::reader)",
        writer = "write_string::<_, 6>(deku::writer, &self.application_name)"
    )]
    pub application_name: String,

    #[deku(
        reader = "read_string::<_, 7>(deku::reader)",
        writer = "write_string::<_, 7>(deku::writer, &self.git_sha1)"
    )]
    pub git_sha1: String,
}

impl SystemFile for FirmwareVersion {
    const ID: u8 = 0x02;
    const SIZE: u32 = 17;
}
