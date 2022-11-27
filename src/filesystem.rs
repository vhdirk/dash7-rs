use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
#[bits = 3]
#[endian = 1]
#[repr(C)]
pub enum ActionCondition {
    List = 0,
    Read = 1,
    Write = 2,
    WriteFlush = 3,
}

#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
#[bits = 2]
#[endian = 1]
#[repr(C)]
pub enum StorageClass {
    /// The content is not kept in memory. It cannot be read back.
    Transient = 0,
    /// The content is kept in a volatile memory of the device. It is accessible for
    /// read, and is lost on power off.
    Volatile = 1,
    /// The content is kept in a volatile memory of the device. It is accessible for
    /// read, and can be backed-up upon request in a permanent storage
    /// location. It is restored from the permanent location on device power on.
    Restorable = 2,
    /// The content is kept in a permanent memory of the device. It is accessible
    /// for read and write.
    Permanent = 3,
}

#[bitfield(bits = 3)]
#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct UserPermissions {
    pub read: bool,
    pub write: bool,
    pub executable: bool,
}

#[bitfield]
#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct FilePermissions {
    pub encrypted: bool,
    pub executable: bool,
    // pub user: UserPermissions,
    // pub guest: UserPermissions,


    pub user_readable: bool,
    pub user_writable: bool,
    pub user_executable: bool,
    pub guest_readable: bool,
    pub guest_writable: bool,
    pub guest_executable: bool,
}

#[bitfield]
#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct FileProperties {
    /// Enables the D7AActP (ALP action to trigger upon some type of access to this file)
    pub enabled: bool,
    /// Type of access needed to trigger the D7AActP
    pub condition: ActionCondition,

    #[skip]
    __: B2,
    /// Type of storage of this file
    pub storage_class: StorageClass,
}

#[bitfield]
#[derive(BitfieldSpecifier, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct FileHeader {
    pub permissions: FilePermissions,
    pub properties: FileProperties,
    pub alp_command_file_id: u8,
    pub interface_file_id: u8,
    pub file_size: u32,
    pub allocated_size: u32,
}

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_file_permissions() {
        let permissions = FilePermissions::new()
            .with_encrypted(true)
            .with_executable(false)

                    .with_user_readable(true)
                    .with_user_writable(true)
                    .with_user_executable(true)
                    .with_guest_readable(false)
                    .with_guest_writable(false)
                    .with_guest_executable(false)
            ;

        let expected = hex!("B8");

        assert_eq!(permissions.bytes, expected);
        assert_eq!(FilePermissions::from_bytes(expected), permissions);
    }

    // #[test]
    // fn test_file_header() {
    //     let header = FileHeader::new()
    //         .with_permissions(
    //             FilePermissions::new()
    //                 .with_encrypted(true)
    //                 .with_executable(false)
    //                 .with_user(
    //                     UserPermissions::new()
    //                         .with_read(true)
    //                         .with_write(true)
    //                         .with_executable(true),
    //                 )
    //                 .with_guest(
    //                     UserPermissions::new()
    //                         .with_read(false)
    //                         .with_write(false)
    //                         .with_executable(false),
    //                 ),
    //         )
    //         .with_properties(
    //             FileProperties::new()
    //                 .with_enabled(false)
    //                 .with_condition(ActionCondition::Read)
    //                 .with_storage_class(StorageClass::Permanent),
    //         )
    //         .with_alp_command_file_id(1)
    //         .with_interface_file_id(2)
    //         .with_file_size(0xDEAD_BEEF)
    //         .with_allocated_size(0xBAAD_FACE);

    //     let expected = hex!("B8 13 01 02 DEADBEEF BAADFACE");

    //     assert_eq!(header.bytes, expected);
    //     assert_eq!(FileHeader::from_bytes(expected), header);
    // }
}
