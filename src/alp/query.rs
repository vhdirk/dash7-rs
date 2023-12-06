use deku::prelude::*;

use super::operand::{FileOffset, Length};

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 3, type = "u8")]
pub enum ArithmeticComparisonType {
    #[deku(id = "0")]
    Inequal,
    #[deku(id = "1")]
    Equal,
    #[deku(id = "2")]
    LessThan,
    #[deku(id = "3")]
    LessThanOrEqual,
    #[deku(id = "4")]
    GreaterThan,
    #[deku(id = "5")]
    GreaterThanOrEqual,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ArithmeticQueryParams {
    #[deku(bits = 1)]
    pub signed: bool,
    pub comparison_type: ArithmeticComparisonType,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 3, type = "u8")]
pub enum RangeComparisonType {
    #[deku(id = "0")]
    NotInRange,
    #[deku(id = "1")]
    InRange,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct RangeQueryParams {
    #[deku(bits = 1)]
    pub signed: bool,
    pub comparison_type: RangeComparisonType,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 3, type = "u8")]
pub enum Query {
    #[deku(id = "0x00")]
    NonVoid(NonVoid),
    #[deku(id = "0x01")]
    ComparisonWithZero(ComparisonWithZero),
    #[deku(id = "0x02")]
    ComparisonWithValue(ComparisonWithValue),
    #[deku(id = "0x03")]
    ComparisonWithOtherFile(ComparisonWithOtherFile),
    #[deku(id = "0x04")]
    BitmapRangeComparison(BitmapRangeComparison),
    #[deku(id = "0x07")]
    StringTokenSearch(StringTokenSearch),
}

