use downcast_rs::{impl_downcast, DowncastSync};
use std::sync::Arc;

#[uniffi::export]
pub trait AlpFile: DowncastSync {}
impl_downcast!(AlpFile);

#[derive(thiserror::Error, Debug, strum::Display, uniffi::Error)]
pub enum FileRegistryError {
    FileNotFound { id: u8, offset: u32, length: u32 },
}

#[uniffi::export]
pub trait FileRegistry: DowncastSync {
    fn parse_file(
        &self,
        id: u8,
        offset: u32,
        data: Vec<u8>,
    ) -> Result<Arc<dyn AlpFile>, FileRegistryError>;
}
impl_downcast!(FileRegistry);

pub struct DefaultFileRegistry {}

impl DefaultFileRegistry {}

// impl FileRegistry for DefaultFileRegistry {
//     fn parse_file(&self,id:u8,offset:u32, data:Vec<u8>) -> Result<Arc<dyn File>, FileRegistryError> {

//         if offset > 0 {
//             return Err(FileRegistryError::FileNotFound { id: id, offset: offset, length: data.len() as u32 })
//         }

//         <SystemFile as DekuReader>::from_reader_with_ctx(reader, ctx)

//             Ok(Arc::new(GenericFile {
//                 id,
//                 offset,
//                 data
//             }))
//     }
// }

// impl TryFrom<u8> for FileId {
//     type Error = DekuError;

//     fn try_from(value: u8) -> Result<Self, Self::Error> {
//         Ok(Self::from_bytes((&vec![value], 0))?.1)
//     }
// }

// impl Into<u8> for FileId {
//     fn into(self) -> u8 {
//         self.deku_id().unwrap()
//     }
// }

// pub trait SystemFile {
//     const ID: u8;
//     const SIZE: u32;
// }

// impl Default for File {
//     fn default() -> Self {
//         Self::Other{file_id: 0xFF, buffer: vec![]}
//     }
// }

// impl DekuEnumExt<'_, FileId> for File {
//     fn deku_id(&self) -> Result<FileId, DekuError> {
//         match self {
//             File::AccessProfile{id, ..} => Ok(FileId::AccessProfile(id.clone())),
//             File::UId(_) => Ok(FileId::UId),
//             File::FactorySettings(_) => Ok(FileId::FactorySettings),
//             File::FirmwareVersion(_) => Ok(FileId::FirmwareVersion),
//             File::EngineeringMode(_) => Ok(FileId::EngineeringMode),
//             File::VId(_) => Ok(FileId::VId),
//             File::PhyStatus(_) => Ok(FileId::PhyStatus),
//             File::DllConfig(_) => Ok(FileId::DllConfig),
//             File::DllStatus(_) => Ok(FileId::DllStatus),
//             File::NetworkSecurityKey(_) => Ok(FileId::NetworkSecurityKey),
//             File::Other{file_id, ..} => Ok(FileId::Other(file_id.clone().into())),
//         }
//     }
// }

// impl DekuReader<'_, ()> for File {
//     #[inline]
//     fn from_reader_with_ctx<R: no_std_io::Read + no_std_io::Seek>(
//         reader: &mut Reader<R>,
//         ctx: (),
//     ) -> Result<Self, DekuError> {

//         let id = <FileId as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?;

//         // TODO: offset
//         let offset = <Length as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?;

//         let length = <Length as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?;

//         match id {
//             FileId::AccessProfile(id) =>
//                 Ok(File::AccessProfile{id: id - 0x20, file: <AccessProfileFile as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?})
//             ,

//             FileId::UId => Ok(File::UId( <Address as DekuReader<'_, _>>::from_reader_with_ctx(reader, AddressType::UId)? )),

//             FileId::FactorySettings => Ok(File::FactorySettings(<FactorySettings as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?)),
//             FileId::FirmwareVersion => Ok(File::FirmwareVersion(<FirmwareVersion as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?)),
//             FileId::EngineeringMode => Ok(File::EngineeringMode(<EngineeringMode as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?)),
//             FileId::VId => Ok(File::VId( <Address as DekuReader<'_, _>>::from_reader_with_ctx(reader, AddressType::VId)? )),

//             FileId::PhyStatus => Ok(File::PhyStatus(<PhyStatus as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?)),

