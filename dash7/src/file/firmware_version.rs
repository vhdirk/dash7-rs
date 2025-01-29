use deku::prelude::*;

use crate::utils::{read_string, write_string};

// TODO: actual fixed length strings would be better here
#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Record)]
pub struct FirmwareVersionFile {
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
