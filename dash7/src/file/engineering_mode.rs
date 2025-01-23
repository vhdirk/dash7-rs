use deku::prelude::*;

use crate::physical::Channel;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
#[deku(id_type = "u8")]
pub enum EngineeringModeMethod {
    #[deku(id = "0")]
    Off,
    #[deku(id = "1")]
    ContTx,
    #[deku(id = "2")]
    TransientTx,
    #[deku(id = "3")]
    PerRx,
    #[deku(id = "4")]
    PerTx,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Record)]
pub struct EngineeringMode {
    pub mode: EngineeringModeMethod,
    pub flags: u8,
    pub timeout: u8,
    pub channel: Channel,
    pub eirp: i8,
}
