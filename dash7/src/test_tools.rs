use core::fmt::Debug;

use deku::prelude::*;

#[derive(DekuRead, DekuWrite, Debug, Clone, PartialEq)]
pub struct WithPadding<T, const B: usize = 0, const A: usize = 0>(
    #[deku(pad_bits_before = "B", pad_bits_after = "A")] pub T,
)
where
    T: Clone + Debug + PartialEq + for<'a> DekuReader<'a> + DekuWriter;

#[track_caller]
pub fn test_item<'a, T>(item: T, data: &'a [u8])
where
    T: Clone + Debug + PartialEq + TryFrom<&'a [u8]> + TryInto<Vec<u8>>,
    <T as TryFrom<&'a [u8]>>::Error: Debug,
    <T as TryInto<Vec<u8>>>::Error: Debug,
{
    let result: Vec<u8> = item.clone().try_into().unwrap();

    use deku::bitvec::{BitVec, Msb0};
    println!(
        "{:?} == {:?}",
        BitVec::<u8, Msb0>::from_slice(&result),
        BitVec::<u8, Msb0>::from_slice(data)
    );

    assert_eq!(result.as_slice(), data, "Serialize {:?} == {:?}", &item, data);

    let result = T::try_from(data).expect("should be parsed without error");
    assert_eq!(result, item.clone(), "Deserialize {:?} == {:?}", data, &item);
}
