use deku::prelude::*;

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 3, endian = "big", type = "u8")]
pub enum ActionCondition {
    #[deku(id = "0")]
    List,
    #[deku(id = "1")]
    Read,
    #[deku(id = "2")]
    Write,
    #[deku(id = "3")]
    WriteFlush,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
#[deku(bits = 2, type = "u8")]
pub enum StorageClass {
    /// The content is not kept in memory. It cannot be read back.
    #[deku(id = "0")]
    Transient,
    /// The content is kept in a volatile memory of the device. It is accessible for
    /// read, and is lost on power off.
    #[deku(id = "1")]
    Volatile,
    /// The content is kept in a volatile memory of the device. It is accessible for
    /// read, and can be backed-up upon request in a permanent storage
    /// location. It is restored from the permanent location on device power on.
    #[deku(id = "2")]
    Restorable,
    /// The content is kept in a permanent memory of the device. It is accessible
    /// for read and write.
    #[deku(id = "3")]
    Permanent,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct UserPermissions {
    #[deku(bits = 1)]
    pub read: bool,
    #[deku(bits = 1)]
    pub write: bool,
    #[deku(bits = 1)]
    pub executable: bool,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct FilePermissions {
    #[deku(bits = 1)]
    pub encrypted: bool,
    #[deku(bits = 1)]
    pub executable: bool,
    pub user: UserPermissions,
    pub guest: UserPermissions,
}


#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct FileProperties {
    /// Enables the D7AActP (ALP action to trigger upon some type of access to this file)
    #[deku(bits=1)]
    pub enabled: bool,

    /// Type of access needed to trigger the D7AActP
    pub condition: ActionCondition,

    /// Type of storage of this file
    #[deku(pad_bits_before = "2")]
    pub storage_class: StorageClass,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Clone, PartialEq)]
pub struct FileHeader {
    pub permissions: FilePermissions,
    pub properties: FileProperties,
    pub alp_command_file_id: u8,
    pub interface_file_id: u8,
    #[deku(endian="big")]
    pub file_size: u32,
    #[deku(endian="big")]
    pub allocated_size: u32,
}

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_file_permissions() {

        let permissions = FilePermissions {
                encrypted: true,
                executable: false,
                user: UserPermissions { read: true, write: true, executable: true },
                guest: UserPermissions { read: false, write: false, executable: false },
        };

        let expected = hex!("B8");


        assert_eq!(permissions.to_bytes().unwrap(), expected);
        assert_eq!(FilePermissions::from_bytes((&expected, 0)).unwrap().1, permissions);
    }

    #[test]
    fn test_file_header() {
        let header = FileHeader{
            permissions: FilePermissions {
                encrypted: true,
                executable: false,
                user: UserPermissions { read: true, write: true, executable: true },
                guest: UserPermissions { read: false, write: false, executable: false },
            },
            properties: FileProperties { enabled: false, condition: ActionCondition::Read, storage_class: StorageClass::Permanent },
            alp_command_file_id: 1,
            interface_file_id: 2,
            file_size: 0xDEAD_BEEF,
            allocated_size: 0xBAAD_FACE,
        };

        let expected = hex!("B8 13 01 02 DEADBEEF BAADFACE");

        assert_eq!(header.to_bytes().unwrap(), expected);
        assert_eq!(FileHeader::from_bytes((&expected, 0)).unwrap().1, header);
    }
}