// ALP_SPEC Does this fail if the content overflows the file?
/// Checks if the file content exists.
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct NonVoid {
    #[deku(pad_bits_before = "5")]
    pub length: Length,
    pub file: FileOffset,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ComparisonWithZero {
    #[deku(bits = 1, update = "self.mask.len() > 0")]
    mask_present: bool,

    pub params: ArithmeticQueryParams,

    #[deku(update = "self.mask.len()")]
    length: Length,

    #[deku(cond = "*mask_present", count = "length", endian = "big")]
    pub mask: Vec<u8>,
    pub file: FileOffset,
}

impl ComparisonWithZero {
    pub fn new(params: ArithmeticQueryParams, mask: Vec<u8>, file: FileOffset) -> Self {
        Self {
            mask_present: mask.len() > 0,
            params,
            length: mask.len().into(),
            mask,
            file,
        }
    }
}

/// Compare some file content optionally masked, with a value
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ComparisonWithValue {
    #[deku(bits = 1, update = "self.mask.len() > 0")]
    mask_present: bool,

    pub params: ArithmeticQueryParams,

    #[deku(update = "self.value.len()")]
    length: Length,

    #[deku(cond = "*mask_present", count = "length", endian = "big")]
    pub mask: Vec<u8>,

    #[deku(count = "length", endian = "big")]
    pub value: Vec<u8>,

    pub file: FileOffset,
}

impl ComparisonWithValue {
    pub fn new(
        params: ArithmeticQueryParams,
        mask: Vec<u8>,
        value: Vec<u8>,
        file: FileOffset,
    ) -> Self {
        Self {
            mask_present: mask.len() > 0,
            params,
            length: value.len().into(),
            mask,
            value,
            file,
        }
    }
}

/// Compare content of 2 files optionally masked
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct ComparisonWithOtherFile {
    #[deku(bits = 1, update = "self.mask.len() > 0")]
    mask_present: bool,
    pub params: ArithmeticQueryParams,

    pub length: Length,
    #[deku(cond = "*mask_present", count = "length", endian = "big")]
    pub mask: Vec<u8>,

    pub file1: FileOffset,
    pub file2: FileOffset,
}

impl ComparisonWithOtherFile {
    pub fn new(
        params: ArithmeticQueryParams,
        mask: Vec<u8>,
        file1: FileOffset,
        file2: FileOffset,
    ) -> Self {
        Self {
            mask_present: mask.len() > 0,
            params,
            length: mask.len().into(),
            mask,
            file1,
            file2,
        }
    }
}

/// Check if the content of a file is (not) contained in the sent bitmap values
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct BitmapRangeComparison {
    #[deku(bits = 1, update = "self.mask.len() > 0")]
    mask_present: bool,
    pub params: RangeQueryParams,
    pub length: Length,
    // ALP SPEC: In theory, start and stop can be huge array thus impossible to cast into any trivial
    // number. How do we deal with this.
    // If the max size is ever settled by the spec, replace the buffer by the max size. This may take up more
    // memory, but would be way easier to use. Also it would avoid having to specify the ".size"
    // field.
    pub start: Length,
    pub stop: Length,

    #[deku(count = "length", endian = "big")]
    pub mask: Vec<u8>,
    pub file: FileOffset,
}

impl BitmapRangeComparison {
    pub fn new(
        params: RangeQueryParams,
        start: u32,
        stop: u32,
        mask: Vec<u8>,
        file: FileOffset,
    ) -> Self {
        Self {
            mask_present: mask.len() > 0,
            params,
            length: mask.len().into(),
            start: start.into(),
            stop: stop.into(),
            mask,
            file,
        }
    }
}

/// Compare some file content, optional masked, with an array of bytes and up to a certain number
/// of errors.
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct StringTokenSearch {
    #[deku(bits = 1, update = "self.mask.len() > 0", pad_bits_after = "1")]
    mask_present: bool,

    // TODO: is this bitsize correct?
    #[deku(bits = 3)]
    pub max_errors: u8,

    #[deku(update = "self.value.len()")]
    pub length: Length,

    #[deku(count = "length", endian = "big")]
    pub mask: Vec<u8>,

    #[deku(count = "length", endian = "big")]
    pub value: Vec<u8>,

    pub file: FileOffset,
}

impl StringTokenSearch {
    pub fn new(max_errors: u8, mask: Vec<u8>, value: Vec<u8>, file: FileOffset) -> Self {
        Self {
            mask_present: mask.len() > 0,
            max_errors,
            length: value.len().into(),
            mask,
            value,
            file,
        }
    }
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use crate::test_tools::test_item;

    use super::*;

    #[test]
    fn test_query_non_void() {
        test_item(
            Query::NonVoid(NonVoid {
                length: 4u32.into(),
                file: FileOffset {
                    file_id: 5,
                    offset: 6u32.into(),
                },
            }),
            &hex!("00 04  05 06"),
        )
    }

    #[test]
    fn test_query_comparison_with_zero() {
        test_item(
            Query::ComparisonWithZero(ComparisonWithZero::new(
                ArithmeticQueryParams {
                    signed: true,
                    comparison_type: ArithmeticComparisonType::Inequal,
                },
                vec![0, 1, 2],
                FileOffset {
                    file_id: 4,
                    offset: 5u32.into(),
                },
            )),
            &hex!("38 03 000102 04 05"),
        )
    }

    #[test]
    fn test_query_comparison_with_value() {
        test_item(
            Query::ComparisonWithValue(ComparisonWithValue::new(
                ArithmeticQueryParams {
                    signed: false,
                    comparison_type: ArithmeticComparisonType::Equal,
                },
                vec![],
                vec![9, 9, 9],
                FileOffset {
                    file_id: 4,
                    offset: 5u32.into(),
                },
            )),
            &hex!("41 03 090909 04 05"),
        )
    }

    #[test]
    fn test_query_comparison_with_other_file() {
        test_item(
            Query::ComparisonWithOtherFile(ComparisonWithOtherFile::new(
                ArithmeticQueryParams {
                    signed: false,
                    comparison_type: ArithmeticComparisonType::GreaterThan,
                },
                vec![0xFF, 0xFF],
                FileOffset {
                    file_id: 4,
                    offset: 5u32.into(),
                },
                FileOffset {
                    file_id: 8,
                    offset: 9u32.into(),
                },
            )),
            &hex!("74 02 FFFF 04 05 08 09"),
        )
    }

    #[test]
    fn test_query_bitmap_range_comparison() {
        test_item(
            Query::BitmapRangeComparison(BitmapRangeComparison::new(
                RangeQueryParams {
                    signed: false,
                    comparison_type: RangeComparisonType::InRange,
                },
                3,
                32,
                hex!("01020304").to_vec(),
                FileOffset {
                    file_id: 0,
                    offset: 4u32.into(),
                },
            )),
            &hex!("91 04 03  20  01020304  00 04"),
        )
    }

    #[test]
    fn test_query_string_token_search() {
        test_item(
            Query::StringTokenSearch(StringTokenSearch::new(
                2,
                hex!("FF00FF00").to_vec(),
                hex!("01020304").to_vec(),
                FileOffset {
                    file_id: 0,
                    offset: 4u32.into(),
                },
            )),
            &hex!("F2 04 FF00FF00  01020304  00 04"),
        )
    }
}
