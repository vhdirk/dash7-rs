use core::fmt::Debug;

use deku::{DekuContainerRead, DekuContainerWrite};

pub fn test_item<T>(item: T, data: &[u8], (rest, offset): (&[u8], usize))
where
    T: Clone + Debug + PartialEq + DekuContainerWrite + for<'a> DekuContainerRead<'a>,
{
    let result = item.to_bytes().unwrap();
    assert_eq!(
        result.as_slice(),
        data,
        "{:?} <=> {:?}", &item, data
    );

    let ((result_rest, result_offset), result) = T::from_bytes((&data, 0)).expect("should be parsed without error");
    assert_eq!((result, result_rest, result_offset), (item.clone(), rest, offset),
        "{:?} <=> {:?}", data, &item
    );
}
