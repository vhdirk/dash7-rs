use core::fmt::Debug;

use deku::prelude::*;
use deku::{DekuContainerRead, DekuContainerWrite};

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct WithPadding<T, const B: usize = 0, const A: usize = 0>(
    #[deku(pad_bits_before = "B", pad_bits_after = "A")] pub T,
)
where
    T: Clone + Debug + PartialEq + for<'a> DekuRead<'a> + DekuWrite;

pub fn test_item<T>(item: T, data: &[u8])
where
    T: Clone + Debug + PartialEq + DekuContainerWrite + for<'a> DekuContainerRead<'a>,
{
    let result = item.to_bytes().unwrap();
    // println!("{:?} == {:?}", BitVec::<u8, Msb0>::from_slice(&result), BitVec::<u8, Msb0>::from_slice(data));
    assert_eq!(result.as_slice(), data, "{:?} == {:?}", &item, data);

    let (_, result) = T::from_bytes((&data, 0)).expect("should be parsed without error");
    assert_eq!(result, item.clone(), "{:?} == {:?}", data, &item);
}
