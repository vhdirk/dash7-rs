use core::slice;
use std::{
    mem::{self, MaybeUninit},
    ptr,
};

use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};
use deku::{DekuError, DekuRead, DekuWrite};

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

pub fn read_array<'a, T, const N: usize>(
    rest: &'a BitSlice<u8, Msb0>,
) -> Result<(&'a BitSlice<u8, Msb0>, [T; N]), DekuError>
where
    T: DekuRead<'a>,
{
    // lots of potentially unsafe operations here, but deemed safe anyway.
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
