use deku::prelude::*;

use super::Length;

/// Describe the location of some data on the filesystem (file + data offset).
#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq)]
pub struct FileOffset {
    pub file_id: u8,
    pub offset: Length,
}

impl FileOffset {
    pub fn no_offset(file_id: u8) -> Self {
        Self {
            file_id,
            offset: 0u32.into(),
        }
    }
}