//             FileId::DllConfig => Ok(File::DllConfig(<DllConfig as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?)),
//             FileId::DllStatus => Ok(File::DllStatus(<DllStatus as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?)),
//             FileId::NetworkSecurityKey => Ok(File::NetworkSecurityKey(<SecurityKey as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?)),
//             FileId::Other(file_id) => Ok(File::Other{file_id, buffer: <Vec<u8> as DekuReader<'_, _>>::from_reader_with_ctx(reader, Limit::new_count(length.into()))? }),
//             FileId::DeviceCapacity => todo!(),
//             FileId::DeviceStatus => todo!(),
//             FileId::PhyConfig => todo!(),
//             FileId::NetworkRouting => todo!(),
//             FileId::NetworkSecurity => todo!(),
//             FileId::NetworkSsr => todo!(),
//             FileId::NetworkStatus => todo!(),
//             FileId::TrlStatus => todo!(),
//             FileId::SelConfig => todo!(),
//             FileId::FofStatus => todo!(),
//             FileId::Rfu(_) => todo!(),
//             FileId::LocationData => todo!(),
//             FileId::RootKey => todo!(),
//             FileId::UserKey => todo!(),
//             FileId::SensorDescription => todo!(),
//             FileId::Rtc => todo!(),
//             FileId::D7AalpRfu(_) => todo!(),
//         }

//     }
// }

// impl DekuWriter<()> for File {
//     fn to_writer<W>(&self, writer: &mut Writer<W>, _: ()) -> Result<(), DekuError>
//     where
//         W: no_std_io::Write + no_std_io::Seek,
//     {

//         // first write the whole item into a byte buffer
//         let mut out_buf_cur = no_std_io::Cursor::new(Vec::new());
//         let mut tmp_writer = Writer::new(&mut out_buf_cur);

//         let file_id = match self {
//             File::AccessProfile { id, .. } =>
//                 *id + 0x20
//             ,
//             File::Other { file_id, .. } => *file_id,
//             _ => self.deku_id()?.deku_id()?
//         };

//         file_id.to_writer(writer, ())?;
//         Length(0).to_writer(writer, ())?;

//         match self {
//             File::AccessProfile { file, .. } => file.to_writer(&mut tmp_writer, ())? ,
//             File::UId(address) => address.to_writer(&mut tmp_writer, AddressType::UId)?,
//             File::FactorySettings(factory_settings) => factory_settings.to_writer(&mut tmp_writer, ())?,
//             File::FirmwareVersion(firmware_version) => firmware_version.to_writer(&mut tmp_writer, ())?,
//             File::EngineeringMode(engineering_mode) => engineering_mode.to_writer(&mut tmp_writer, ())?,
//             File::VId(address) => address.to_writer(&mut tmp_writer, AddressType::VId)?,
//             File::PhyStatus(phy_status) => phy_status.to_writer(&mut tmp_writer, ())?,
//             File::DllConfig(dll_config) => dll_config.to_writer(&mut tmp_writer, ())?,
//             File::DllStatus(dll_status) => dll_status.to_writer(&mut tmp_writer, ())?,
//             File::NetworkSecurityKey(security_key) => security_key.to_writer(&mut tmp_writer, ())?,
//             File::Other { buffer, .. } => buffer.to_writer(&mut tmp_writer, ())? ,
//         };

//         let _ = tmp_writer.finalize();

//         // get the length of it
//         let out_buf = out_buf_cur.get_mut();
//         let data_length: Length = out_buf.len().into();

//         // and then write them
//         data_length.to_writer(writer, ())?;
//         out_buf.to_writer(writer, ())?;

//         Ok(())
//     }
// }

// impl File {

//     // pub fn from_bytes<'a>(
//     //     input: (&'a [u8], usize),
//     //     file_id: FileId,
//     //     length: u32,
//     // ) -> Result<((&'a [u8], usize), Self), DekuError> {
//     //     from_bytes(input, (file_id, Length::default(), length.into()))
//     // }

//     // fn to_bytes(&self) -> Result<Vec<u8>, DekuError> {
//     //     let output = self.to_bits()?;
//     //     Ok(output.into_vec())
//     // }

//     // fn to_bits(&self) -> Result<BitVec<u8, Msb0>, DekuError> {
//     //     let mut output: BitVec<u8, Msb0> = BitVec::new();
//     //     self.write(&mut output, u32::MAX)?;
//     //     Ok(output)
//     // }
// }
