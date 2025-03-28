use core::cmp;
use core::{ptr, slice};
use std::borrow::Cow;
use std::fmt::Debug;

use deku::{
    ctx::{ByteSize, Limit},
    no_std_io::{Cursor, Read, Seek, Write},
    prelude::*,
};

use crate::types::Length;

struct TransientDropper<T> {
    base_ptr: *mut T,
    initialized_count: usize,
}

impl<T> Drop for TransientDropper<T> {
    fn drop(self: &'_ mut Self) {
        unsafe {
            ptr::drop_in_place(slice::from_raw_parts_mut(
                self.base_ptr,
                self.initialized_count,
            ));
        }
    }
}

pub fn pad_rest<'a>(rest: (&'a [u8], usize), bits_read: usize) -> (&'a [u8], usize) {
    if (rest.0.len() * 8 + rest.1) <= bits_read {
        return (&[], bits_read % 8);
    }

    let read_whole_byte = (bits_read % 8) == 0;
    let idx = if read_whole_byte {
        bits_read / 8
    } else {
        (bits_read - (bits_read % 8)) / 8
    };
    (&rest.0[idx..], bits_read % 8)
}

pub fn from_reader<'a, R, T, Ctx>(
    input: (&'a mut R, usize),
    ctx: Ctx,
) -> Result<(usize, T), DekuError>
where
    T: DekuReader<'a, Ctx>,
    R: Read + Seek,
{
    let reader = &mut Reader::new(input.0);
    if input.1 != 0 {
        reader.skip_bits(input.1)?;
    }
    let value = T::from_reader_with_ctx(reader, ctx)?;
    Ok((reader.bits_read, value))
}

pub fn from_bytes<'a, T, Ctx>(
    input: (&'a [u8], usize),
    ctx: Ctx,
) -> Result<((&'a [u8], usize), T), DekuError>
where
    T: DekuReader<'a, Ctx> + Debug,
{
    let mut cursor = Cursor::new(input.0);
    let mut reader = &mut Reader::new(&mut cursor);
    if input.1 != 0 {
        reader.skip_bits(input.1)?;
    }
    let value = T::from_reader_with_ctx(&mut reader, ctx)?;
    println!(
        "value {:?}, reader.bits_read {:?}  {:?}",
        value, reader.bits_read, input
    );

    Ok((pad_rest(input, reader.bits_read), value))
}

pub fn write_length_prefixed<W, T, Ctx>(
    writer: &mut Writer<W>,
    item: &T,
    ctx: Ctx,
) -> Result<(), DekuError>
where
    T: DekuWriter<Ctx>,
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

/// Read and convert to String
pub fn read_string<R, const N: usize>(reader: &mut Reader<R>) -> Result<String, DekuError>
where
    R: Read + Seek,
{
    let value = Vec::<u8>::from_reader_with_ctx(reader, Limit::new_byte_size(ByteSize(N)))?;

    String::from_utf8(value).map_err(|err| {
        DekuError::Parse(Cow::Owned(
            format!("Could not parse bytes into string {:?}", err).to_owned(),
        ))
    })
}

/// from String to [u8] and write
pub fn write_string<W, const N: usize>(writer: &mut Writer<W>, value: &str) -> Result<(), DekuError>
where
    W: Write + Seek,
{
    let mut bytes = [0u8; N];

    let max_index = cmp::min(value.len(), N);
    bytes[0..max_index].clone_from_slice(&value.as_bytes()[0..max_index]);

    DekuWriter::to_writer(&bytes.as_slice(), writer, ())
}
