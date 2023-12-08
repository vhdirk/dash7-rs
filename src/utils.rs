use core::{
    mem::{self, MaybeUninit},
    ptr, slice,
};
use core::cmp;

use deku::{
    prelude::*,
    bitvec::{BitSlice, BitVec, Msb0}, ctx::{ByteSize, Limit},
};

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

/// Read and convert to String
pub fn read_string<const N: usize>(
    rest: &BitSlice<u8, Msb0>,
) -> Result<(&BitSlice<u8, Msb0>, String), DekuError> {
    let (rest, value) = Vec::<u8>::read(rest, Limit::new_byte_size(ByteSize(N)))?;

    String::from_utf8(value).map_err(|err| {
        DekuError::Parse(format!("Could not parse bytes into string {:?}", err))
    }).map(|value| (rest, value))
}

/// from String to [u8] and write
pub fn write_string<const N: usize>(output: &mut BitVec<u8, Msb0>, value: &str) -> Result<(), DekuError> {
    let mut bytes = [0u8;N];

    let max_index = cmp::min(value.len(), N);
    bytes[0..max_index].clone_from_slice(&value.as_bytes()[0..max_index]);

    DekuWrite::write(&bytes.as_slice(), output, ())
}


pub fn read_array<'a, T, const N: usize>(
    rest: &'a BitSlice<u8, Msb0>,
) -> Result<(&'a BitSlice<u8, Msb0>, [T; N]), DekuError>
where
    T: DekuRead<'a>,
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

    let mut rest = rest;

    for i in 0..N {
        let (new_rest, value) = <T as DekuRead<'_, _>>::read(rest, ())?;
        rest = new_rest;
        data[i].write(value);
        unsafe { transient_dropper.base_ptr = transient_dropper.base_ptr.add(1) };
        transient_dropper.initialized_count += 1;
    }

    mem::forget(transient_dropper);

    Ok((
        rest,
        data.map(|elem: MaybeUninit<T>| unsafe { elem.assume_init() }),
    ))
}

pub fn write_array<T, const N: usize>(
    output: &mut BitVec<u8, Msb0>,
    value: &[T; N],
) -> Result<(), DekuError>
where
    T: DekuWrite,
{
    for elem in value.iter() {
        DekuWrite::write(elem, output, ())?;
    }
    Ok(())
}
