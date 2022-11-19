use packed_struct::prelude::*;

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum ActionCondition {
    List = 0,
    Read = 1,
    Write = 2,
    WriteFlush = 3,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, Debug, PartialEq)]
pub enum StorageClass {
    Transient = 0,
    Volatile = 1,
    Restorable = 2,
    Permanent = 3,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct FilePermissions {
    #[packed_field(bits = "0")]
    encrypted: bool,
    #[packed_field(bits = "1")]
    executable: bool,
    #[packed_field(bits = "2")]
    user_readable: bool,
    #[packed_field(bits = "3")]
    user_writable: bool,
    #[packed_field(bits = "4")]
    user_executable: bool,
    #[packed_field(bits = "5")]
    guest_readable: bool,
    #[packed_field(bits = "6")]
    guest_writable: bool,
    #[packed_field(bits = "7")]
    guest_executable: bool,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct FileProperties {
    #[packed_field(bits = "0")]
    enabled: bool,

    #[packed_field(bits = "1..=3", ty = "enum")]
    condition: ActionCondition,

    #[packed_field(bits = "4..=5")]
    rfu: Integer<u8, packed_bits::Bits<2>>,

    #[packed_field(bits = "6..=7", ty = "enum")]
    storage_class: StorageClass,
}

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0")]
pub struct FileHeader {
    #[packed_field(element_size_bytes="1")]
    permissions: FilePermissions,

    #[packed_field(element_size_bytes="1")]
    properties: FileProperties,

    alp_command_file_id: u8,
    interface_file_id: u8,

    #[packed_field(endian="msb")]
    file_size: u32,

    #[packed_field(endian="msb")]
    allocated_size: u32,
}
