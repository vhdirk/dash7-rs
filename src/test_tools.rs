// #[cfg(not(feature = "std"))]
// use alloc::fmt;

// #[cfg(feature = "std")]
// use std::fmt::Debug;

use core::fmt::Debug;

use deku::{DekuContainerRead, DekuContainerWrite};

pub fn test_item<T>(item: T, data: &[u8], rest: &[u8], descr: &str)
where
    T: Debug + PartialEq + DekuContainerWrite + for<'a> DekuContainerRead<'a>,
{
    let result = item.to_bytes().unwrap();
    assert_eq!(result.as_slice(), data, "{} | Left: item::to_bytes, Right: expected data", descr);

    assert_eq!(
        T::from_bytes((&data, 0)).expect("should be parsed without error"),
        ((rest, rest.len(),), item)
    );
}
