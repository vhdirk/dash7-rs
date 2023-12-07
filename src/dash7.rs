


// #[derive(Debug, Copy, Clone, Hash, PartialEq)]
// pub enum InterfaceConfigurationDecodingError {
//     MissingBytes(usize),
//     Qos(QosDecodingError),
// }

// impl From<StdError> for InterfaceConfigurationDecodingError {
//     fn from(e: StdError) -> Self {
//         match e {
//             StdError::MissingBytes(n) => Self::MissingBytes(n),
//         }
//     }
// }

// impl Codec for InterfaceConfiguration {
//     type Error = InterfaceConfigurationDecodingError;
//     fn encoded_size(&self) -> usize {
//         self.qos.encoded_size() + 4 + self.address.encoded_size()
//     }
//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         self.qos.encode_in(out);
//         out[1] = self.to;
//         out[2] = self.te;
//         out[3] = ((self.address.id_type() as u8) << 4) | (self.nls_method as u8);
//         out[4] = self.access_class;
//         5 + self.address.encode_in(&mut out[5..])
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.len() < 5 {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(
//                 5 - out.len(),
//             )));
//         }
//         let WithSize {
//             value: qos,
//             size: qos_size,
//         } = Qos::decode(out).map_err(|e| e.map_value(Self::Error::Qos))?;
//         let to = out[1];
//         let te = out[2];
//         let address_type = AddressType::from((out[3] & 0x30) >> 4);
//         let nls_method = unsafe { NlsMethod::from(out[3] & 0x0F) };
//         let access_class = out[4];
//         let WithSize {
//             value: address,
//             size: address_size,
//         } = Address::parse(address_type, &out[5..]).map_err(|e| {
//             let WithOffset { offset, value } = e;
//             WithOffset {
//                 offset: offset + 5,
//                 value: value.into(),
//             }
//         })?;
//         Ok(WithSize {
//             value: Self {
//                 qos,
//                 to,
//                 te,
//                 access_class,
//                 nls_method,
//                 address,
//             },
//             size: qos_size + 4 + address_size,
//         })
//     }
// }
// #[test]
// fn test_interface_configuration() {
//     test_item(
//         InterfaceConfiguration {
//             qos: Qos {
//                 retry: RetryMode::No,
//                 resp: RespMode::Any,
//             },
//             to: 0x23,
//             te: 0x34,
//             nls_method: NlsMethod::AesCcm32,
//             access_class: 0xFF,
//             address: Address::Vid([0xAB, 0xCD]),
//         },
//         &hex!("02 23 34   37 FF ABCD"),
//     )
// }

// #[test]
// fn test_interface_configuration_with_address_nbid() {
//     test_item(
//         InterfaceConfiguration {
//             qos: Qos {
//                 retry: RetryMode::No,
//                 resp: RespMode::Any,
//             },
//             to: 0x23,
//             te: 0x34,
//             nls_method: NlsMethod::None,
//             access_class: 0x00,
//             address: Address::NbId(0x15),
//         },
//         &hex!("02 23 34   00 00 15"),
//     )
// }
// #[test]
// fn test_interface_configuration_with_address_noid() {
//     test_item(
//         InterfaceConfiguration {
//             qos: Qos {
//                 retry: RetryMode::No,
//                 resp: RespMode::Any,
//             },
//             to: 0x23,
//             te: 0x34,
//             nls_method: NlsMethod::AesCbcMac128,
//             access_class: 0x24,
//             address: Address::NoId,
//         },
//         &hex!("02 23 34   12 24"),
//     )
// }
// #[test]
// fn test_interface_configuration_with_address_uid() {
//     test_item(
//         InterfaceConfiguration {
//             qos: Qos {
//                 retry: RetryMode::No,
//                 resp: RespMode::Any,
//             },
//             to: 0x23,
//             te: 0x34,
//             nls_method: NlsMethod::AesCcm64,
//             access_class: 0x48,
//             address: Address::Uid([0, 1, 2, 3, 4, 5, 6, 7]),
//         },
//         &hex!("02 23 34   26 48 0001020304050607"),
//     )
// }
// #[test]
// fn test_interface_configuration_with_address_vid() {
//     test_item(
//         InterfaceConfiguration {
//             qos: Qos {
//                 retry: RetryMode::No,
//                 resp: RespMode::Any,
//             },
//             to: 0x23,
//             te: 0x34,
//             nls_method: NlsMethod::AesCcm32,
//             access_class: 0xFF,
//             address: Address::Vid([0xAB, 0xCD]),
//         },
//         &hex!("02 23 34   37 FF AB CD"),
//     )
// }



// pub mod file {
//     pub mod id {
//         //! File IDs 0x00-0x17 and 0x20-0x2F are reserved by the DASH7 spec.
//         //! File IDs 0x18-0x1F Reserved for D7AALP.
//         //! File IDs 0x20+I with I in [0, 14] are reserved for Access Profiles.
//         pub const UID: u8 = 0x00;
//         pub const FACTORY_SETTINGS: u8 = 0x01;
//         pub const FIRMWARE_VERSIOR: u8 = 0x02;
//         pub const DEVICE_CAPACITY: u8 = 0x03;
//         pub const DEVICE_STATUS: u8 = 0x04;
//         pub const ENGINEERING_MODE: u8 = 0x05;
//         pub const VID: u8 = 0x06;
//         pub const PHY_CONFIGURATION: u8 = 0x08;
//         pub const PHY_STATUS: u8 = 0x09;
//         pub const DLL_CONFIGURATION: u8 = 0x0A;
//         pub const DLL_STATUS: u8 = 0x0B;
//         pub const NWL_ROUTING: u8 = 0x0C;
//         pub const NWL_SECURITY: u8 = 0x0D;
//         pub const NWL_SECURITY_KEY: u8 = 0x0E;
//         pub const NWL_SECURITY_STATE_REGISTER: u8 = 0x0F;
//         pub const NWL_STATUS: u8 = 0x10;
//         pub const TRL_STATUS: u8 = 0x11;
//         pub const SEL_CONFIGURATION: u8 = 0x12;
//         pub const FOF_STATUS: u8 = 0x13;
//         pub const LOCATION_DATA: u8 = 0x17;
//         pub const ROOT_KEY: u8 = 0x18;
//         pub const USER_KEY: u8 = 0x19;
//         pub const SENSOR_DESCRIPTION: u8 = 0x1B;
//         pub const RTC: u8 = 0x1C;
//     }
//     // TODO Write standard file structs
// }
