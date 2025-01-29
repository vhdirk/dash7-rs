use super::{AlpFile, FileCtx};
use deku::{DekuRead, DekuWrite};


#[derive(DekuRead, DekuWrite)]
#[derive(Default, Debug, Clone, PartialEq, uniffi::Record)]
#[deku(ctx="ctx: FileCtx")]
pub struct OtherFile {
    #[deku(count="ctx.length")]
    pub data: Vec<u8>,
}

impl OtherFile {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data
        }
    }
}


impl AlpFile for OtherFile {}
