use core::fmt;
use deku::no_std_io;
use deku::prelude::*;

mod access_profile;
mod address;
mod dll_config;
mod dll_status;
mod engineering_mode;
mod factory_settings;
mod firmware_version;
mod interface_configuration;
mod other;
mod phy_status;
mod security_key;
mod system;
mod traits;

pub use access_profile::AccessProfileFile;
pub use dll_config::DllConfigFile;
pub use dll_status::DllStatusFile;
pub use engineering_mode::{EngineeringModeFile, EngineeringModeMethod};
pub use factory_settings::FactorySettingsFile;
pub use firmware_version::FirmwareVersionFile;
pub use interface_configuration::InterfaceConfiguration;
pub use other::OtherFile;
pub use phy_status::PhyStatusFile;
pub use security_key::SecurityKeyFile;
pub use system::SystemFile;
pub use traits::*;

use crate::types::Length;
use crate::utils::from_bytes;
use crate::utils::write_length_prefixed;

#[derive(DekuRead, DekuWrite, Default, Debug, Clone, PartialEq, uniffi::Record)]
pub struct FileCtx {
    pub id: u8,
    pub offset: u32,
    pub length: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FileData<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    pub id: u8,

    pub offset: Length,

    pub file: File<F>,
}

impl<F> FileData<F>
where
    F: for<'a> DekuReader<'a, FileCtx> + DekuWriter<FileCtx> + fmt::Debug,
{
    pub fn from_bytes(input: (&'_ [u8], usize)) -> Result<((&'_ [u8], usize), Self), DekuError> {
        from_bytes(input, ())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum File<F = OtherFile>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    System(SystemFile),
    User(F),
    Other(OtherFile),
}

impl<F> Default for File<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    fn default() -> Self {
        Self::Other(OtherFile::default())
    }
}

impl<F> DekuReader<'_, FileCtx> for File<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    fn from_reader_with_ctx<R: no_std_io::Read + no_std_io::Seek>(
        reader: &mut Reader<R>,
        ctx: FileCtx,
    ) -> Result<Self, DekuError>
    where
        Self: Sized,
    {
        // first try userfiles
        let file = if let Ok(user_file) =
            <F as DekuReader<'_, _>>::from_reader_with_ctx(reader, ctx.clone())
        {
            File::User(user_file)
        }
        // then try systemfiles
        else if let Ok(system_file) =
            <SystemFile as DekuReader<'_, _>>::from_reader_with_ctx(reader, ctx.clone())
        {
            File::System(system_file)
        }
        // fallback in case user forgot
        else {
            let other_file =
                <OtherFile as DekuReader<'_, _>>::from_reader_with_ctx(reader, ctx.clone())?;
            File::Other(other_file)
        };
        Ok(file)
    }
}

impl<F> DekuWriter<FileCtx> for File<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    fn to_writer<W>(&self, writer: &mut Writer<W>, ctx: FileCtx) -> Result<(), DekuError>
    where
        W: no_std_io::Write + no_std_io::Seek,
    {
        match self {
            File::System(ref file) => file.to_writer(writer, ctx),
            File::User(ref file) => file.to_writer(writer, ctx),
            File::Other(ref file) => file.to_writer(writer, ctx),
        }
    }
}

impl<F> DekuReader<'_, ()> for FileData<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    #[inline]
    fn from_reader_with_ctx<R>(reader: &mut Reader<R>, _: ()) -> Result<Self, DekuError>
    where
        R: no_std_io::Read + no_std_io::Seek,
    {
        let id = <u8 as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?;

        let offset = <Length as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?;

        let length = <Length as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?;

        let file = <File<F> as DekuReader<'_, _>>::from_reader_with_ctx(
            reader,
            FileCtx {
                id,
                offset: offset.into(),
                length: length.into(),
            },
        )?;

        Ok(FileData { id, offset, file })
    }
}

impl<F> DekuWriter<()> for FileData<F>
where
    F: for<'f> DekuReader<'f, FileCtx> + DekuWriter<FileCtx>,
{
    fn to_writer<W>(&self, writer: &mut Writer<W>, _: ()) -> Result<(), DekuError>
    where
        W: no_std_io::Write + no_std_io::Seek,
    {
        self.id.to_writer(writer, ())?;

        self.offset.to_writer(writer, ())?;

        let ctx = FileCtx {
            id: self.id,
            offset: self.offset.into(),
            length: 0u32.into(),
        };

        write_length_prefixed(writer, &self.file, ctx)
    }
}

impl<F> File<F>
where
    F: for<'a> DekuReader<'a, FileCtx> + DekuWriter<FileCtx> + fmt::Debug,
{
    pub fn from_bytes(
        input: (&'_ [u8], usize),
        id: u8,
        offset: u32,
    ) -> Result<((&'_ [u8], usize), Self), DekuError> {
        from_bytes(
            input,
            FileCtx {
                id,
                offset,
                length: input.0.len() as u32,
            },
        )
    }
}
