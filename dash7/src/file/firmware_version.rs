use deku::prelude::*;

use crate::utils::{read_string, write_string};

use super::SystemFile;

// TODO: fixed length strings would be better here
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,

    #[deku(
        reader = "read_string::<6>(deku::rest)",
        writer = "write_string::<6>(deku::output, &self.application_name)"
    )]
    pub application_name: String,

    #[deku(
        reader = "read_string::<7>(deku::rest)",
        writer = "write_string::<7>(deku::output, &self.git_sha1)"
    )]
    pub git_sha1: String,
}

impl SystemFile for FirmwareVersion {
    const ID: u8 = 0x02;
    const SIZE: u32 = 17;
}
