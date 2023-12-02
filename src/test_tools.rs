use core::fmt::Debug;

use deku::{DekuContainerRead, DekuContainerWrite};

pub fn test_item<T>(item: T, data: &[u8], rest: &[u8])
where
    T: Clone + Debug + PartialEq + DekuContainerWrite + for<'a> DekuContainerRead<'a>,
{
    let result = item.to_bytes().unwrap();
    assert_eq!(
        result.as_slice(),
        data,
        "{:?} <=> {:?}", &item, data
    );

    assert_eq!(
        T::from_bytes((&data, 0)).expect("should be parsed without error"),
        ((rest, rest.len(),), item.clone()),
        "{:?} <=> {:?}", data, &item
    );
}
