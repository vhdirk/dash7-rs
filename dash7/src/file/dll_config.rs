use deku::prelude::*;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Record)]
pub struct DllConfig {
    pub ac: u8,
    #[deku(pad_bits_before = "16")]
    pub lq_filter: u8,
    pub nf_ctrl: u8,
    pub rx_nf_method_parameter: u8,
    pub tx_nf_method_parameter: u8,
}
