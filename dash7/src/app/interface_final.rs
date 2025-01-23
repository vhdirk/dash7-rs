use deku::no_std_io::{Cursor, Read, Seek, Write};
use deku::prelude::*;

use super::action::OpCode;
use super::operation::{Length, Status};
use crate::session::{InterfaceFinalStatus, InterfaceTxStatus};

fn read_length_prefixed<'a, R, T, L, C>(reader: &mut Reader<R>, ctx: C) -> Result<T, DekuError>
where
    T: DekuReader<'a, (C, L)>,
    Length: Into<L>,
    R: Read + Seek,
{
    let length = <Length as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?;
    T::from_reader_with_ctx(reader, (ctx, length.into()))
}

fn write_length_prefixed<W, T, C>(writer: &mut Writer<W>, item: &T, ctx: C) -> Result<(), DekuError>
where
    T: DekuWriter<C>,
    W: Write + Seek,
{
    // first write the whole item into a byte buffer
    let mut out_buf_cur = Cursor::new(Vec::new());
    let mut tmp_writer = Writer::new(&mut out_buf_cur);
    let _ = item.to_writer(&mut tmp_writer, ctx)?;
    let _ = tmp_writer.finalize();

    // get the length of it
    let out_buf = out_buf_cur.get_mut();
    let data_length: Length = out_buf.len().into();

    // and then write them
    data_length.to_writer(writer, ())?;
    out_buf.to_writer(writer, ())?;

    Ok(())
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Object)]
pub struct InterfaceFinalStatusOperation {
    pub interface_id: u8,

    #[deku(
        reader = "InterfaceFinalStatus::read(deku::reader, *interface_id)",
        writer = "InterfaceFinalStatus::write(deku::writer, &self.status, self.interface_id)"
    )]
    pub status: InterfaceFinalStatus,
}

impl From<InterfaceFinalStatus> for InterfaceFinalStatusOperation {
    fn from(status: InterfaceFinalStatus) -> Self {
        Self {
            interface_id: status.deku_id().unwrap(),
            status,
        }
    }
}

impl Into<Status> for InterfaceFinalStatusOperation {
    fn into(self) -> Status {
        Status::InterfaceFinal(self).into()
    }
}

impl InterfaceFinalStatus {
    pub fn read<'a, R>(
        reader: &mut Reader<'a, R>,
        interface_id: u8,
    ) -> Result<InterfaceFinalStatus, DekuError>
    where
        R: Read + Seek,
    {
        read_length_prefixed(reader, interface_id)
    }

    pub fn write<W: Write + Seek>(
        writer: &mut Writer<W>,
        status: &InterfaceFinalStatus,
        interface_id: u8,
    ) -> Result<(), DekuError> {
        let vec_size = match status {
            InterfaceFinalStatus::Other(val) => val.len() as u32,
            _ => 0,
        };
        return write_length_prefixed(writer, status, (interface_id, vec_size));
    }
}

#[derive(DekuRead, DekuWrite, Clone, Copy, Debug, PartialEq, strum::Display, uniffi::Enum)]
#[deku(bits = 2, id_type = "u8")]
pub enum TxStatusType {
    #[deku(id = "1")]
    Interface,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Object)]
#[deku(ctx = "_opcode: OpCode")]
pub struct TxStatusOperation {
    #[deku(update = "self.status.deku_id().unwrap()", pad_bits_after = "6")]
    status_type: TxStatusType,

    #[deku(writer = "_opcode.to_writer(deku::writer, ())")]
    pub opcode: OpCode,

    #[deku(ctx = "*status_type")]
    pub status: TxStatus,
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, strum::Display, uniffi::Enum)]
#[deku(ctx = "status_type: TxStatusType", id = "status_type")]
pub enum TxStatus {
    #[deku(id = "TxStatusType::Interface")]
    Interface(InterfaceTxStatusOperation),
}

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq, uniffi::Object)]
pub struct InterfaceTxStatusOperation {
    pub interface_id: u8,

    #[deku(
        reader = "InterfaceTxStatus::read(deku::reader, *interface_id)",
        writer = "InterfaceTxStatus::write(deku::writer, &self.status, self.interface_id)"
    )]
    pub status: InterfaceTxStatus,
}

impl From<InterfaceTxStatus> for InterfaceTxStatusOperation {
    fn from(status: InterfaceTxStatus) -> Self {
        Self {
            interface_id: status.deku_id().unwrap(),
            status,
        }
    }
}

impl InterfaceTxStatus {
    pub fn read<'a, R>(
        reader: &mut Reader<'a, R>,
        interface_id: u8,
    ) -> Result<InterfaceTxStatus, DekuError>
    where
        R: Read + Seek,
    {
        return read_length_prefixed(reader, interface_id);
    }

    pub fn write<W: Write + Seek>(
        writer: &mut Writer<W>,
        status: &InterfaceTxStatus,
        interface_id: u8,
    ) -> Result<(), DekuError> {
        let vec_size = match status {
            InterfaceTxStatus::Other(val) => val.len() as u32,
            _ => 0,
        };
        return write_length_prefixed(writer, status, (interface_id, vec_size));
    }
}
