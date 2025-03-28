use deku::prelude::*;

use crate::link::AccessProfile;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx = "id: u8")]
pub struct AccessProfileFile {
    #[deku(skip, default = "id")]
    pub specifier: u8,

    pub profile: AccessProfile,
}
