use deku::prelude::*;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Record)]
pub struct FactorySettingsFile {
    pub gain: i8,
    #[deku(endian = "big")]
    pub rx_bw_low_rate: u32,
    #[deku(endian = "big")]
    pub rx_bw_normal_rate: u32,
    #[deku(endian = "big")]
    pub rx_bw_high_rate: u32,
    #[deku(endian = "big")]
    pub bitrate_lo_rate: u32,
    #[deku(endian = "big")]
    pub fdev_lo_rate: u32,
    #[deku(endian = "big")]
    pub bitrate_normal_rate: u32,
    #[deku(endian = "big")]
    pub fdev_normal_rate: u32,
    #[deku(endian = "big")]
    pub bitrate_hi_rate: u32,
    #[deku(endian = "big")]
    pub fdev_hi_rate: u32,

    pub preamble_size_lo_rate: u8,
    pub preamble_size_normal_rate: u8,
    pub preamble_size_hi_rate: u8,

    pub preamble_detector_size_lo_rate: u8,
    pub preamble_detector_size_normal_rate: u8,
    pub preamble_detector_size_hi_rate: u8,

    pub preamble_tol_lo_rate: u8,
    pub preamble_tol_normal_rate: u8,
    pub preamble_tol_hi_rate: u8,

    pub rssi_smoothing: u8,
    pub rssi_offset: u8,

    #[deku(endian = "big")]
    pub lora_bw: u32,
    pub lora_sf: u8,

    pub gaussian: u8,
    #[deku(endian = "big")]
    pub paramp: u16,
}
