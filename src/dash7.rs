


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

// /// Dash7 metadata upon packet reception.
// // ALP SPEC: Add link to D7a section (names do not even match)
// #[derive(Clone, Debug, PartialEq)]
// pub struct InterfaceStatus {
//     /// PHY layer channel header
//     pub ch_header: u8,
//     /// PHY layer channel index
//     pub ch_idx: u16,
//     /// PHY layer RX level in -dBm
//     pub rxlev: u8,
//     /// PHY layer link budget in dB
//     pub lb: u8,
//     /// Signal-to-noise Ratio (in dB)
//     pub snr: u8,
//     /// TODO
//     pub status: u8,
//     /// Value of the D7ATP Dialog ID
//     pub token: u8,
//     /// Value of the D7ATP Transaction ID
//     pub seq: u8,
//     // D7A SPEC: What is that?
//     /// Time during which the response is expected in Compressed Format
//     pub resp_to: u8,
//     // TODO Should I digress from the pure ALP description to restructure (addressee + nls_state)
//     // into a type protected NLS based structure? Maybe yes.
//     /// Listening access class of the sender
//     pub access_class: u8,
//     /// Address of source
//     pub address: Address,
//     /// Security data
//     pub nls_state: NlsState,
// }
// impl Codec for InterfaceStatus {
//     type Error = StdError;
//     fn encoded_size(&self) -> usize {
//         12 + self.address.encoded_size() + self.nls_state.encoded_size()
//     }

//     unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
//         let mut i = 0;
//         out[i] = self.ch_header;
//         i += 1;
//         out[i..(i + 2)].clone_from_slice(&self.ch_idx.to_be_bytes());
//         i += 2;
//         out[i] = self.rxlev;
//         i += 1;
//         out[i] = self.lb;
//         i += 1;
//         out[i] = self.snr;
//         i += 1;
//         out[i] = self.status;
//         i += 1;
//         out[i] = self.token;
//         i += 1;
//         out[i] = self.seq;
//         i += 1;
//         out[i] = self.resp_to;
//         i += 1;
//         out[i] = ((self.address.id_type() as u8) << 4) | (self.nls_state.method() as u8);
//         i += 1;
//         out[i] = self.access_class;
//         i += 1;
//         i += self.address.encode_in(&mut out[i..]);
//         if let Some(data) = self.nls_state.get_data() {
//             out[i..i + 5].clone_from_slice(&data[..]);
//             i += 5;
//         }
//         i
//     }
//     fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
//         if out.len() < 10 {
//             return Err(WithOffset::new_head(Self::Error::MissingBytes(
//                 10 - out.len(),
//             )));
//         }
//         let ch_header = out[0];
//         let ch_idx = ((out[1] as u16) << 8) + out[2] as u16;
//         let rxlev = out[3];
//         let lb = out[4];
//         let snr = out[5];
//         let status = out[6];
//         // TODO Bypass checks for faster parsing?
//         let token = out[7];
//         let seq = out[8];
//         let resp_to = out[9];

//         let address_type = AddressType::from((out[10] & 0x30) >> 4);
//         let nls_method = unsafe { NlsMethod::from(out[10] & 0x0F) };
//         let access_class = out[11];

//         let WithSize {
//             size: address_size,
//             value: address,
//         } = Address::parse(address_type, &out[12..]).map_err(|e| e.shift(12))?;

//         let mut offset = 12 + address_size;
//         let nls_state = match nls_method {
//             NlsMethod::None => NlsState::None,
//             method => {
//                 if out.len() < offset + 5 {
//                     return Err(WithOffset::new(
//                         offset,
//                         Self::Error::MissingBytes(offset + 5 - out.len()),
//                     ));
//                 } else {
//                     let mut nls_state = [0u8; 5];
//                     nls_state.clone_from_slice(&out[offset..offset + 5]);
//                     offset += 5;
//                     NlsState::build_non_none(method, nls_state)
//                 }
//             }
//         };
//         let size = offset;
//         Ok(WithSize {
//             value: Self {
//                 ch_header,
//                 ch_idx,
//                 rxlev,
//                 lb,
//                 snr,
//                 status,
//                 token,
//                 seq,
//                 resp_to,
//                 access_class,
//                 address,
//                 nls_state,
//             },
//             size,
//         })
//     }
// }
// #[test]
// fn test_interface_status() {
//     test_item(
//         InterfaceStatus {
//             ch_header: 1,
//             ch_idx: 0x0123,
//             rxlev: 2,
//             lb: 3,
//             snr: 4,
//             status: 5,
//             token: 6,
//             seq: 7,
//             resp_to: 8,
//             access_class: 0xFF,
//             address: Address::Vid([0xAB, 0xCD]),
//             nls_state: NlsState::AesCcm32(hex!("00 11 22 33 44")),
//         },
//         &hex!("01 0123 02 03 04 05 06 07 08   37 FF ABCD  0011223344"),
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
