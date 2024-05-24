use core::cmp;
use core::{
    mem::{self, MaybeUninit},
    ptr, slice,
};
use std::borrow::Cow;
use std::io::Write;

use deku::no_std_io;
use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    ctx::{ByteSize, Limit},
    prelude::*,
};

use crate::app::operand::Length;

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

pub fn pad_rest<'a>(
    input_bits: &'a BitSlice<u8, Msb0>,
    rest: &'a BitSlice<u8, Msb0>,
) -> (&'a [u8], usize) {
    let pad = 8 * ((rest.len() + 7) / 8) - rest.len();
    let read_idx = input_bits.len() - (rest.len() + pad);
    (input_bits[read_idx..].domain().region().unwrap().1, pad)
}

pub fn read_length_prefixed<'a, I, E, T, L, R>(
    reader: &mut Reader<R>,
    enum_id: I,
) -> Result<T, DekuError>
where
    T: DekuReader<'a, (E, L)>,
    E: TryFrom<I>,
    DekuError: From<<E as TryFrom<I>>::Error>,
    Length: Into<L>,
    R: no_std_io::Read,
{
    let length = <Length as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?;
    let enum_id = enum_id.try_into()?;
    T::from_reader_with_ctx(reader, (enum_id, Into::<L>::into(length)))
}

pub fn write_length_prefixed<W, I, E, T, L>(
    writer: &mut Writer<W>,
    item: &T,
    enum_id: I,
    fallback_length: L,
) -> Result<(), DekuError>
where
    T: DekuWriter<(E, L)>,
    W: no_std_io::Write,
    E: TryFrom<I>,
    DekuError: From<<E as TryFrom<I>>::Error>,
    L: Into<Length>,
{
    let enum_id = enum_id.try_into()?;

    // first write the whole item into a byte buffer
    let mut out_buf = Vec::new();
    let mut tmp_writer = Writer::new(&mut out_buf);
    item.to_writer(&mut tmp_writer, (enum_id, fallback_length))?;
    tmp_writer.finalize();

    // get the length of it
    let data_length: Length = out_buf.len().into();

    // and then write them
    data_length.to_writer(writer, ())?;
    out_buf.to_writer(writer, ())?;

    Ok(())
}

/// Read and convert to String
pub fn read_string<R, const N: usize>(reader: &mut Reader<R>) -> Result<String, DekuError>
where
    R: no_std_io::Read,
{
    let value = Vec::<u8>::from_reader_with_ctx(reader, Limit::new_byte_size(ByteSize(N)))?;

    String::from_utf8(value).map_err(|err| {
        DekuError::Parse(Cow::Owned(
            format!("Could not parse bytes into string {:?}", err).to_owned(),
        ))
    })
}

/// from String to [u8] and write
pub fn write_string<W, const N: usize>(
    writer: &mut Writer<W>,
    value: &str,
) -> Result<(), DekuError>
where
    W: no_std_io::Write,
{
    let mut bytes = [0u8; N];

    let max_index = cmp::min(value.len(), N);
    bytes[0..max_index].clone_from_slice(&value.as_bytes()[0..max_index]);

    DekuWriter::to_writer(&bytes.as_slice(), writer, ())
}

pub fn read_array<'a, R, T, const N: usize>(reader: &mut Reader<R>) -> Result<[T; N], DekuError>
where
    T: DekuReader<'a>,
    R: no_std_io::Read,
{
    // Potentially unsafe operations here, but deemed safe anyway.
    // We create an array of MaybeUninit. If deserializing an element would
    // error, all previously elements would leak.
    // Therefore, we add a transient dropper: it will keep track of everything
    // initialized so far, dropping all of it when it goes out of scope.
    // When all elements deserialized successfully, we just forget about the
    // transient dropper in whole

    let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

    let mut transient_dropper = TransientDropper {
        base_ptr: data.as_mut_ptr() as *mut T,
        initialized_count: 0,
    };

    for i in 0..N {
        let value = <T as DekuReader<'_, _>>::from_reader_with_ctx(reader, ())?;
        data[i].write(value);
        unsafe { transient_dropper.base_ptr = transient_dropper.base_ptr.add(1) };
        transient_dropper.initialized_count += 1;
    }

    mem::forget(transient_dropper);

    Ok(data.map(|elem: MaybeUninit<T>| unsafe { elem.assume_init() }))
}

pub fn write_array<W, T, const N: usize>(
    writer: &mut Writer<W>,
    value: &[T; N],
) -> Result<(), DekuError>
where
    T: DekuWriter,
    W: no_std_io::Write,
{
    for elem in value.iter() {
        DekuWriter::to_writer(elem, writer, ())?;
    }
    Ok(())
}
